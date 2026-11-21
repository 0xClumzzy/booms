//! Glow-style markdown rendering: a Rust port of the glow/glamour
//! approach: markdown in, styled [`MdLine`]s out.
//!
//! [`render_markdown`] turns a GitHub-flavored markdown string into lines
//! of styled spans using ANSI colors and modifiers: colored headings with
//! underline, bold and italic text, dim inline code, underlined links
//! followed by the URL, left-margined blockquotes, lists with bullets,
//! numbers and task-list markers (`✓` / `✗`), and boxed code blocks.
//!
//! The output is pure data: paint it with [`paint_md_line`] or read
//! [`MdLine::spans`] to draw it however you like (the agent harness
//! prefixes role labels onto the first line of each message this way).

use crate::grid::{Grid, char_width, str_width};
use crate::style::{Color, Modifiers, Style};

/// A rendered markdown line: a run of styled spans.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MdLine {
    /// The spans, in drawing order.
    pub spans: Vec<MdSpan>,
}

impl MdLine {
    /// The display width of the line.
    pub fn width(&self) -> usize {
        self.spans.iter().map(|s| str_width(&s.text)).sum()
    }

    /// True when the line has no visible text.
    pub fn is_empty(&self) -> bool {
        self.spans.iter().all(|s| s.text.is_empty())
    }
}

/// A styled run of text within an [`MdLine`].
#[derive(Clone, Debug, PartialEq)]
pub struct MdSpan {
    /// The span's text.
    pub text: String,
    /// The span's style.
    pub style: Style,
}

impl MdSpan {
    /// Build a span.
    pub fn new(text: impl Into<String>, style: Style) -> MdSpan {
        MdSpan {
            text: text.into(),
            style,
        }
    }
}

/// Render markdown text into styled lines of at most `width` columns.
///
/// `role_color` colors list bullets and numbers, so a message can be
/// tinted per its speaker (blue for the user, yellow for the agent).
pub fn render_markdown(text: &str, width: usize, role_color: Color) -> Vec<MdLine> {
    let mut lines = Vec::new();
    let mut in_code_block = false;
    let mut code_block_lang = String::new();
    let mut code_buffer: Vec<String> = Vec::new();

    for raw_line in text.lines() {
        let line = raw_line.trim_end();

        // Code block fences: ```lang … ```
        if line.starts_with("```") {
            if !in_code_block {
                in_code_block = true;
                code_block_lang = line
                    .strip_prefix("```")
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                code_buffer.clear();
            } else {
                in_code_block = false;
                lines.extend(render_code_block(&code_buffer, &code_block_lang, width));
                code_buffer.clear();
                code_block_lang.clear();
            }
            continue;
        }

        if in_code_block {
            code_buffer.push(line.to_string());
            continue;
        }

        lines.extend(render_line(line, width, role_color));
    }

    // Handle an unclosed code block.
    if in_code_block && !code_buffer.is_empty() {
        lines.extend(render_code_block(&code_buffer, &code_block_lang, width));
    }

    // Always emit at least one line.
    if lines.is_empty() {
        lines.push(MdLine::default());
    } else if lines.last().map(|l| l.is_empty()).unwrap_or(false) {
        lines.pop();
        lines.push(MdLine::default());
    } else {
        lines.push(MdLine { spans: vec![] });
    }

    lines
}

/// Paint one [`MdLine`] into `grid` at `(x, y)`; returns the next `x`.
pub fn paint_md_line(grid: &mut Grid, x: u16, y: u16, line: &MdLine) -> u16 {
    let mut cx = x;
    for span in &line.spans {
        cx = grid.set_str(cx, y, &span.text, span.style);
    }
    cx
}

