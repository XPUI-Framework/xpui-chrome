//! What the components actually put on the panel.
//!
//! These paint through `xpui`'s fake host, which records every primitive, so
//! each assertion is about pixels-as-drawn rather than about intent.

use xpui::host::{Chrome, Hint, RowField};
use xpui::testing::{self, DrawOp, RectKind};
use xpui::{Rect, Renderer};
use xpui_chrome::Tokens;

const TOKENS: Tokens = Tokens::DEFAULT;

/// A backend that is nothing but the fake host, wearing this crate's `Chrome`.
///
/// It exists to prove the macro produces a real `Chrome` impl, and that the
/// impl reaches the installed canvas.
struct Plain;

xpui_chrome::plain_chrome! {
    for Plain,
    tokens: |_backend| &TOKENS,
    request_update: |_backend| {},
}

fn start() -> Plain {
    testing::install();
    testing::reset();
    Plain
}

/// One row's three fields.
type Cells = (&'static str, Option<&'static str>, Option<&'static str>);

/// Rows the tests draw, so each case says only what it varies.
///
/// `use<>` because the closure copies the cells out: without it, Rust 2024
/// captures the argument's lifetime and a `rows(&[..])` temporary cannot
/// outlive its own statement.
fn rows(cells: &[Cells]) -> impl Fn(usize, RowField) -> Option<&'static str> + use<> {
    let cells = cells.to_vec();
    move |index, field| {
        let (title, subtitle, value) = *cells.get(index)?;
        match field {
            RowField::Title => Some(title),
            RowField::Subtitle => subtitle,
            RowField::Value => value,
        }
    }
}

// -- the invariant that matters most ---------------------------------------

/// A dialog paints its rows and separately reports where they are for
/// hit-testing. If those two derivations drift, tapping row 3 selects row 2 —
/// and nothing about the screen looks wrong, so only a test catches it.
#[test]
fn every_dialog_row_is_painted_where_hit_testing_says_it_is() {
    let backend = start();
    let options = ["Serif", "Sans", "Mono", "Slab"];
    let get = |index: usize| options.get(index).copied();

    backend.draw_option_popup("Font", &get, options.len(), 1);

    // Every row's title, and the rect it was drawn at.
    let painted: Vec<(String, i32)> = testing::ops_log()
        .into_iter()
        .filter_map(|op| match op {
            DrawOp::Text { origin, text, .. } if options.contains(&text.as_str()) => {
                Some((text, origin.y))
            }
            _ => None,
        })
        .collect();

    assert_eq!(painted.len(), options.len(), "every option was drawn");

    // Where the label sits *inside* its own row. If paint and hit-test derive
    // the same geometry this offset is identical for every row; if they drift
    // it creeps, and it creeps by less than a row height — which is why
    // "is the text somewhere inside the rect" does not catch it.
    let offsets: Vec<i32> = painted
        .iter()
        .enumerate()
        .map(|(index, (label, drawn_y))| {
            let rect = backend
                .option_popup_row_rect("Font", &get, options.len(), index)
                .unwrap_or_else(|| panic!("no rect reported for row {index}"));
            assert!(
                *drawn_y >= rect.y() && *drawn_y < rect.y() + rect.height(),
                "row {index} ({label}) was painted at y={drawn_y}, outside {rect:?}"
            );
            drawn_y - rect.y()
        })
        .collect();

    assert!(
        offsets.windows(2).all(|pair| pair[0] == pair[1]),
        "the painted rows and the hit-test rects drift apart: \
         label offsets within their own row were {offsets:?}, which must all be equal"
    );
}

#[test]
fn a_dialog_reports_no_rect_past_its_last_row() {
    let backend = start();
    let get = |i: usize| ["A", "B"].get(i).copied();

    assert!(backend.option_popup_row_rect("T", &get, 2, 1).is_some());
    assert!(
        backend.option_popup_row_rect("T", &get, 2, 2).is_none(),
        "asking past the end must not invent a row"
    );
}

