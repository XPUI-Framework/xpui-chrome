//! What each component draws, committed as a transcript.
//!
//! `tests/paint.rs` asserts *properties* — that the selected row has a marker,
//! that a blank key gets no word. Forty-one of them, each written by hand, and
//! between them they leave the thing most likely to change unasserted: **where
//! everything is**. Move the header's baseline two pixels and every one of the
//! forty-one still passes.
//!
//! A transcript catches that, because it is the whole ordered draw-call log
//! rather than a sampled claim about it.
//!
//! **Text, not pixels, and not for want of trying.** Pixels need a backend,
//! and every backend depends on this crate. Adding one back as a
//! dev-dependency is not rejected by cargo — dev-dependency cycles are legal —
//! but it resolves **two** copies of `xpui-chrome`: the local one under test,
//! and the published one the backend pulls for itself. The pixels would come
//! from the copy you did not change. That is worse than no golden, because it
//! is green for a reason nobody would look for.
//!
//! The log also carries what a picture cannot: call order, and the state a
//! control was drawn in. The pixels are proved downstream instead, where a
//! backend and a board meet.
//!
//! `xpui-gallery`'s seventy board captures are those pixels. This is the half
//! that can be proved here.
//!
//! ```bash
//! UPDATE_SNAPSHOTS=1 cargo test        # then read the diff
//! ```

use xpui::Rect;
use xpui::host::{ControlState, Hint, RowField};
use xpui::host::{KeyRow, RowKey};
use xpui::testing;
use xpui_chrome::{Labels, Metrics};

const LABELS: Labels = Labels::ENGLISH;
const KEYS: KeyRow = KeyRow::READER;

/// All three presets, because a metric is only interesting when something
/// reads it — and because each is what some board actually paints with.
/// `DEFAULT` is a 480x800 reader, `COMPACT` is the Tufty 2040's 320x240, and
/// `SMALL` is what a 296x128 badge needs. They disagree about every height in
/// the crate, and a transcript per preset is what makes a per-board regression
/// visible here rather than in a repository two hops downstream.
const PRESETS: [(&str, Metrics); 3] = [
    ("default", Metrics::DEFAULT),
    ("compact", Metrics::COMPACT),
    ("small", Metrics::SMALL),
];

fn start() {
    testing::install();
    testing::reset();
}

fn cells(index: usize, field: RowField) -> Option<&'static str> {
    let row = [
        ("Wi-Fi", None, Some("Off")),
        ("Storage", Some("3.1 GB free"), None),
        ("About", None, None),
    ];
    let (title, subtitle, value) = *row.get(index)?;
    match field {
        RowField::Title => Some(title),
        RowField::Subtitle => subtitle,
        RowField::Value => value,
    }
}

fn options(index: usize) -> Option<&'static str> {
    ["Never", "After 5 minutes", "After an hour"]
        .get(index)
        .copied()
}

#[test]
fn a_header_with_a_subtitle() {
    for (name, metrics) in PRESETS {
        start();
        xpui_chrome::draw_header(&metrics, Some("Settings"), Some("Wi-Fi"));
        testing::assert_snapshot(&format!("header_{name}"));

        // A title long enough to be **truncated**, under `DEFAULT`, by the
        // room the subtitle leaves. That is the only condition under which
        // `vertical_spacing` is observable: the painter subtracts it from
        // `room`, and `room` is the width the title is cut to. A title that
        // fits reads none of it, and passes with the subtraction removed.
        start();
        xpui_chrome::draw_header(
            &metrics,
            Some("Network, internet and connected device settings"),
            Some("Wi-Fi, Bluetooth and aeroplane mode"),
        );
        testing::assert_snapshot(&format!("header_crowded_{name}"));
    }
}

#[test]
fn a_sub_header_with_a_right_hand_word() {
    for (name, metrics) in PRESETS {
        start();
        xpui_chrome::draw_sub_header(&metrics, Rect::new(0, 40, 480, 24), "System", Some("3"));
        testing::assert_snapshot(&format!("sub_header_{name}"));
    }
}

#[test]
fn a_list_with_a_row_selected() {
    for (name, metrics) in PRESETS {
        start();
        xpui_chrome::draw_list(&metrics, Rect::new(0, 60, 480, 400), 3, 1, &cells);
        testing::assert_snapshot(&format!("list_{name}"));
    }
}

