//! The three panel sizes this crate ships presets for.
//!
//! Separate from [`Tokens`] itself because they are different jobs: that file
//! says what a token set *is* and how one is derived from another, and this one
//! is the measured answer for three shapes of glass. A fourth panel is a fourth
//! constant here and nothing else.

use crate::row::READER_ROW;
use crate::tokens::Tokens;

impl Tokens {
    /// The defaults every function here uses when a backend supplies nothing.
    pub const DEFAULT: Tokens = Tokens {
        top_padding: 8,
        header_height: 40,
        vertical_spacing: 12,
        spacing_small: 4,
        button_hints_height: 40,
        content_side_padding: 16,

        list_row_height: 40,
        list_row_height_with_subtitle: 56,
        list_row_gap: 4,
        selection_marker_width: 4,

        sub_header_height: 17,

        progress_bar_height: 6,
        min_touch_size: 44,

        slider_knob_width: 14,
        slider_knob_height: 22,
        slider_side_inset: 8,
        slider_track_height: 6,

        scrollbar_width: 4,
        scrollbar_inset: 2,

        dialog_border: 2,
        dialog_padding: 12,
        dialog_width_percent: 80,

        standard_hints: ["Back", "Select", "Up", "Down"],
        mode_hints: ["Edit", "Done", "Cancel"],
        row: READER_ROW,
    };

    /// For a panel of roughly 320x240 — a Tufty 2040, or any small colour LCD.
    ///
    /// Chrome that costs 96 pixels of a 800-pixel panel costs the same 96 of a
    /// 240-pixel one, which is 40% of it. Everything shrinks, and touch
    /// targets shrink furthest: a board with five buttons and no touchscreen
    /// does not need a 44-pixel finger target.
    pub const COMPACT: Tokens = Tokens {
        top_padding: 2,
        header_height: 24,
        vertical_spacing: 4,
        spacing_small: 3,
        button_hints_height: 22,
        content_side_padding: 8,

        list_row_height: 30,
        list_row_height_with_subtitle: 42,
        list_row_gap: 3,
        selection_marker_width: 3,

        sub_header_height: 14,

        progress_bar_height: 5,
        min_touch_size: 28,

        slider_knob_width: 10,
        slider_knob_height: 18,
        slider_side_inset: 6,
        slider_track_height: 5,

        scrollbar_width: 3,
        scrollbar_inset: 2,

        dialog_border: 1,
        dialog_padding: 6,
        dialog_width_percent: 88,

        standard_hints: ["Back", "OK", "Up", "Down"],
        mode_hints: ["Edit", "Done", "Cancel"],
        row: READER_ROW,
    };

    /// For a panel of roughly 296x128 — a Badger 2040, or any small e-ink strip.
    ///
    /// This is the size where the defaults stop working rather than merely
    /// looking cramped: a 40-pixel header, 40-pixel hint bar and 40-pixel rows
    /// leave 28 pixels of content, and a list refuses to paint a row that does
    /// not fit, so the screen comes back empty. Three rows is the target — a
    /// list of two with somewhere to scroll to.
    pub const SMALL: Tokens = Tokens {
        top_padding: 0,
        header_height: 18,
        // Still larger than `spacing_small`, even squeezed this far: a heading
        // spaced as widely within its group as between groups reads as
        // belonging to whatever sits above it.
        vertical_spacing: 4,
        spacing_small: 2,
        button_hints_height: 16,
        content_side_padding: 4,

        list_row_height: 24,
        list_row_height_with_subtitle: 34,
        list_row_gap: 2,
        selection_marker_width: 3,

        sub_header_height: 12,

        progress_bar_height: 4,
        min_touch_size: 24,

        slider_knob_width: 8,
        slider_knob_height: 14,
        slider_side_inset: 4,
        slider_track_height: 4,

        scrollbar_width: 3,
        scrollbar_inset: 1,

        dialog_border: 1,
        dialog_padding: 4,
        dialog_width_percent: 92,

        // Two of the five buttons on these boards are Up and Down; the labels
        // have to fit a quarter of a 296-pixel strip in a 6-pixel font.
        standard_hints: ["Back", "OK", "Up", "Dn"],
        mode_hints: ["Edit", "Done", "Undo"],
        row: READER_ROW,
    };
}