/// A dialog sits over content, so it has to clear its own background before
/// painting or the screen behind shows through its text.
#[test]
fn a_dialog_clears_its_own_background_first() {
    let backend = start();
    let get = |i: usize| ["A", "B"].get(i).copied();
    backend.draw_option_popup("Pick", &get, 2, 0);

    let first_rect = testing::ops_log().into_iter().find_map(|op| match op {
        DrawOp::Rect { rect, kind, black } => Some((rect, kind, black)),
        _ => None,
    });

    let (_, kind, black) = first_rect.expect("the dialog painted a rectangle");
    assert_eq!(kind, RectKind::Filled);
    assert!(!black, "the first thing a dialog paints is its background");
}

// -- lists -----------------------------------------------------------------

#[test]
fn a_list_draws_every_row_it_was_given() {
    let backend = start();
    let cells = rows(&[
        ("Wi-Fi", None, Some("Off")),
        ("Storage", Some("3.1 GB free"), None),
        ("About", None, None),
    ]);

    backend.draw_list(Rect::new(0, 0, 400, 400), 3, -1, &cells);

    let drawn: Vec<String> = testing::drawn_text()
        .into_iter()
        .map(|(_, _, text, _, _)| text)
        .collect();

    for expected in ["Wi-Fi", "Off", "Storage", "3.1 GB free", "About"] {
        assert!(
            drawn.contains(&expected.to_string()),
            "missing {expected} in {drawn:?}"
        );
    }
}

/// Selection cannot invert the row — text is always ink, so ink-on-ink would
/// erase the label. It is a marker bar plus an outline instead.
#[test]
fn the_selected_row_is_marked_without_hiding_its_label() {
    let backend = start();
    let cells = rows(&[("One", None, None), ("Two", None, None)]);

    backend.draw_list(Rect::new(0, 0, 400, 400), 2, 1, &cells);

    let filled: Vec<Rect> = testing::ops_log()
        .into_iter()
        .filter_map(|op| match op {
            DrawOp::Rect {
                rect,
                kind: RectKind::Filled,
                black: true,
            } => Some(rect),
            _ => None,
        })
        .collect();

    assert_eq!(filled.len(), 1, "exactly one row is marked: {filled:?}");
    assert_eq!(
        filled[0].width(),
        TOKENS.selection_marker_width,
        "the mark is a bar at the edge, not a fill over the whole row"
    );

    let drawn: Vec<String> = testing::drawn_text()
        .into_iter()
        .map(|(_, _, text, _, _)| text)
        .collect();
    assert!(
        drawn.contains(&"Two".to_string()),
        "and the selected row's own label survives"
    );
}

/// A row half off the bottom reads as a rendering fault. The list stops
/// instead, and the scroll indicator is what says there is more.
#[test]
fn a_list_stops_before_a_row_that_would_be_clipped() {
    let backend = start();
    let cells = rows(&[
        ("One", None, None),
        ("Two", None, None),
        ("Three", None, None),
    ]);

    // Room for two rows and most of a third.
    let stride = TOKENS.list_row_height + TOKENS.list_row_gap;
    backend.draw_list(Rect::new(0, 0, 400, stride * 2 + 10), 3, -1, &cells);

    let drawn: Vec<String> = testing::drawn_text()
        .into_iter()
        .map(|(_, _, text, _, _)| text)
        .collect();
    assert_eq!(
        drawn,
        vec!["One", "Two"],
        "the third row was cut, not clipped"
    );
}

/// A row with a subtitle gets the taller height, or the two lines collide.
#[test]
fn a_subtitle_row_uses_the_taller_row_height() {
    let backend = start();
    let with = rows(&[("A", Some("sub"), None), ("B", Some("sub"), None)]);
    backend.draw_list(Rect::new(0, 0, 400, 400), 2, -1, &with);

    let tall: Vec<i32> = testing::drawn_text()
        .into_iter()
        .filter(|(_, _, text, _, _)| text == "A" || text == "B")
        .map(|(_, y, _, _, _)| y)
        .collect();

    let gap = tall[1] - tall[0];
    assert_eq!(
        gap,
        TOKENS.list_row_height_with_subtitle + TOKENS.list_row_gap,
        "rows with subtitles are spaced by the taller stride"
    );
}

// -- the rest --------------------------------------------------------------

