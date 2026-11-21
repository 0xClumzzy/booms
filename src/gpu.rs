//! Headless GPU rendering for `booms` (feature `gpu`).
//!
//! [`GpuSurface`] owns an offscreen [`wgpu`] pipeline: a full-screen triangle
//! runs an animated fragment shader, the resulting texture is copied into a
//! staging buffer, and the pixels are read back to the CPU. The widget then
//! paints those pixels into a [`Grid`] using the truecolor *half-block* trick:
//! each terminal cell becomes two vertical pixels via `▀` (foreground = top
//! pixel, background = bottom pixel).
//!
//! No window or surface is created: rendering is fully offscreen. A GPU (or
//! a software adapter such as Vulkan lavapipe / llvmpipe) must be available at
//! runtime; if initialization fails, [`GpuSurface`] degrades to a text message
//! rather than crashing the app.
//!
//! ```
//! # #[cfg(feature = "gpu")]
//! # {
//! use booms::gpu::GpuSurface;
//! use booms::widgets::Widget;
//!
//! let mut surface = GpuSurface::new();
//! if surface.available() {
//!     surface.on_tick(); // render the next frame
//!     assert!(surface.time() >= 0.0);
//! }
//! # }
//! ```

use std::error::Error;
use std::fmt;

use crate::grid::Grid;
use crate::layout::Rect;
use crate::style::{Color, Style};
use crate::widgets::Widget;

/// Size of the offscreen render target, in pixels. Kept modest: readback is a
/// synchronization point and the terminal downsamples it anyway.
const TEX_SIZE: u32 = 128;

const SHADER: &str = r#"
struct Uniforms {
    time: f32,
    width: f32,
    height: f32,
    _pad: f32,
};
@group(0) @binding(0) var<uniform> u: Uniforms;

struct VOut {
    @builtin(position) pos: vec4f,
    @location(0) uv: vec2f,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VOut {
    var p = array<vec2f, 3>(
        vec2f(-1.0, -1.0),
        vec2f( 3.0, -1.0),
        vec2f(-1.0,  3.0),
    );
    let xy = p[vi];
    var out: VOut;
    out.pos = vec4f(xy, 0.0, 1.0);
    out.uv = xy * 0.5 + 0.5;
    return out;
}

@fragment
fn fs_main(in: VOut) -> @location(0) vec4f {
    let uv = in.uv;
    let t = u.time;
    let v = sin(uv.x * 10.0 + t)
          + sin(uv.y * 10.0 + t * 1.3)
          + sin((uv.x + uv.y) * 10.0 + t * 0.7)
          + sin(length(uv - 0.5) * 20.0 - t * 2.0);
    let n = v * 0.125 + 0.5;
    let c1 = vec3f(0.07, 0.08, 0.16);
    let c2 = vec3f(0.54, 0.71, 0.98);
    let c3 = vec3f(0.98, 0.75, 0.57);
    let c4 = vec3f(0.96, 0.64, 0.85);
    var col = mix(c1, c2, smoothstep(0.0, 0.4, n));
    col = mix(col, c3, smoothstep(0.4, 0.7, n));
    col = mix(col, c4, smoothstep(0.7, 1.0, n));
    return vec4f(col, 1.0);
}
"#;

/// Error returned when a GPU adapter cannot be acquired.
#[derive(Debug)]
pub enum GpuError {
    NoAdapter,
    Device(String),
}

impl fmt::Display for GpuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GpuError::NoAdapter => write!(f, "no compatible GPU adapter found"),
            GpuError::Device(msg) => write!(f, "GPU device request failed: {msg}"),
        }
    }
}

impl Error for GpuError {}

/// Owns the wgpu resources for one offscreen render target.
struct GpuRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    output_buffer: wgpu::Buffer,
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    bytes_per_row: u32,
    pixels: Vec<u8>,
}

