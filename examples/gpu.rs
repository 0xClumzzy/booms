//! Headless wgpu rendering inside the terminal (requires the `gpu` feature).
//!
//! Run with `cargo run --features gpu --example gpu`.
//!
//! A full-screen WGSL plasma shader renders offscreen, the pixels are read
//! back to the CPU, and `GpuSurface` paints them into the terminal using the
//! truecolor half-block trick (`▀`, top pixel as foreground, bottom as
//! background). If no GPU/software adapter is available, a text fallback is
//! drawn instead of crashing.

use booms::app::App;
use booms::gpu::GpuSurface;
use booms::style::{Color, Style};
use booms::widgets::{Text, Widget};

/// Header showing whether the GPU pipeline initialized successfully.
struct Status {
    available: bool,
    error: Option<String>,
}

impl Widget for Status {
    fn constraint(&self) -> booms::Constraint {
        booms::Constraint::Exact(2)
    }

    fn render(&self, area: booms::Rect, grid: &mut booms::Grid) {
        let (label, color) = if self.available {
            ("wgpu plasma: GPU active", Color::Rgb(166, 227, 161))
        } else {
            (
                self.error
                    .as_deref()
                    .unwrap_or("wgpu plasma: no GPU adapter"),
                Color::Rgb(243, 139, 168),
            )
        };
        grid.set_str(area.x, area.y, label, Style::DEFAULT.fg(color));
        grid.set_str(
            area.x,
            area.y + 1,
            "every frame: offscreen render → pixel readback → half-block cells",
            Style::DEFAULT.fg(Color::Rgb(108, 112, 134)),
        );
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let surface = GpuSurface::new();
    let status = Status {
        available: surface.available(),
        error: surface.error().map(str::to_string),
    };

    let mut app = App::new()?;
    app.add(Text::new(["booms + wgpu".to_string()]));
    app.add(status);
    app.add(surface);
    app.run()?;
    Ok(())
}