/// A full-height scrollbar tells the reader there is more to see when there is
/// not, so nothing is drawn when the content already fits.
#[test]
fn a_scroll_indicator_stays_away_when_everything_fits() {
    let backend = start();
    backend.draw_scroll_indicator(Rect::new(0, 0, 480, 700), 300, 700, 0);
    assert!(
        testing::ops_log().is_empty(),
        "content shorter than the window needs no indicator"
    );

    testing::reset();
    backend.draw_scroll_indicator(Rect::new(0, 0, 480, 700), 1400, 700, 0);
    assert!(
        !testing::ops_log().is_empty(),
        "and content taller than it does"
    );
}

/// The thumb tracks the offset, and reaches the bottom at the bottom.
#[test]
fn the_scroll_thumb_travels_with_the_offset() {
    let backend = start();
    let track = Rect::new(0, 0, 480, 700);

    let thumb_y = |offset: i32| {
        testing::reset();
        backend.draw_scroll_indicator(track, 1400, 700, offset);
        testing::ops_log()
            .into_iter()
            .filter_map(|op| match op {
                DrawOp::Rect {
                    rect,
                    kind: RectKind::Filled,
                    ..
                } => Some(rect.y()),
                _ => None,
            })
            .next_back()
            .expect("a thumb was drawn")
    };

    let top = thumb_y(0);
    let middle = thumb_y(350);
    let bottom = thumb_y(700);

    assert_eq!(top, 0, "at the top it sits at the top");
    assert!(
        top < middle && middle < bottom,
        "and it moves monotonically"
    );
    assert_eq!(
        bottom + 350,
        700,
        "at the bottom its lower edge meets the track's"
    );
}

/// `Hint::None` is the only one that blanks a slot. `Hint::Standard` means
/// "your own label", which is a label.
#[test]
fn hint_slots_distinguish_standard_from_blank() {
    let backend = start();
    backend.draw_button_hints(
        &Hint::Standard,
        &Hint::text("Save"),
        &Hint::None,
        &Hint::None,
    );

    let drawn: Vec<String> = testing::drawn_text()
        .into_iter()
        .map(|(_, _, text, _, _)| text)
        .collect();

    assert!(
        drawn.contains(&"Back".to_string()),
        "Standard drew its own label"
    );
    assert!(
        drawn.contains(&"Save".to_string()),
        "Text drew the screen's"
    );
    assert_eq!(drawn.len(), 2, "and the two blanked slots drew nothing");
}

#[test]
fn a_progress_bar_fills_in_proportion() {
    let backend = start();
    let bar = Rect::new(0, 0, 102, 10);

    let filled_width = |current: u32| {
        testing::reset();
        backend.draw_progress_bar(bar, current, 100);
        testing::ops_log()
            .into_iter()
            .find_map(|op| match op {
                DrawOp::Rect {
                    rect,
                    kind: RectKind::Filled,
                    ..
                } => Some(rect.width()),
                _ => None,
            })
            .unwrap_or(0)
    };

    assert_eq!(filled_width(0), 0, "empty draws no fill at all");
    assert_eq!(filled_width(50), 50);
    assert_eq!(filled_width(100), 100, "full fills the whole interior");
}

/// A bar that overran its total would paint outside its own outline.
#[test]
fn a_progress_bar_cannot_overrun_its_outline() {
    let backend = start();
    let bar = Rect::new(0, 0, 102, 10);
    backend.draw_progress_bar(bar, 500, 100);

    let fill = testing::ops_log()
        .into_iter()
        .find_map(|op| match op {
            DrawOp::Rect {
                rect,
                kind: RectKind::Filled,
                ..
            } => Some(rect),
            _ => None,
        })
        .expect("a fill was drawn");

    assert!(
        fill.x() + fill.width() <= bar.x() + bar.width(),
        "the fill stayed inside: {fill:?} in {bar:?}"
    );
}

