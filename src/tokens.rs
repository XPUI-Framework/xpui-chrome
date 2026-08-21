//! The numbers everything here paints with.
//!
//! A backend that wants a different look changes these rather than the drawing
//! code. They are the same values `xpui`'s [`ThemeMetric`] asks for, in one
//! struct, so a backend can hand over a modified copy without implementing a
//! seventeen-arm match of its own.

use xpui::host::ThemeMetric;
use xpui::{Font, Renderer};

use crate::row::RowKey;

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
    /// The words a value control's mode needs, which the four above have no
    /// room for: opening one, keeping what it reads, and putting it back.
    ///
    /// Separate because the four are chosen by *which key* a slot sits over and
    /// these are chosen by what the framework is doing — no key implies "Edit".
    /// Board-supplied for the same reason as the four: this crate has no idea
    /// what language its user reads.
    pub mode_hints: [&'static str; 3],
    /// What each key along the bottom edge means, left to right.
    ///
    /// Its length is how many keys the row has, so this answers both questions
    /// a hint bar asks — how many slots to divide the band into, and which
    /// word goes in each. They used to be one question, inferred from the
    /// count: three keys was taken to mean a badge with no key for Back. That
    /// held until a board arrived with three keys *and* a Back among them, and
    /// every label after the first sat over the wrong key.
    ///
    /// Naming a key the device does not have is worse than naming none — it
    /// sends a person looking for it — so a slot with nothing behind it is
    /// [`RowKey::Unassigned`] and stays blank.
    pub row: &'static [RowKey],
}

impl Tokens {
    /// The same chrome, for a device whose bottom row is not a reader's four.
    ///
    /// ```text
    /// // A badge: three keys, the last with nothing on it yet.
    /// Tokens::SMALL.with_row(&[RowKey::Back, RowKey::Confirm, RowKey::Unassigned])
    /// ```
    ///
    /// Giving that third key a job later is this one line, which is the point
    /// of the row being data rather than a rule inside the painter.
    pub const fn with_row(&self, row: &'static [RowKey]) -> Tokens {
        let mut tokens = *self;
        tokens.row = row;
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

    /// The same chrome, sized for a board that asks for larger targets.
    ///
    /// `percent` is a board's UI scale: 100 leaves every number alone, 120
    /// turns a 40px row into 48. An integer percentage rather than a float
    /// because `Tokens` is `Eq` and every preset is a `const` — neither of
    /// which an `f32` allows — and because the device targets have no
    /// floating-point unit, so a multiply by 1.2 there is a soft-float call in
    /// code every layout pass runs.
    ///
    /// Everything counted in pixels scales. Three fields do not:
    /// [`dialog_width_percent`] is already a ratio of the panel, and scaling a
    /// ratio pushes the dialog past the edge it is measured against; and
    /// [`standard_hints`] and [`mode_hints`] are words.
    ///
    /// [`dialog_width_percent`]: Tokens::dialog_width_percent
    /// [`standard_hints`]: Tokens::standard_hints
    /// [`mode_hints`]: Tokens::mode_hints
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
            mode_hints: self.mode_hints,
            row: self.row,
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
