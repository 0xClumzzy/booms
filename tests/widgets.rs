use std::time::Duration;

use booms::grid::{Cell, Grid};
use booms::layout::{Constraint, Rect};
use booms::style::{Color, Modifiers, Style};
use booms::widgets::{
    Asciify, Block, BorderType, Borders, Gauge, Input, List, MarkdownText, Panel, Scrollbar,
    Sparkline, SpinnerOrb, Tabs, Text, Widget,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[test]
fn text_writes_cells() {
    let mut grid = Grid::new(20, 3, Cell::blank());
    Text::new(["hello", "world"]).render(Rect::new(0, 0, 20, 3), &mut grid);
    assert_eq!(grid.get(0, 0).ch, 'h');
    assert_eq!(grid.get(4, 0).ch, 'o');
    assert_eq!(grid.get(4, 1).ch, 'd');
}

#[test]
fn text_wraps_words_at_width() {
    let mut grid = Grid::new(5, 3, Cell::blank());
    Text::new(["hello world foo"]).render(Rect::new(0, 0, 5, 3), &mut grid);
    assert_eq!(grid.get(0, 0).ch, 'h');
    assert_eq!(grid.get(0, 1).ch, 'w');
    assert_eq!(grid.get(0, 2).ch, 'f');
}

#[test]
fn block_draws_border_and_title() {
    let mut grid = Grid::new(10, 5, Cell::blank());
    Block::new()
        .title("editor")
        .borders(Borders::ALL)
        .render(Rect::new(0, 0, 10, 5), &mut grid);
    assert_eq!(grid.get(0, 0).ch, '┌');
    assert_eq!(grid.get(9, 0).ch, '┐');
    assert_eq!(grid.get(0, 4).ch, '└');
    assert_eq!(grid.get(9, 4).ch, '┘');
    assert_eq!(grid.get(1, 0).ch, 'e'); // title
    assert_eq!(grid.get(6, 0).ch, 'r'); // title end
    assert_eq!(grid.get(7, 0).ch, '─');
    assert_eq!(grid.get(0, 1).ch, '│');
    assert_eq!(grid.get(9, 1).ch, '│');
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

#[test]
fn input_edits_and_shows_cursor() {
    let mut input = Input::new();
    input.set_focus(true);
    input.on_key(key(KeyCode::Char('a')));
    input.on_key(key(KeyCode::Char('b')));
    assert_eq!(input.value(), "ab");
    assert_eq!(input.cursor_pos(Rect::new(0, 0, 10, 1)), Some((2, 0)));
    input.on_key(key(KeyCode::Backspace));
    assert_eq!(input.value(), "a");
    assert_eq!(input.cursor_pos(Rect::new(0, 0, 10, 1)), Some((1, 0)));
}

#[test]
fn list_moves_the_cursor() {
    let mut list = List::new(["a", "b", "c"]);
    assert_eq!(list.selected(), Some("a"));
    list.on_key(key(KeyCode::Down));
    list.on_key(key(KeyCode::Down));
    assert_eq!(list.selected(), Some("c"));
    list.on_key(key(KeyCode::Up));
    assert_eq!(list.selected(), Some("b"));
    list.on_key(key(KeyCode::Home));
    assert_eq!(list.selected(), Some("a"));
}

#[test]
fn focusability() {
    let input = Input::new();
    let list = List::new(["a"]);
    let text = Text::new(["a"]);
    assert!(input.is_focusable());
    assert!(list.is_focusable());
    assert!(!text.is_focusable());
}

#[test]
fn style_builders() {
    let s = Style::DEFAULT
        .fg(Color::Ansi(3))
        .bg(Color::Rgb(1, 2, 3))
        .mods(Modifiers::BOLD);
    assert_eq!(s.fg, Color::Ansi(3));
    assert_eq!(s.bg, Color::Rgb(1, 2, 3));
    assert!(s.has(Modifiers::BOLD));
    assert!(!s.has(Modifiers::REVERSE));
}

#[test]
fn layout_divides_constraints() {
    // Exact heights, then a flexible remainder.
    let sizes = booms::layout::layout(
        &[
            Constraint::Exact(3),
            Constraint::Flex(1),
            Constraint::Flex(1),
        ],
        10,
    );
    assert_eq!(sizes, vec![3, 3, 4]);
    let row = booms::layout::layout_row(
        Rect::new(0, 0, 10, 1),
        &[Constraint::Exact(2), Constraint::Flex(1)],
    );
    assert_eq!(row[0], Rect::new(0, 0, 2, 1));
    assert_eq!(row[1], Rect::new(2, 0, 8, 1));
    let col = booms::layout::layout_col(Rect::new(0, 0, 1, 5), &[Constraint::Exact(2)]);
    assert_eq!(col[0], Rect::new(0, 0, 1, 2));
}

#[test]
fn anim_smoothstep_and_lifecycle() {
    use booms::anim::{Anim, Easing};
    assert_eq!(Easing::SmoothStep.apply(0.0), 0.0);
    assert_eq!(Easing::SmoothStep.apply(1.0), 1.0);
    assert!((Easing::SmoothStep.apply(0.25) - 0.15625).abs() < 1e-4);
    let mut anim = Anim::new(0.0f32, 10.0, Duration::from_millis(60));
    while !anim.is_finished() {
        anim.update();
    }
    assert_eq!(*anim.value(), 10.0);
    anim.restart();
    assert!(!anim.is_finished());
    assert_eq!(*anim.value(), 0.0);
}

#[test]
fn spring_settles_to_target() {
    use booms::anim::Spring;
    let mut spring = Spring::new(0.0, 1.0, 200.0, 40.0, 1.0);
    assert!(spring.is_settled());
    assert!((spring.value() - 1.0).abs() < 0.05);
    spring.set_target(0.0);
    assert_eq!(spring.value(), 0.0);
    let mut ticks = 0;
    while !spring.is_settled() && ticks < 10_000 {
        spring.tick();
        ticks += 1;
    }
    assert!(spring.is_settled());
    assert!((spring.value() - 0.0).abs() < 0.05);
}

#[test]
fn scrollbar_thumb_moves_with_offset() {
    let mut grid = Grid::new(1, 10, Cell::blank());
    // Content overflows: 20 lines in a 5-line viewport.
    let bar = Scrollbar::new(20, 5, 0);
    assert!(bar.is_scrollable());
    assert_eq!(bar.max_offset(), 15);
    bar.render(Rect::new(0, 0, 1, 10), &mut grid);
    // At offset 0 the thumb sits in the top rows; a track line is below.
    let thumb_rows = (0..10).filter(|y| grid.get(0, *y).ch == '█').count();
    assert_eq!(thumb_rows, 3); // 5/20 of 10 rows, rounded
    assert!(grid.get(0, 0).ch == '█');
    assert!(grid.get(0, 9).ch == '│');

    // At max offset the thumb reaches the bottom rows.
    let mut grid = Grid::new(1, 10, Cell::blank());
    Scrollbar::new(20, 5, 15).render(Rect::new(0, 0, 1, 10), &mut grid);
    assert!(grid.get(0, 9).ch == '█');
    assert!(grid.get(0, 0).ch == '│');
}

#[test]
fn scrollbar_plain_track_when_content_fits() {
    let mut grid = Grid::new(1, 6, Cell::blank());
    let bar = Scrollbar::new(3, 5, 0);
    assert!(!bar.is_scrollable());
    bar.render(Rect::new(0, 0, 1, 6), &mut grid);
    for y in 0..6 {
        assert_eq!(grid.get(0, y).ch, '│');
    }
}

#[test]
fn markdown_text_renders_headings_and_bold() {
    let mut grid = Grid::new(20, 3, Cell::blank());
    MarkdownText::new("# Hello\n\nsome **bold**", Color::Ansi(14))
        .render(Rect::new(0, 0, 20, 3), &mut grid);
    // The heading lands on the first row, styled and underlined.
    assert_eq!(grid.get(0, 0).ch, 'H');
    let head = grid.get(0, 0).style;
    assert!(head.mods.contains(Modifiers::UNDERLINE));
    // The paragraph starts on the third row (blank line between).
    assert_eq!(grid.get(0, 2).ch, 's');
}

#[test]
fn markdown_text_wraps_at_width() {
    let mut grid = Grid::new(10, 6, Cell::blank());
    MarkdownText::new("one two three four", Color::Ansi(14))
        .render(Rect::new(0, 0, 10, 6), &mut grid);
    // The wrap is character-based: "one two th" fills the first row and
    // the remainder continues on the next row.
    let row0: String = (0..10).map(|x| grid.get(x, 0).ch).collect();
    assert_eq!(row0, "one two th");
    assert_eq!(grid.get(0, 1).ch, 'r');
}

#[test]
fn asciify_fills_area_and_animates() {
    let mut a = Grid::new(20, 8, Cell::blank());
    let scene = Asciify::new(booms::AsciifyEffect::Wave);
    scene.render(Rect::new(0, 0, 20, 8), &mut a);

    // Every cell is a ramp character, and the scene is not blank.
    let mut nonblank = 0;
    for y in 0..8 {
        for x in 0..20 {
            assert!(booms::asciify::FILL.contains(&a.get(x, y).ch));
            if a.get(x, y).ch != ' ' {
                nonblank += 1;
            }
        }
    }
    assert!(nonblank > 20, "scene left the grid too empty");

    // Ticking advances time, changing the frame.
    let mut scene = scene;
    scene.on_tick();
    let mut b = Grid::new(20, 8, Cell::blank());
    scene.render(Rect::new(0, 0, 20, 8), &mut b);
    assert!(a.get(5, 4) != b.get(5, 4) || a.get(3, 2) != b.get(3, 2));
}

#[test]
fn asciify_cycles_effects() {
    let mut scene = Asciify::new(booms::AsciifyEffect::Wave);
    assert_eq!(scene.effect().name(), "wave");
    scene.cycle_effect();
    assert_eq!(scene.effect().name(), "fog");
}

#[test]
fn panel_rounded_corners_and_plain_corners() {
    let mut g = Grid::new(8, 4, Cell::blank());
    Panel::new()
        .borders(BorderType::Rounded)
        .content(["x"])
        .render(Rect::new(0, 0, 8, 4), &mut g);
    assert_eq!(g.get(0, 0).ch, '╭');
    assert_eq!(g.get(7, 0).ch, '╮');
    assert_eq!(g.get(0, 3).ch, '╰');
    assert_eq!(g.get(7, 3).ch, '╯');

    let mut p = Grid::new(8, 4, Cell::blank());
    Panel::new()
        .borders(BorderType::Plain)
        .content(["x"])
        .render(Rect::new(0, 0, 8, 4), &mut p);
    assert_eq!(p.get(0, 0).ch, '┌');
    assert_eq!(p.get(7, 3).ch, '┘');
}

#[test]
fn panel_long_title_wraps_around_corner() {
    // A 6-wide panel: the title starts on the top edge and continues down
    // the right edge once the top edge is full.
    let mut g = Grid::new(6, 5, Cell::blank());
    Panel::new()
        .borders(BorderType::Rounded)
        .title("WRAPAROUND")
        .content(["hi"])
        .render(Rect::new(0, 0, 6, 5), &mut g);
    // top edge title glyphs (after the ╭ corner)
    assert_eq!(g.get(1, 0).ch, 'W');
    // the perimeter continues around the corner; some title glyph must land
    // on the right edge (x == width-1, y > 0)
    let right_has_title =
        (1..5).any(|y| matches!(g.get(5, y).ch, 'R' | 'A' | 'P' | 'O' | 'U' | 'N' | 'D'));
    assert!(right_has_title, "title should wrap around the corner");
}

#[test]
fn gauge_fills_filled_and_empty_cells() {
    let mut g = Grid::new(12, 1, Cell::blank());
    Gauge::new(0.5).render(Rect::new(0, 0, 12, 1), &mut g);
    // label " 50% " is 5 cols, so the bar occupies cols 5..12 (7 cells).
    // at ratio 0.5 ~= 4 filled + 3 empty; both glyph kinds must appear.
    let chars: Vec<char> = (5..12).map(|x| g.get(x, 0).ch).collect();
    assert!(chars.contains(&'█'), "expected filled cells: {chars:?}");
    assert!(
        chars.iter().any(|c| *c != '█' && *c != ' '),
        "expected empty track cells: {chars:?}"
    );
}

#[test]
fn sparkline_writes_bar_cells() {
    let mut g = Grid::new(8, 1, Cell::blank());
    Sparkline::new([1u64, 5, 3, 8]).render(Rect::new(0, 0, 8, 1), &mut g);
    let bars: Vec<char> = (0..8).map(|x| g.get(x, 0).ch).collect();
    assert!(
        bars.iter().any(|c| "▁▂▃▄▅▆▇█".contains(*c)),
        "expected bar glyphs: {bars:?}"
    );
}

#[test]
fn tabs_selection_is_tracked() {
    let mut tabs = Tabs::new(["one", "two", "three"]).selected(1);
    assert_eq!(tabs.selection(), 1);
    tabs.select(2);
    assert_eq!(tabs.selection(), 2);
    let mut g = Grid::new(24, 1, Cell::blank());
    tabs.render(Rect::new(0, 0, 24, 1), &mut g);
    // each tab label is rendered as " name " with a `│` separator
    assert_eq!(g.get(1, 0).ch, 'o');
    assert_eq!(g.get(5, 0).ch, '│');
}

#[test]
fn spinner_orb_advances_on_tick() {
    let mut orb = SpinnerOrb::new();
    let first = orb.frame();
    for _ in 0..3 {
        orb.on_tick();
    }
    assert_ne!(
        orb.frame(),
        first,
        "spinner frame should advance after enough ticks"
    );
    let mut g = Grid::new(4, 1, Cell::blank());
    orb.render(Rect::new(0, 0, 4, 1), &mut g);
    assert_eq!(g.get(0, 0).ch, orb.frame());
}