/// Both ends of a slider must be reachable, and the knob must never leave the
/// track — that is what makes the value the user sees match the one stored.
#[test]
fn a_slider_knob_stays_on_its_track_at_both_ends() {
    let backend = start();
    let bounds = Rect::new(0, 0, 300, 44);

    let knob = |value: i32| {
        testing::reset();
        backend.draw_slider(bounds, value, 100);
        // The knob is the last stroked rect: track, fill, knob-clear, knob.
        testing::ops_log()
            .into_iter()
            .filter_map(|op| match op {
                DrawOp::Rect {
                    rect,
                    kind: RectKind::Stroked,
                    ..
                } => Some(rect),
                _ => None,
            })
            .next_back()
            .expect("a knob was drawn")
    };

    let low = knob(0);
    let high = knob(100);

    assert!(low.x() >= bounds.x(), "the knob starts inside the widget");
    assert!(
        high.x() + high.width() <= bounds.x() + bounds.width(),
        "and ends inside it: {high:?}"
    );
    assert!(low.x() < high.x(), "and moves right as the value rises");
}

/// The knob is cleared before it is outlined, or the dithered track shows
/// through it and the two read as one grey smear.
#[test]
fn a_slider_knob_is_opaque_over_its_track() {
    let backend = start();
    backend.draw_slider(Rect::new(0, 0, 300, 44), 50, 100);

    let kinds: Vec<RectKind> = testing::ops_log()
        .into_iter()
        .filter_map(|op| match op {
            DrawOp::Rect { kind, .. } => Some(kind),
            _ => None,
        })
        .collect();

    let last_two = &kinds[kinds.len() - 2..];
    assert_eq!(
        last_two,
        [RectKind::Filled, RectKind::Stroked],
        "the knob clears, then outlines: {kinds:?}"
    );
}

/// The header rule is what separates chrome from content on a panel with no
/// colour to do it with.
#[test]
fn a_header_draws_its_title_and_a_rule() {
    let backend = start();
    backend.draw_header(Some("Settings"), None);

    let drawn: Vec<String> = testing::drawn_text()
        .into_iter()
        .map(|(_, _, text, _, _)| text)
        .collect();
    assert_eq!(drawn, vec!["Settings"]);

    let lines = testing::ops_log()
        .into_iter()
        .filter(|op| matches!(op, DrawOp::Line { .. }))
        .count();
    assert_eq!(lines, 1, "and one rule beneath it");
}

/// Long text must be cut on a character boundary. Cutting on a byte boundary
/// inside a multi-byte character panics.
#[test]
fn truncation_cuts_multibyte_text_safely() {
    let backend = start();
    let long = "Прокрутка страницы очень длинный заголовок настроек";
    let cells = rows(&[(long, None, None)]);

    // Narrow enough to force truncation.
    backend.draw_list(Rect::new(0, 0, 80, 200), 1, -1, &cells);

    let drawn = testing::drawn_text();
    assert_eq!(drawn.len(), 1);
    let text = &drawn[0].2;
    assert!(text.ends_with('…'), "it was truncated: {text:?}");
    assert!(text.chars().count() < long.chars().count());
}

/// Every metric `xpui` can ask for has to come back, or a layout silently
/// measures against zero.
#[test]
fn every_theme_metric_is_answered() {
    use xpui::host::ThemeMetric::*;
    let backend = start();

    for metric in [
        TopPadding,
        HeaderHeight,
        VerticalSpacing,
        ButtonHintsHeight,
        ContentSidePadding,
        ContentTop,
        ContentBottom,
        ListRowHeight,
        ListRowHeightWithSubtitle,
        ListRowGap,
        ProgressBarHeight,
        MinTouchSize,
        SliderKnobWidth,
        SliderKnobHeight,
        SliderSideInset,
        SubHeaderHeight,
        SpacingSmall,
    ] {
        assert!(
            backend.metric(metric) > 0,
            "{metric:?} answered {}",
            backend.metric(metric)
        );
    }
}

/// The content band has to sit between the header and the hints, whatever the
/// panel height — it is derived from the live screen size, not stored.
#[test]
fn the_content_band_clears_the_chrome_at_both_ends() {
    let backend = start();
    let screen = Renderer::screen_size();

    let top = backend.metric(xpui::host::ThemeMetric::ContentTop);
    let bottom = backend.metric(xpui::host::ThemeMetric::ContentBottom);

    assert!(
        top >= TOKENS.top_padding + TOKENS.header_height,
        "content starts below the header band"
    );
    assert_eq!(
        bottom,
        screen.height - TOKENS.button_hints_height,
        "and ends above the hints"
    );
    assert!(bottom > top);
}