impl GpuRenderer {
    fn new() -> Result<Self, Box<dyn Error>> {
        let mut instance_desc = wgpu::InstanceDescriptor::new_without_display_handle();
        instance_desc.backends = wgpu::Backends::all();
        let instance = wgpu::Instance::new(instance_desc);
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            force_fallback_adapter: false,
            compatible_surface: None,
            apply_limit_buckets: false,
        }))
        .map_err(|e| GpuError::Device(e.to_string()))?;

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("booms offscreen device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::default(),
            ..Default::default()
        }))
        .map_err(|e| GpuError::Device(e.to_string()))?;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("booms plasma shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("booms uniforms"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("booms plasma pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview_mask: None,
            cache: None,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("booms offscreen target"),
            size: wgpu::Extent3d {
                width: TEX_SIZE,
                height: TEX_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Row stride must be a multiple of COPY_BYTES_PER_ROW_ALIGNMENT (256).
        let unpadded = TEX_SIZE * 4;
        let bytes_per_row = unpadded.div_ceil(256) * 256;

        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("booms readback buffer"),
            size: (bytes_per_row * TEX_SIZE) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let pixels = vec![0u8; (bytes_per_row * TEX_SIZE) as usize];

        Ok(Self {
            device,
            queue,
            pipeline,
            bind_group,
            uniform_buffer,
            output_buffer,
            texture,
            texture_view,
            bytes_per_row,
            pixels,
        })
    }

    fn render(&mut self, time: f32) -> Result<(), Box<dyn Error>> {
        // Uniforms: time, width, height, padding (4 x f32 = 16 bytes).
        let mut uniform = [0u8; 16];
        uniform[0..4].copy_from_slice(&time.to_ne_bytes());
        uniform[4..8].copy_from_slice(&(TEX_SIZE as f32).to_ne_bytes());
        uniform[8..12].copy_from_slice(&(TEX_SIZE as f32).to_ne_bytes());
        self.queue.write_buffer(&self.uniform_buffer, 0, &uniform);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("booms render encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("booms render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &self.output_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(self.bytes_per_row),
                    rows_per_image: Some(TEX_SIZE),
                },
            },
            wgpu::Extent3d {
                width: TEX_SIZE,
                height: TEX_SIZE,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit(Some(encoder.finish()));

        let slice = self.output_buffer.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| GpuError::Device(e.to_string()))?;
        {
            let view = slice
                .get_mapped_range()
                .map_err(|e| GpuError::Device(e.to_string()))?;
            self.pixels.copy_from_slice(&view[..]);
        }
        self.output_buffer.unmap();
        Ok(())
    }

    /// Top-left-origin RGB pixel at `(x, y)`.
    fn pixel(&self, x: usize, y: usize) -> (u8, u8, u8) {
        let bpr = self.bytes_per_row as usize;
        let i = y * bpr + x * 4;
        (self.pixels[i], self.pixels[i + 1], self.pixels[i + 2])
    }
}

/// A terminal widget backed by an offscreen [`wgpu`] render.
///
/// On each [`Widget::on_tick`] it advances the shader clock and reads back a
/// fresh frame. Rendering samples the framebuffer with nearest-neighbour
/// scaling into the widget area, packing two pixel rows per cell via `▀`.
pub struct GpuSurface {
    renderer: Option<GpuRenderer>,
    time: f32,
    speed: f32,
    error: Option<String>,
}

impl GpuSurface {
    /// Create the widget, attempting GPU initialization immediately.
    pub fn new() -> Self {
        match GpuRenderer::new() {
            Ok(mut renderer) => {
                let _ = renderer.render(0.0);
                Self {
                    renderer: Some(renderer),
                    time: 0.0,
                    speed: 0.02,
                    error: None,
                }
            }
            Err(e) => Self {
                renderer: None,
                time: 0.0,
                speed: 0.02,
                error: Some(e.to_string()),
            },
        }
    }

    /// Seconds-equivalent added to the shader clock per tick.
    pub fn speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    /// Whether a GPU device was successfully initialized.
    pub fn available(&self) -> bool {
        self.renderer.is_some()
    }

    /// Why GPU initialization failed, if it did.
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Current shader clock.
    pub fn time(&self) -> f32 {
        self.time
    }
}

impl Default for GpuSurface {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for GpuSurface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GpuSurface")
            .field("available", &self.available())
            .field("time", &self.time)
            .field("error", &self.error)
            .finish()
    }
}

impl Widget for GpuSurface {
    fn constraint(&self) -> crate::layout::Constraint {
        crate::layout::Constraint::Flex(1)
    }

    fn on_tick(&mut self) {
        if let Some(renderer) = &mut self.renderer {
            self.time += self.speed;
            if let Err(e) = renderer.render(self.time) {
                self.error = Some(e.to_string());
                self.renderer = None;
            }
        }
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let Some(renderer) = &self.renderer else {
            let msg = self
                .error
                .as_deref()
                .unwrap_or("GPU unavailable (enable feature `gpu`)");
            grid.set_str(area.x, area.y, msg, Style::DEFAULT.fg(Color::Ansi(9)));
            return;
        };

        let cells_w = area.width as usize;
        let cells_h = area.height as usize;
        let tex = TEX_SIZE as usize;
        for cy in 0..cells_h {
            for cx in 0..cells_w {
                // Nearest-neighbour sample: map cell to a pixel column, and
                // two pixel rows (top/bottom of the half block) to this cell.
                let px = (cx * tex / cells_w).min(tex - 1);
                let py_top = (cy * 2 * tex / (cells_h * 2)).min(tex - 1);
                let py_bot = ((cy * 2 + 1) * tex / (cells_h * 2)).min(tex - 1);
                let (tr, tg, tb) = renderer.pixel(px, py_top);
                let (br, bg, bb) = renderer.pixel(px, py_bot);
                let style = Style::DEFAULT
                    .fg(Color::Rgb(tr, tg, tb))
                    .bg(Color::Rgb(br, bg, bb));
                grid.set(area.x + cx as u16, area.y + cy as u16, '▀', style);
            }
        }
    }
}