/// The same list with no subtitle anywhere, and it is not a duplicate.
///
/// `row_height` is uniform across a list: if **any** row carries a subtitle,
/// every row gets `list_row_height_with_subtitle`. So the case above never
/// reads `list_row_height` at all — changing that metric left its transcript
/// untouched, which is a test passing for a reason nobody chose. This is the
/// case that reads it.
#[test]
fn a_list_where_no_row_has_a_subtitle() {
    fn plain(index: usize, field: RowField) -> Option<&'static str> {
        let row = ["Wi-Fi", "Bluetooth", "About"];
        match field {
            RowField::Title => row.get(index).copied(),
            _ => None,
        }
    }
    for (name, metrics) in PRESETS {
        start();
        xpui_chrome::draw_list(&metrics, Rect::new(0, 60, 480, 400), 3, 1, &plain);
        testing::assert_snapshot(&format!("list_plain_{name}"));
    }
}

#[test]
fn an_option_popup() {
    for (name, metrics) in PRESETS {
        start();
        xpui_chrome::draw_option_popup(&metrics, "Sleep after", &options, 3, 2);
        testing::assert_snapshot(&format!("popup_{name}"));
    }
}

/// Every state a slider has, in one transcript — the three are meant to look
/// different from each other, and a transcript is where that is legible.
#[test]
fn a_slider_in_each_of_its_states() {
    for (name, metrics) in PRESETS {
        start();
        // One golden per state rather than three concatenated into one. The
        // three differ by a line or two, and run together they were a wall of
        // near-identical rects in which a reviewer could not see where one
        // state ended — which defeats the point of committing them to be read.
        for (state, label) in [
            (ControlState::Idle, "idle"),
            (ControlState::Focused, "focused"),
            (ControlState::Editing, "editing"),
        ] {
            start();
            xpui_chrome::draw_slider(&metrics, Rect::new(16, 100, 448, 40), 60, 100, state);
            testing::assert_snapshot(&format!("slider_{label}_{name}"));
        }
    }
}

/// The one component that reads nothing from `Metrics` — it draws entirely
/// from the rect it is handed. So the rect is sized the way a caller sizes it,
/// from `progress_bar_height`; otherwise every preset's transcript is the same
/// file, committed three times.
///
/// The three degenerate cases are here because each returns early on its own
/// line, and a bar that silently draws nothing is indistinguishable from one
/// that draws an empty outline.
#[test]
fn a_progress_bar() {
    for (name, metrics) in PRESETS {
        start();
        let full_width = Rect::new(16, 100, 448, metrics.progress_bar_height);
        xpui_chrome::draw_progress_bar(&metrics, full_width, 60, 100);
        xpui_chrome::draw_progress_bar(&metrics, full_width, 0, 100);
        xpui_chrome::draw_progress_bar(&metrics, full_width, 100, 100);
        // Nothing to be a fraction of, and a bar with no room for a fill.
        xpui_chrome::draw_progress_bar(&metrics, full_width, 5, 0);
        xpui_chrome::draw_progress_bar(
            &metrics,
            Rect::new(16, 200, 2, metrics.progress_bar_height),
            60,
            100,
        );
        testing::assert_snapshot(&format!("progress_{name}"));
    }
}

/// Content taller than the panel, and content that fits. The second draws
/// nothing at all, which is the behaviour worth pinning.
#[test]
fn a_scroll_indicator_with_and_without_something_to_show() {
    for (name, metrics) in PRESETS {
        start();
        let rect = Rect::new(470, 60, 8, 400);
        // A page three times the panel: an ordinary thumb.
        xpui_chrome::draw_scroll_indicator(&metrics, rect, 1200, 400, 200);
        // Content that fits. Draws nothing at all, which is the behaviour.
        xpui_chrome::draw_scroll_indicator(&metrics, rect, 300, 400, 0);
        // A hundred thousand pixels of it. The proportional thumb would be one
        // pixel, so the floor has to bind — without it an e-ink panel shows a
        // thumb nobody can see, and no other case here reaches that branch.
        xpui_chrome::draw_scroll_indicator(&metrics, rect, 100_000, 400, 50_000);
        testing::assert_snapshot(&format!("scroll_{name}"));
    }
}

/// Four slots, and a row that has only three keys — so the fourth word has
/// nowhere to go. Which word lands over which key is the fault this catches.
#[test]
fn button_hints_over_a_three_key_row() {
    const BADGE: KeyRow = KeyRow::new(&[RowKey::Back, RowKey::Confirm, RowKey::Unassigned]);
    for (name, metrics) in PRESETS {
        start();
        xpui_chrome::draw_button_hints(
            &metrics,
            &LABELS,
            &BADGE,
            &Hint::Standard,
            &Hint::text("Edit"),
            &Hint::Standard,
            &Hint::Standard,
        );
        testing::assert_snapshot(&format!("hints_badge_{name}"));

        start();
        xpui_chrome::draw_button_hints(
            &metrics,
            &LABELS,
            &KEYS,
            &Hint::Standard,
            &Hint::Standard,
            &Hint::Standard,
            &Hint::Standard,
        );
        testing::assert_snapshot(&format!("hints_reader_{name}"));
    }
}