// -- the painter and the framework must agree ------------------------------

/// The framework turns a touch into a value with `value_at`, using
/// `SliderSideInset` and `SliderKnobWidth` from the theme. This crate paints
/// the knob using the same two numbers. If the two derivations drift, the knob
/// lags the finger — a fault that looks like sloppy input handling rather than
/// like arithmetic, so nobody goes looking for it here.
///
/// The round trip is the assertion: paint the knob for a value, read its
/// centre back through `value_at`, and get the value again.
#[test]
fn a_painted_knob_reads_back_as_the_value_it_was_painted_for() {
    let backend = start();
    let track = Rect::new(20, 0, 300, 44);

    // The last stroked rect is the knob: track, fill, knob-clear, knob.
    let knob_for = |value: i32| {
        testing::reset();
        backend.draw_slider(track, value, 100);
        testing::ops_log()
            .into_iter()
            .filter_map(|op| match op {
                DrawOp::Rect {
                    rect,
                    kind: RectKind::Stroked,
                    ..
                } => Some(rect),
                _ => None,
            })
            .next_back()
            .expect("a knob was drawn")
    };

    for value in [0, 1, 25, 50, 73, 99, 100] {
        let knob = knob_for(value);
        let centre = knob.x() + knob.width() / 2;
        let read_back = xpui::value_at(track, centre, 100);

        assert_eq!(
            read_back, value,
            "a knob painted for {value} sits at x={centre}, which reads back as {read_back}"
        );
    }
}

/// Both ends have to be reachable by touch, or the last percent of a setting
/// cannot be chosen with a finger at all.
#[test]
fn both_ends_of_a_slider_are_reachable_by_touch() {
    let backend = start();
    let track = Rect::new(0, 0, 300, 44);
    let _ = backend;

    let left = track.x() + TOKENS.slider_side_inset + TOKENS.selection_marker_width;
    let right = track.x() + track.width();

    assert_eq!(
        xpui::value_at(track, track.x() - 50, 100),
        0,
        "a touch left of the track reads as the minimum, not a negative"
    );
    assert_eq!(
        xpui::value_at(track, right + 50, 100),
        100,
        "and one past the right end reads as the maximum"
    );
    assert!(xpui::value_at(track, left, 100) < 100);
}

// -- what a review found, turned into tests --------------------------------

/// A long subtitle used to be positioned at `right - its own width`, which
/// goes negative, and then left the title with negative room so it was dropped
/// entirely. Both halves silent: a header showing only a subtitle, painted
/// partly off the left of the panel.
#[test]
fn a_long_subtitle_neither_escapes_the_panel_nor_eats_the_title() {
    let backend = start();
    let long = "x".repeat(120);
    backend.draw_header(Some("Settings"), Some(&long));

    let drawn = testing::drawn_text();
    assert!(
        drawn.iter().any(|(_, _, text, _, _)| text == "Settings"),
        "the title survived a long subtitle: {drawn:?}"
    );
    for (x, _, text, _, _) in &drawn {
        assert!(*x >= 0, "{text:?} was painted at x={x}, off the panel");
    }
}

/// The same fault in the sub-header's right label.
#[test]
fn a_long_right_label_neither_escapes_its_rect_nor_eats_the_heading() {
    let backend = start();
    let rect = Rect::new(16, 200, 100, 17);
    let long = "y".repeat(60);
    backend.draw_sub_header(rect, "Heading", Some(&long));

    let drawn = testing::drawn_text();
    assert!(
        drawn.iter().any(|(_, _, text, _, _)| text == "Heading"),
        "the heading survived: {drawn:?}"
    );
    for (x, _, text, _, _) in &drawn {
        assert!(
            *x >= rect.x(),
            "{text:?} was painted at x={x}, left of its own rect at {}",
            rect.x()
        );
    }
}

