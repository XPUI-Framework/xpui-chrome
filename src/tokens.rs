//! The numbers everything here paints with.
//!
//! A backend that wants a different look changes these rather than the drawing
//! code. They are the same values `xpui`'s [`ThemeMetric`] asks for, in one
//! struct, so a backend can hand over a modified copy without implementing a
//! seventeen-arm match of its own.

use xpui::host::ThemeMetric;
use xpui::{Font, Renderer};

/// Geometry for the components in this crate.
///
/// Every field is a pixel count. The defaults suit a portrait e-ink panel of
/// roughly 480x800 at 1 bit; they scale by being asked for rather than
/// hardcoded at call sites, so a smaller panel needs a smaller `Tokens` and no
/// other change.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Tokens {
    /// Gap above the header band.
    pub top_padding: i32,
    pub header_height: i32,
    /// The standard gap between stacked elements.
    pub vertical_spacing: i32,
    /// The small step, for space *within* a group. Must stay smaller than
    /// `vertical_spacing` or a heading reads as belonging to what sits above.
    pub spacing_small: i32,
    /// Height reserved at the bottom for button hints.
    pub button_hints_height: i32,
    pub content_side_padding: i32,

    pub list_row_height: i32,
    pub list_row_height_with_subtitle: i32,
    /// Space left between one row and the next.
    pub list_row_gap: i32,
    /// Width of the bar marking the selected row. Selection cannot be drawn by
    /// inverting the row: [`Canvas::draw_text`](xpui::host::Canvas::draw_text)
    /// always paints ink, so ink-on-ink would erase the label.
    pub selection_marker_width: i32,

    /// Height of the band a sub-header needs — the heading's own line, not a
    /// list row.
    ///
    /// A *minimum*: the answer is this or the small font's line height,
    /// whichever is larger. [`Fonts`](crate) are swappable, and a 20px face in
    /// a 17px band overprints the heading's own rule and spills into the
    /// content below.
    pub sub_header_height: i32,

    pub progress_bar_height: i32,
    /// Smallest comfortably tappable dimension.
    pub min_touch_size: i32,

    pub slider_knob_width: i32,
    pub slider_knob_height: i32,
    /// Padding the slider track is inset by at each end.
    pub slider_side_inset: i32,
    pub slider_track_height: i32,

    /// Width of the scroll indicator, and its inset from the panel edge.
    pub scrollbar_width: i32,
    pub scrollbar_inset: i32,

    /// Border thickness for a dialog.
    pub dialog_border: i32,
    pub dialog_padding: i32,
    /// Fraction of the panel width a dialog occupies, as a percentage.
    pub dialog_width_percent: i32,

    /// What the four button-hint slots say when a screen passes
    /// [`Hint::Standard`](xpui::host::Hint), in meaning order: back, confirm,
    /// previous, next.
    ///
    /// Here rather than hardcoded in the painter because "your own standard
    /// label for this slot" is precisely the part a product supplies — this
    /// crate has no idea what language its user reads. The defaults are
    /// English because something has to be.
    pub standard_hints: [&'static str; 4],
    /// How many keys the row along the bottom actually has.
    ///
    /// Four is a reader: Back, Confirm, and a pair that walks the list. Three
    /// is a badge, whose keys are labelled a, b and c — there is no key for
    /// Back, so Confirm takes the first slot and Back lives on a double press
    /// of it.
    ///
    /// Drawing four hints over three keys is worse than drawing none: it tells
    /// you to press something that is not there, and every label after the
    /// first sits over the wrong key.
    pub hint_slots: u8,
}

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
        hint_slots: 4,
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
        hint_slots: 4,
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
        hint_slots: 4,
    };

    /// The same chrome, sized for a board that asks for larger targets.
    ///
    /// `percent` is a board's UI scale: 100 leaves every number alone, 120
    /// turns a 40px row into 48. An integer percentage rather than a float
    /// because `Tokens` is `Eq` and every preset is a `const` — neither of
    /// which an `f32` allows — and because the device targets have no
    /// floating-point unit, so a multiply by 1.2 there is a soft-float call in
    /// code every layout pass runs.
    ///
    /// Everything counted in pixels scales. Two fields do not:
    /// [`dialog_width_percent`] is already a ratio of the panel, and scaling a
    /// ratio pushes the dialog past the edge it is measured against; and
    /// [`standard_hints`] are words.
    ///
    /// The same chrome, for a device whose bottom row has `keys` keys.
    ///
    /// Three is a badge: a, b and c, with no key of its own for Back.
    pub const fn with_hint_slots(&self, keys: u8) -> Tokens {
        let mut tokens = *self;
        tokens.hint_slots = keys;
        tokens
    }

    /// The same chrome with no band reserved for button hints.
    ///
    /// For a device whose Back and Confirm come from its touchscreen rather
    /// than from a row of keys along the bottom. The firmware's own themes
    /// return before drawing a single hint on those boards, for the same
    /// reason: a hint names a key, and naming one that is not there sends a
    /// person looking for it.
    ///
    /// The height is the reservation as well as the drawing, so the content
    /// gains the space rather than leaving a gap where the band would be.
    pub const fn without_button_hints(&self) -> Tokens {
        let mut tokens = *self;
        tokens.button_hints_height = 0;
        tokens
    }

    /// [`dialog_width_percent`]: Tokens::dialog_width_percent
    /// [`standard_hints`]: Tokens::standard_hints
    pub const fn scaled(&self, percent: u16) -> Tokens {
        Tokens {
            top_padding: scale(self.top_padding, percent),
            header_height: scale(self.header_height, percent),
            vertical_spacing: scale(self.vertical_spacing, percent),
            spacing_small: scale(self.spacing_small, percent),
            button_hints_height: scale(self.button_hints_height, percent),
            content_side_padding: scale(self.content_side_padding, percent),

            list_row_height: scale(self.list_row_height, percent),
            list_row_height_with_subtitle: scale(self.list_row_height_with_subtitle, percent),
            list_row_gap: scale(self.list_row_gap, percent),
            selection_marker_width: scale(self.selection_marker_width, percent),

            sub_header_height: scale(self.sub_header_height, percent),

            progress_bar_height: scale(self.progress_bar_height, percent),
            min_touch_size: scale(self.min_touch_size, percent),

            slider_knob_width: scale(self.slider_knob_width, percent),
            slider_knob_height: scale(self.slider_knob_height, percent),
            slider_side_inset: scale(self.slider_side_inset, percent),
            slider_track_height: scale(self.slider_track_height, percent),

            scrollbar_width: scale(self.scrollbar_width, percent),
            scrollbar_inset: scale(self.scrollbar_inset, percent),

            dialog_border: scale(self.dialog_border, percent),
            dialog_padding: scale(self.dialog_padding, percent),
            dialog_width_percent: self.dialog_width_percent,

            standard_hints: self.standard_hints,
            hint_slots: self.hint_slots,
        }
    }

    /// The preset that fits a panel of this size.
    ///
    /// Chosen by height, because that is what the chrome eats: a header and a
    /// hint bar cost the same number of pixels on any width. The thresholds
    /// are where a preset stops leaving room for three list rows.
    ///
    /// A backend with an opinion supplies its own `Tokens` instead — this is a
    /// sensible default, not a rule.
    pub const fn for_panel(width: i32, height: i32) -> Tokens {
        let _ = width;
        if height <= 160 {
            Tokens::SMALL
        } else if height <= 320 {
            Tokens::COMPACT
        } else {
            Tokens::DEFAULT
        }
    }

    /// How many list rows fit in the content band of a panel this tall.
    ///
    /// The question every preset exists to answer, so it is worth being able
    /// to ask it directly — and worth testing, because the answer was zero.
    pub const fn list_rows_for(&self, panel_height: i32) -> i32 {
        let band = panel_height - self.button_hints_height - self.content_top_const();
        let stride = self.list_row_height + self.list_row_gap;
        if band <= 0 || stride <= 0 {
            return 0;
        }
        // The last row needs no gap after it.
        (band + self.list_row_gap) / stride
    }

    /// [`content_top`](Tokens::content_top) without needing an installed host,
    /// so [`list_rows_for`](Tokens::list_rows_for) can be a `const fn`.
    const fn content_top_const(&self) -> i32 {
        self.top_padding + self.header_height + self.vertical_spacing
    }

    /// First y below the header that content may use.
    pub fn content_top(&self) -> i32 {
        self.content_top_const()
    }

    /// First y occupied by the button hints; content must stay above it.
    ///
    /// Derived from the live panel height rather than stored, so one `Tokens`
    /// works on every panel a backend might be driving.
    pub fn content_bottom(&self) -> i32 {
        Renderer::screen_size().height - self.button_hints_height
    }

    /// Answers [`ThemeMetric`], which is how `xpui` asks for all of this.
    pub fn metric(&self, metric: ThemeMetric) -> i32 {
        match metric {
            ThemeMetric::TopPadding => self.top_padding,
            ThemeMetric::HeaderHeight => self.header_height,
            ThemeMetric::VerticalSpacing => self.vertical_spacing,
            ThemeMetric::ButtonHintsHeight => self.button_hints_height,
            ThemeMetric::ContentSidePadding => self.content_side_padding,
            ThemeMetric::ContentTop => self.content_top(),
            ThemeMetric::ContentBottom => self.content_bottom(),
            ThemeMetric::ListRowHeight => self.list_row_height,
            ThemeMetric::ListRowHeightWithSubtitle => self.list_row_height_with_subtitle,
            ThemeMetric::ListRowGap => self.list_row_gap,
            ThemeMetric::ProgressBarHeight => self.progress_bar_height,
            ThemeMetric::MinTouchSize => self.min_touch_size,
            ThemeMetric::SliderKnobWidth => self.slider_knob_width,
            ThemeMetric::SliderKnobHeight => self.slider_knob_height,
            ThemeMetric::SliderSideInset => self.slider_side_inset,
            // The larger of the token and what the font actually needs; see
            // the field's own note.
            ThemeMetric::SubHeaderHeight => {
                self.sub_header_height.max(Font::ui_small().line_height())
            }
            ThemeMetric::SpacingSmall => self.spacing_small,
        }
    }
}

impl Default for Tokens {
    fn default() -> Self {
        Tokens::DEFAULT
    }
}

/// One metric, scaled by a percentage and rounded to the nearest pixel.
///
/// A number that was worth a pixel stays worth one: a 1px rule scaled to 0 is
/// a rule that quietly stops being drawn, and a 0px gap is a list whose rows
/// touch.
const fn scale(value: i32, percent: u16) -> i32 {
    if value <= 0 {
        return value;
    }
    let scaled = (value * percent as i32 + 50) / 100;
    if scaled < 1 { 1 } else { scaled }
}