/// The plain text of a rendered markdown document, for tests and previews.
pub fn lines_text(lines: &[MdLine]) -> String {
    lines
        .iter()
        .map(|l| l.spans.iter().map(|s| s.text.as_str()).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

// ---------------------------------------------------------------------------
// Line structure
// ---------------------------------------------------------------------------

fn render_line(line: &str, width: usize, role_color: Color) -> Vec<MdLine> {
    let mut result = Vec::new();

    if line.is_empty() {
        result.push(MdLine { spans: vec![] });
        return result;
    }

    // Headings: `#` … `######`
    if let Some((level, content)) = strip_heading(line) {
        result.push(MdLine {
            spans: vec![MdSpan::new(content, heading_style(level))],
        });
        return result;
    }

    // Horizontal rule: `---`, `***`, `___` and friends.
    if is_hr(line) {
        result.push(MdLine {
            spans: vec![MdSpan::new(
                "━".repeat(width.clamp(1, 60)),
                Style::DEFAULT.fg(Color::Ansi(8)),
            )],
        });
        return result;
    }

    // Blockquote: `>` with a left margin.
    if let Some(content) = line.strip_prefix('>') {
        let mut spans = vec![MdSpan::new(
            "▌ ",
            Style::DEFAULT.fg(Color::Ansi(8)).mods(Modifiers::BOLD),
        )];
        spans.extend(
            parse_inline(content.trim_start())
                .into_iter()
                .map(|mut span| {
                    span.style = span.style.mods(Modifiers::ITALIC);
                    span
                }),
        );
        result.extend(wrap_spans(spans, width));
        return result;
    }

    // List items: `-`, `*`, `+`, `1.`, and task lists `- [ ]` / `- [x]`.
    if let Some(item) = parse_list_item(line, role_color) {
        return render_list_item(&item, width);
    }

    // Regular paragraph with inline formatting.
    result.extend(wrap_spans(parse_inline(line), width));
    result
}

fn strip_heading(line: &str) -> Option<(usize, &str)> {
    let level = line.chars().take_while(|&c| c == '#').count();
    if level == 0 {
        return None;
    }
    let after = &line[level..];
    if !after.is_empty() && !after.starts_with(' ') && !after.starts_with('\t') {
        return None;
    }
    Some((level.min(6), after.trim_start()))
}

fn heading_style(level: usize) -> Style {
    match level {
        1 => Style::DEFAULT
            .fg(Color::Ansi(13))
            .mods(Modifiers::BOLD | Modifiers::UNDERLINE),
        2 => Style::DEFAULT
            .fg(Color::Ansi(11))
            .mods(Modifiers::BOLD | Modifiers::UNDERLINE),
        3 => Style::DEFAULT.fg(Color::Ansi(14)).mods(Modifiers::BOLD),
        4 => Style::DEFAULT.fg(Color::Ansi(10)).mods(Modifiers::BOLD),
        5 => Style::DEFAULT.fg(Color::Ansi(12)).mods(Modifiers::BOLD),
        _ => Style::DEFAULT.fg(Color::Ansi(8)).mods(Modifiers::BOLD),
    }
}

fn is_hr(line: &str) -> bool {
    let t: String = line.trim().chars().filter(|c| !c.is_whitespace()).collect();
    t.len() >= 3 && t.chars().all(|c| matches!(c, '-' | '*' | '_'))
}

// ---------------------------------------------------------------------------
// Lists
// ---------------------------------------------------------------------------

struct ListItem {
    indent: usize,
    marker: String,
    marker_style: Style,
    content: String,
}

fn parse_list_item(line: &str, role_color: Color) -> Option<ListItem> {
    let trimmed = line.trim_start();
    if trimmed.is_empty() {
        return None;
    }
    let indent = str_width(&line[..line.len() - trimmed.len()]);

    // Unordered markers: `- `, `* `, `+ `
    for prefix in ["- ", "* ", "+ "] {
        if let Some(content) = trimmed.strip_prefix(prefix) {
            if let Some((marker_text, marker_style, text)) = parse_task(content) {
                return Some(ListItem {
                    indent,
                    marker: marker_text.to_string(),
                    marker_style,
                    content: text,
                });
            }
            return Some(ListItem {
                indent,
                marker: "• ".to_string(),
                marker_style: Style::DEFAULT.fg(role_color).mods(Modifiers::BOLD),
                content: content.to_string(),
            });
        }
    }

    // Ordered markers: `1. ` or `1) `: keep the real number.
    let digits: String = trimmed.chars().take_while(|c| c.is_ascii_digit()).collect();
    if !digits.is_empty() {
        let after = &trimmed[digits.len()..];
        let sep = if after.starts_with(')') { ") " } else { ". " };
        if let Some(content) = after.strip_prefix(sep) {
            return Some(ListItem {
                indent,
                marker: format!("{}{}", digits, sep),
                marker_style: Style::DEFAULT.fg(role_color).mods(Modifiers::BOLD),
                content: content.to_string(),
            });
        }
    }

    None
}

/// Parse the `[ ]` / `[x]` task marker after a list bullet.
fn parse_task(content: &str) -> Option<(&'static str, Style, String)> {
    let inner = content.strip_prefix('[')?;
    let (marker, rest) = inner.split_once(']')?;
    let text = rest.strip_prefix(' ').unwrap_or(rest).to_string();
    let checked = marker.trim().eq_ignore_ascii_case("x");
    let (marker_text, marker_style) = if checked {
        (
            "✓ ",
            Style::DEFAULT.fg(Color::Ansi(10)).mods(Modifiers::BOLD),
        )
    } else {
        (
            "✗ ",
            Style::DEFAULT.fg(Color::Ansi(8)).mods(Modifiers::BOLD),
        )
    };
    Some((marker_text, marker_style, text))
}

fn render_list_item(item: &ListItem, width: usize) -> Vec<MdLine> {
    let indent = item.indent.min(8);
    let indent_str = " ".repeat(indent);
    let marker_w = str_width(&item.marker);
    let cont_prefix = format!("{}{}", indent_str, " ".repeat(marker_w));
    let content_width = width.saturating_sub(indent + marker_w).max(4);

    let wrapped = wrap_spans(parse_inline(&item.content), content_width);

    let mut lines = Vec::new();
    for (i, sp_line) in wrapped.into_iter().enumerate() {
        let mut spans = vec![if i == 0 {
            MdSpan::new(item.marker.clone(), item.marker_style)
        } else {
            MdSpan::new(cont_prefix.clone(), Style::DEFAULT)
        }];
        spans.extend(sp_line.spans);
        lines.push(MdLine { spans });
    }
    lines
}

// ---------------------------------------------------------------------------
// Inline parsing
// ---------------------------------------------------------------------------

fn parse_inline(text: &str) -> Vec<MdSpan> {
    let mut spans: Vec<MdSpan> = Vec::new();
    let mut rest = text;

    while !rest.is_empty() {
        // Locate the earliest special marker. Ties at the same index are
        // broken by priority: strike > bold > italic > code > link.
        let mut cands: Vec<(usize, usize, u8, &'static str)> = Vec::new();
        if let Some(i) = rest.find("~~") {
            cands.push((i, 2, 4, "strike"));
        }
        if let Some(i) = rest.find("**") {
            cands.push((i, 2, 3, "bold"));
        }
        if let Some(i) = rest.find('*') {
            cands.push((i, 1, 2, "italic"));
        }
        if let Some(i) = rest.find('`') {
            cands.push((i, 1, 1, "code"));
        }
        if let Some(i) = rest.find('[') {
            cands.push((i, 1, 0, "link"));
        }
        cands.sort_by_key(|c| (c.0, std::cmp::Reverse(c.2)));

        let Some(&(idx, len, _prio, kind)) = cands.first() else {
            spans.push(MdSpan::new(rest, base_style()));
            break;
        };

        push_if_nonempty(&mut spans, &rest[..idx], base_style());
        rest = &rest[idx + len..];

        match kind {
            "strike" => match rest.find("~~") {
                Some(j) => {
                    push_if_nonempty(&mut spans, &rest[..j], strike_style());
                    rest = &rest[j + 2..];
                }
                None => spans.push(MdSpan::new("~~", base_style())),
            },
            "bold" => match rest.find("**") {
                Some(j) => {
                    push_if_nonempty(&mut spans, &rest[..j], bold_style());
                    rest = &rest[j + 2..];
                }
                None => spans.push(MdSpan::new("**", base_style())),
            },
            "italic" => match rest.find('*') {
                Some(j) => {
                    push_if_nonempty(&mut spans, &rest[..j], italic_style());
                    rest = &rest[j + 1..];
                }
                None => spans.push(MdSpan::new("*", base_style())),
            },
            "code" => match rest.find('`') {
                Some(j) => {
                    push_if_nonempty(&mut spans, &rest[..j], code_style());
                    rest = &rest[j + 1..];
                }
                None => spans.push(MdSpan::new("`", base_style())),
            },
            _ => match rest.find("](") {
                Some(j) => {
                    let link_text = &rest[..j];
                    let after = &rest[j + 2..];
                    match after.find(')') {
                        Some(k) => {
                            let url = &after[..k];
                            push_if_nonempty(&mut spans, link_text, link_text_style());
                            if !url.is_empty() {
                                spans.push(MdSpan {
                                    text: format!(" ({})", url),
                                    style: url_style(),
                                });
                            }
                            rest = &after[k + 1..];
                        }
                        None => spans.push(MdSpan::new("[", base_style())),
                    }
                }
                None => spans.push(MdSpan::new("[", base_style())),
            },
        }
    }

    spans
}

fn push_if_nonempty(spans: &mut Vec<MdSpan>, text: &str, style: Style) {
    if !text.is_empty() {
        spans.push(MdSpan::new(text, style));
    }
}

// ---------------------------------------------------------------------------
// Styles
// ---------------------------------------------------------------------------

fn base_style() -> Style {
    Style::DEFAULT.fg(Color::Ansi(15))
}

fn bold_style() -> Style {
    Style::DEFAULT.fg(Color::Ansi(11)).mods(Modifiers::BOLD)
}

fn italic_style() -> Style {
    Style::DEFAULT.fg(Color::Ansi(15)).mods(Modifiers::ITALIC)
}

fn strike_style() -> Style {
    Style::DEFAULT
        .fg(Color::Ansi(8))
        .mods(Modifiers::STRIKETHROUGH)
}

fn code_style() -> Style {
    Style::DEFAULT
        .fg(Color::Ansi(11))
        .bg(Color::Ansi(0))
        .mods(Modifiers::DIM)
}

fn link_text_style() -> Style {
    Style::DEFAULT
        .fg(Color::Ansi(14))
        .mods(Modifiers::UNDERLINE)
}

fn url_style() -> Style {
    Style::DEFAULT.fg(Color::Ansi(8)).mods(Modifiers::DIM)
}

// ---------------------------------------------------------------------------
// Wrapping
// ---------------------------------------------------------------------------

fn wrap_spans(spans: Vec<MdSpan>, width: usize) -> Vec<MdLine> {
    let width = width.max(1);

    // Flatten to (char, style) cells.
    let mut cells: Vec<(char, Style)> = Vec::new();
    for span in spans {
        for ch in span.text.chars() {
            cells.push((ch, span.style));
        }
    }

    let mut lines: Vec<Vec<MdSpan>> = Vec::new();
    let mut cur: Vec<MdSpan> = Vec::new();
    let mut cur_w = 0usize;

    for (ch, style) in cells {
        if ch == '\n' {
            lines.push(std::mem::take(&mut cur));
            cur_w = 0;
            continue;
        }
        let w = char_width(ch) as usize;
        if w > 0 && cur_w + w > width && cur_w > 0 {
            lines.push(std::mem::take(&mut cur));
            cur_w = 0;
        }
        if let Some(last) = cur.last_mut() {
            if last.style == style {
                last.text.push(ch);
                if w > 0 {
                    cur_w += w;
                }
                continue;
            }
        }
        cur.push(MdSpan {
            text: ch.to_string(),
            style,
        });
        if w > 0 {
            cur_w += w;
        }
    }
    if !cur.is_empty() || lines.is_empty() {
        lines.push(cur);
    }

    lines.into_iter().map(|spans| MdLine { spans }).collect()
}

// ---------------------------------------------------------------------------
// Code blocks
// ---------------------------------------------------------------------------

/// Truncate to at most `max` characters (char-safe, unlike byte slicing).
fn truncate_chars(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

fn render_code_block(lines: &[String], lang: &str, width: usize) -> Vec<MdLine> {
    let mut result = Vec::new();
    let inner = width.saturating_sub(4).max(8);

    // Language label with a top border.
    let label = if lang.is_empty() { "code" } else { lang };
    let top = format!(" ╭─ {} ", label);
    result.push(MdLine {
        spans: vec![MdSpan::new(
            truncate_chars(&top, inner),
            Style::DEFAULT.fg(Color::Ansi(8)).mods(Modifiers::ITALIC),
        )],
    });

    // Code lines with a background tint.
    for line in lines {
        let display = if char_width_of(line) > inner {
            format!("{}…", truncate_chars(line, inner.saturating_sub(1)))
        } else {
            line.clone()
        };
        let body = format!(" │ {}", display);
        result.push(MdLine {
            spans: vec![MdSpan::new(
                truncate_chars(&body, inner + 1),
                Style::DEFAULT.fg(Color::Ansi(11)).bg(Color::Ansi(0)),
            )],
        });
    }

    // Bottom border.
    result.push(MdLine {
        spans: vec![MdSpan::new(
            " ╰─",
            Style::DEFAULT.fg(Color::Ansi(8)).mods(Modifiers::ITALIC),
        )],
    });

    result
}

fn char_width_of(s: &str) -> usize {
    str_width(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(lines: &[MdLine]) -> String {
        lines_text(lines)
    }

    #[test]
    fn heading_styles() {
        let lines = render_markdown("# Big\n## Sub\n### Small", 40, Color::Ansi(10));
        assert!(flat(&lines).contains("Big"));
        assert!(lines[0].spans[0].style.mods.contains(Modifiers::UNDERLINE));
        assert!(lines[1].spans[0].style.mods.contains(Modifiers::UNDERLINE));
        assert!(!lines[2].spans[0].style.mods.contains(Modifiers::UNDERLINE));
    }

    #[test]
    fn bold_and_italic() {
        let lines = render_markdown("a **bold** and *ital*", 80, Color::Ansi(10));
        let all = flat(&lines);
        assert!(all.contains("bold"));
        assert!(all.contains("ital"));
        assert!(
            lines[0]
                .spans
                .iter()
                .any(|s| s.style.mods.contains(Modifiers::BOLD))
        );
        assert!(
            lines[0]
                .spans
                .iter()
                .any(|s| s.style.mods.contains(Modifiers::ITALIC))
        );
    }

    #[test]
    fn strike_and_code() {
        let lines = render_markdown("~~gone~~ see `x`", 80, Color::Ansi(10));
        let all = flat(&lines);
        assert!(all.contains("gone"));
        assert!(all.contains("x"));
        assert!(
            lines[0]
                .spans
                .iter()
                .any(|s| s.style.mods.contains(Modifiers::STRIKETHROUGH))
        );
    }

    #[test]
    fn link_shows_url() {
        let lines = render_markdown("click [here](https://example.com)", 80, Color::Ansi(10));
        let all = flat(&lines);
        assert!(all.contains("here"));
        assert!(all.contains("https://example.com"));
    }

    #[test]
    fn lists_and_tasks() {
        let text = "- buy milk\n- [ ] todo\n- [x] done\n1. first\n2. second";
        let lines = render_markdown(text, 80, Color::Ansi(12));
        let all = flat(&lines);
        assert!(all.contains("•"));
        assert!(all.contains("✗"));
        assert!(all.contains("✓"));
        assert!(all.contains("first"));
        assert!(all.contains("second"));
    }

    #[test]
    fn blockquote_and_hr() {
        let lines = render_markdown("> quoted\n---", 80, Color::Ansi(10));
        assert!(flat(&lines).contains("▌"));
        assert!(flat(&lines).contains("quoted"));
        assert!(flat(&lines).contains("━"));
    }

    #[test]
    fn code_block_is_boxed() {
        let lines = render_markdown("```rust\nfn main() {}\n```", 40, Color::Ansi(10));
        let all = flat(&lines);
        assert!(all.contains("rust"));
        assert!(all.contains("fn main() {}"));
        assert!(all.contains("│"));
        assert!(all.contains("╰─"));
    }

    #[test]
    fn long_text_wraps() {
        let text = "one two three four five six seven eight nine ten";
        let lines = render_markdown(text, 20, Color::Ansi(10));
        assert!(lines.len() > 1);
        let all = flat(&lines);
        assert!(all.contains("one"));
        assert!(all.contains("ten"));
    }

    #[test]
    fn wide_content_truncates_safely() {
        // Multibyte content must not panic on byte slicing.
        let text = format!("```\n{}\n```", "éü中文".repeat(40));
        let lines = render_markdown(&text, 20, Color::Ansi(10));
        for line in &lines {
            assert!(line.width() <= 25);
        }
    }
}