/// A dialog with more options than the panel is tall used to draw its frame
/// past the bottom edge and paint rows nobody could see or touch — while the
/// widget still gave every one of them a focus stop.
#[test]
fn a_dialog_never_grows_past_the_panel() {
    let backend = start();
    let options: Vec<String> = (0..40).map(|i| format!("Option {i}")).collect();
    let get = |i: usize| options.get(i).map(String::as_str);

    backend.draw_option_popup("Language", &get, options.len(), 0);

    let screen = Renderer::screen_size();
    for op in testing::ops_log() {
        if let DrawOp::Rect { rect, .. } = op {
            assert!(
                rect.y() >= 0 && rect.y() + rect.height() <= screen.height,
                "{rect:?} falls outside a {}x{} panel",
                screen.width,
                screen.height
            );
        }
    }

    // And what was not painted must not be touchable.
    let painted = testing::drawn_text().len();
    assert!(painted < options.len(), "the dialog was clamped");
    assert!(
        backend
            .option_popup_row_rect("Language", &get, options.len(), options.len() - 1)
            .is_none(),
        "a row past the clamp reports no rect, so nothing can tap it"
    );
}

/// The scroll thumb used to have a minimum height but no maximum, so on a
/// short track it spilled past the bottom — onto whatever sits below the
/// viewport, because the indicator is drawn after the clip is lifted.
#[test]
fn a_scroll_thumb_never_outgrows_its_track() {
    let backend = start();
    let track = Rect::new(0, 0, 100, 8);
    backend.draw_scroll_indicator(track, 1000, 8, 0);

    for op in testing::ops_log() {
        if let DrawOp::Rect { rect, .. } = op {
            assert!(
                rect.y() + rect.height() <= track.y() + track.height(),
                "{rect:?} spills past a track {track:?}"
            );
        }
    }
}

/// `metric` is a seventeen-arm match, which is exactly the shape a copy-paste
/// swap hides in. Asserting only "non-zero" would pass with every arm swapped.
#[test]
fn every_metric_returns_its_own_field() {
    use xpui::host::ThemeMetric::*;
    let backend = start();

    assert_eq!(backend.metric(TopPadding), TOKENS.top_padding);
    assert_eq!(backend.metric(HeaderHeight), TOKENS.header_height);
    assert_eq!(backend.metric(VerticalSpacing), TOKENS.vertical_spacing);
    assert_eq!(
        backend.metric(ButtonHintsHeight),
        TOKENS.button_hints_height
    );
    assert_eq!(
        backend.metric(ContentSidePadding),
        TOKENS.content_side_padding
    );
    assert_eq!(backend.metric(ContentTop), TOKENS.content_top());
    assert_eq!(backend.metric(ContentBottom), TOKENS.content_bottom());
    assert_eq!(backend.metric(ListRowHeight), TOKENS.list_row_height);
    assert_eq!(
        backend.metric(ListRowHeightWithSubtitle),
        TOKENS.list_row_height_with_subtitle
    );
    assert_eq!(backend.metric(ListRowGap), TOKENS.list_row_gap);
    assert_eq!(
        backend.metric(ProgressBarHeight),
        TOKENS.progress_bar_height
    );
    assert_eq!(backend.metric(MinTouchSize), TOKENS.min_touch_size);
    assert_eq!(backend.metric(SliderKnobWidth), TOKENS.slider_knob_width);
    assert_eq!(backend.metric(SliderKnobHeight), TOKENS.slider_knob_height);
    assert_eq!(backend.metric(SliderSideInset), TOKENS.slider_side_inset);
    assert_eq!(backend.metric(SpacingSmall), TOKENS.spacing_small);

    // The one that is not a plain field: a taller font wins over the token,
    // so a swapped face cannot overprint the heading's own rule.
    assert_eq!(
        backend.metric(SubHeaderHeight),
        TOKENS
            .sub_header_height
            .max(xpui::Font::ui_small().line_height())
    );
}

/// A list stops before a row that would be clipped; every row it *does* draw
/// has to land inside the rect it was given, in order.
#[test]
fn list_rows_are_painted_in_order_inside_their_rect() {
    let backend = start();
    let cells = rows(&[
        ("One", None, None),
        ("Two", None, None),
        ("Three", None, None),
    ]);
    let bounds = Rect::new(10, 20, 400, 400);
    backend.draw_list(bounds, 3, -1, &cells);

    let ys: Vec<i32> = testing::drawn_text()
        .into_iter()
        .map(|(_, y, _, _, _)| y)
        .collect();

    assert_eq!(ys.len(), 3);
    assert!(ys.windows(2).all(|p| p[0] < p[1]), "top to bottom: {ys:?}");
    for y in &ys {
        assert!(
            *y >= bounds.y() && *y < bounds.y() + bounds.height(),
            "a row at y={y} left {bounds:?}"
        );
    }
}

/// Truncated text has to fit the width it was given. Cutting on a character
/// boundary is necessary but not sufficient — the point is that it fits.
#[test]
fn truncated_text_fits_the_width_it_was_given() {
    let backend = start();
    let long = "A line far too long for the space it is being given here";

    for width in [40, 60, 120, 200] {
        testing::reset();
        backend.draw_sub_header(Rect::new(0, 0, width, 17), long, None);

        let drawn = testing::drawn_text();
        if let Some((_, _, text, _, _)) = drawn.first() {
            let painted = xpui::Font::ui_small().bold().text_width(text);
            assert!(
                painted <= width,
                "at width {width} it painted {painted}px of {text:?}"
            );
        }
    }
}

// -- panels smaller than a phone -------------------------------------------

/// The default tokens are sized for a 480x800 e-reader. On a Badger 2040's
/// 296x128 strip they leave a 28-pixel content band, and a list refuses to
/// paint a row that does not fit — so the screen comes back **empty**. This
/// is the arithmetic that catches it, and the reason `SMALL` exists.
#[test]
fn the_default_tokens_do_not_fit_a_small_panel() {
    assert_eq!(
        Tokens::DEFAULT.list_rows_for(128),
        0,
        "if this ever becomes non-zero the defaults changed, and `for_panel` \
         should be revisited rather than this test relaxed"
    );
}

/// What each preset is *for*, as a number. A preset that stops fitting its own
/// panel is a preset that has quietly stopped working.
#[test]
fn every_preset_fits_the_panel_it_is_named_for() {
    assert!(
        Tokens::SMALL.list_rows_for(128) >= 3,
        "Badger 2040: {} rows",
        Tokens::SMALL.list_rows_for(128)
    );
    assert!(
        Tokens::COMPACT.list_rows_for(240) >= 4,
        "Tufty 2040: {} rows",
        Tokens::COMPACT.list_rows_for(240)
    );
    assert!(
        Tokens::DEFAULT.list_rows_for(800) >= 12,
        "a 480x800 e-reader: {} rows",
        Tokens::DEFAULT.list_rows_for(800)
    );
}

/// `for_panel` has to pick a preset that actually fits, for every panel any of
/// these boards has — otherwise it is a lookup table that returns the wrong
/// answer confidently.
#[test]
fn for_panel_always_picks_something_that_fits() {
    for (name, width, height) in [
        ("Badger 2040", 296, 128),
        ("Tufty 2040", 320, 240),
        ("a 480x800 e-reader", 480, 800),
        ("a 800x480 e-reader on its side", 800, 480),
    ] {
        let tokens = Tokens::for_panel(width, height);
        let rows = tokens.list_rows_for(height);
        assert!(
            rows >= 3,
            "{name} ({width}x{height}) got a preset with room for {rows} rows"
        );
    }
}

/// Chrome must not eat the whole panel. Half is already generous on a strip
/// 128 pixels tall.
#[test]
fn chrome_never_costs_more_than_half_a_small_panel() {
    let tokens = Tokens::for_panel(296, 128);
    let chrome = tokens.content_top() + tokens.button_hints_height;
    assert!(
        chrome <= 64,
        "chrome takes {chrome} of 128 pixels, leaving {} for content",
        128 - chrome
    );
}

// -- scaling for a board that wants bigger targets --------------------------

/// A board that asks for no change must get none. `scaled(100)` runs over
/// every field, and rounding that is off by one somewhere would move chrome on
/// boards that never opted into anything.
#[test]
fn scaling_by_one_hundred_changes_nothing() {
    for (name, tokens) in [
        ("DEFAULT", Tokens::DEFAULT),
        ("COMPACT", Tokens::COMPACT),
        ("SMALL", Tokens::SMALL),
    ] {
        assert_eq!(tokens.scaled(100), tokens, "{name} moved at 100%");
    }
}

/// What the scale is for: a row, a header, a hint band and a touch target all
/// grow together. One of them left behind is a layout that no longer adds up.
#[test]
fn scaling_grows_every_band_together() {
    let plain = Tokens::DEFAULT;
    let scaled = plain.scaled(120);

    for (name, before, after) in [
        ("row", plain.list_row_height, scaled.list_row_height),
        ("header", plain.header_height, scaled.header_height),
        (
            "hints",
            plain.button_hints_height,
            scaled.button_hints_height,
        ),
        ("touch target", plain.min_touch_size, scaled.min_touch_size),
        (
            "side padding",
            plain.content_side_padding,
            scaled.content_side_padding,
        ),
    ] {
        assert!(
            after > before,
            "the {name} stayed at {before} while the rest of the chrome grew"
        );
    }

    assert_eq!(
        scaled.list_row_height, 48,
        "a 40px row at 120% is 48, which is the number the firmware's board \
         table describes"
    );
}

/// A ratio is not a pixel count. Scaling `dialog_width_percent` would push a
/// dialog past the panel it is measured against — 80% of the width becomes 96%
/// once, and over 100% on any board that ever wants more.
#[test]
fn scaling_leaves_what_is_not_a_pixel_alone() {
    let scaled = Tokens::DEFAULT.scaled(120);

    assert_eq!(
        scaled.dialog_width_percent,
        Tokens::DEFAULT.dialog_width_percent
    );
    assert_eq!(scaled.standard_hints, Tokens::DEFAULT.standard_hints);
}

/// A hairline is one pixel and cannot become none: a rule scaled to zero stops
/// being drawn, and a row gap of zero is a list whose rows touch.
#[test]
fn a_single_pixel_survives_being_scaled_down() {
    let tiny = Tokens::SMALL.scaled(10);

    assert!(
        tiny.scrollbar_inset >= 1 && tiny.dialog_border >= 1 && tiny.list_row_gap >= 1,
        "something rounded away to nothing: {tiny:?}"
    );
}

/// Every preset has to be internally coherent, whatever its size: the small
/// step smaller than the standard one, a taller row for a subtitle, a knob
/// that fits its own track.
#[test]
fn every_preset_is_internally_consistent() {
    for (name, tokens) in [
        ("DEFAULT", Tokens::DEFAULT),
        ("COMPACT", Tokens::COMPACT),
        ("SMALL", Tokens::SMALL),
        // And each of them as a touch board scales it. A preset that is
        // coherent at 100 and incoherent at 120 breaks on hardware, where
        // rounding decides whether the small step is still smaller than the
        // standard one.
        ("DEFAULT scaled", Tokens::DEFAULT.scaled(120)),
        ("COMPACT scaled", Tokens::COMPACT.scaled(120)),
        ("SMALL scaled", Tokens::SMALL.scaled(120)),
    ] {
        assert!(
            tokens.spacing_small < tokens.vertical_spacing,
            "{name}: a heading spaced as widely within its group as between \
             groups reads as belonging to the wrong one"
        );
        assert!(
            tokens.list_row_height_with_subtitle > tokens.list_row_height,
            "{name}: a two-line row needs more height than a one-line row"
        );
        assert!(
            tokens.slider_knob_width < tokens.min_touch_size,
            "{name}: a knob wider than the minimum touch target has nowhere to travel"
        );
        assert!(
            tokens.dialog_border * 2 < tokens.dialog_padding,
            "{name}: a border thicker than its own padding leaves no room for text"
        );
        assert!(
            tokens.dialog_width_percent > 50 && tokens.dialog_width_percent <= 100,
            "{name}: a dialog narrower than half the panel is unreadable"
        );
    }
}
