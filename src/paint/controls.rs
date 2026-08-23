//! The components that show a value: hints, a progress bar, a slider and a
//! scroll indicator.

use xpui::host::{ControlState, Hint, HintWord};
use xpui::{Font, Rect, Renderer};

use super::text::{centred_y, draw_truncated};
use xpui::host::{KeyRow, RowKey};

use crate::labels::Labels;
use crate::metrics::Metrics;

/// The hints along the bottom, one per key the device says it has.
///
/// Three things, because three parties own them: the band's size is the
/// caller's [`Metrics`], the words are its [`Labels`], and which word sits
/// over which key is the device's [`KeyRow`].
pub fn draw_button_hints(
    metrics: &Metrics,
    labels: &Labels,
    keys: &KeyRow,
    back: &Hint,
    confirm: &Hint,
    previous: &Hint,
    next: &Hint,
) {
    // A board whose keys are not a row along the bottom reserves no band, and
    // there is nothing to draw in it. Labelling keys that are not there is
    // worse than labelling none: it tells you to press something the device
    // does not have.
    if metrics.button_hints_height <= 0 {
        return;
    }

    let size = Renderer::screen_size();
    let band = Rect::new(
        0,
        size.height - metrics.button_hints_height,
        size.width,
        metrics.button_hints_height,
    );
    let font = Font::ui_small();

    // A rule above, matching the one under the header.
    Renderer::draw_line(
        xpui::Point::new(0, band.y()),
        xpui::Point::new(size.width - 1, band.y()),
    );

    // The band is divided by the keys the board *has*, which is the length of
    // its row. A board with five keys and four things to say leaves the last
    // slot blank; dividing by four instead would slide every label off the key
    // it names.
    let slot_width = band.width() / keys.len().max(1) as i32;
    let y = centred_y(band, font);

    for (index, key) in keys.iter().enumerate() {
        // What the screen said about *this* key, not about this position. A row
        // that omits Back must not shift Back's hint onto Confirm's key, which
        // is the bug this indirection exists to prevent.
        let (hint, standard) = match key {
            RowKey::Back => (back, labels.standard_hints[0]),
            RowKey::Confirm => (confirm, labels.standard_hints[1]),
            RowKey::Previous => (previous, labels.standard_hints[2]),
            RowKey::Next => (next, labels.standard_hints[3]),
            RowKey::Unassigned => continue,
        };

        // `None` from `label()` means "a word of your own", and `word()` says
        // which: the slot's own by default, or one of the three a value
        // control's mode needs. `Some("")` means the screen asked for blank.
        let label = match hint.label() {
            None => match hint.word() {
                HintWord::Standard => standard,
                HintWord::Edit => labels.mode_hints[0],
                HintWord::Done => labels.mode_hints[1],
                HintWord::Cancel => labels.mode_hints[2],
            },
            Some("") => continue,
            Some(text) => text,
        };
        let area = Rect::new(
            band.x() + slot_width * index as i32,
            band.y(),
            slot_width,
            band.height(),
        );
        let text_width = font.text_width(label).min(area.width());
        draw_truncated(
            area.x() + (area.width() - text_width) / 2,
            y,
            label,
            font,
            area.width() - metrics.spacing_small,
        );
    }
}

/// A determinate bar: an outline, filled up to `current`.
pub fn draw_progress_bar(_metrics: &Metrics, rect: Rect, current: u32, total: u32) {
    Renderer::stroke_rect(rect);
    if total == 0 || rect.width() <= 2 {
        return;
    }

    let inner = rect.width() - 2;
    let filled = (inner as i64 * current.min(total) as i64 / total as i64) as i32;
    if filled > 0 {
        Renderer::fill_rect(
            Rect::new(rect.x() + 1, rect.y() + 1, filled, rect.height() - 2),
            true,
        );
    }
}

/// Where the knob sits for `value`, and the track it slides along.
///
/// Shared by the painter and by `xpui`'s own touch-to-value conversion, which
/// reads [`Metrics::slider_side_inset`] and `slider_knob_width` through
/// `ThemeMetric`. The two agree because they use the same two numbers.
fn slider_knob(metrics: &Metrics, rect: Rect, value: i32, max: i32) -> Rect {
    let inset = metrics.slider_side_inset;
    // `.max(1)`, matching `xpui::value_at`, not `.max(0)`. The two divide by
    // the same number to convert between a position and a value; if one
    // saturates at 0 and the other at 1 they disagree on a track too narrow
    // for its own knob, and the knob sits outside the widget while the value
    // snaps between the extremes.
    let travel = (rect.width() - inset * 2 - metrics.slider_knob_width).max(1);
    let along = if max > 0 {
        (travel as i64 * value.clamp(0, max) as i64 / max as i64) as i32
    } else {
        0
    };
    // Kept inside the widget even when the widget is narrower than the knob,
    // so it cannot overlap whatever sits beside it.
    let width = metrics.slider_knob_width.min(rect.width());
    Rect::new(
        (rect.x() + inset + along).min(rect.x() + rect.width() - width),
        rect.y() + (rect.height() - metrics.slider_knob_height).max(0) / 2,
        width,
        metrics.slider_knob_height.min(rect.height()),
    )
}

/// Track, fill and knob — and, when the keys are on it, the knob says so.
///
/// The three states have to be told apart at a glance on one bit of colour.
/// The knob carries the first difference, because it is the one part of the
/// control still painted paper: idle leaves it so, and focus fills it with a
/// dither. An open control adds one outline on top of that. A device with no
/// Left/Right pair changes what four keys mean when a value opens, and this is
/// the only thing that says so.
///
/// **Two other marks were built and rejected by eye.** A bar down the leading
/// edge lands where a stepper draws its `-` and reads as part of the glyph; a
/// box around a wide, mostly empty control is the heaviest thing on the panel.
/// And a second *shade* for the track is not available at all —
/// `fill_rect_dither`'s flag is a parity, not a density, so both of its values
/// are the same 50% checkerboard on opposite squares. That is what left the
/// knob as the only place with a change of shade still in it.
pub fn draw_slider(metrics: &Metrics, rect: Rect, value: i32, max: i32, state: ControlState) {
    if max <= 0 || rect.width() <= 0 || rect.height() <= 0 {
        // No range means no position to show. `Slider` guards this too, but
        // these functions are public and a backend may call them directly.
        return;
    }
    let inset = metrics.slider_side_inset;
    let track = Rect::new(
        rect.x() + inset,
        rect.y() + (rect.height() - metrics.slider_track_height).max(0) / 2,
        (rect.width() - inset * 2).max(0),
        metrics.slider_track_height.min(rect.height()),
    );

    // Dithered rather than solid: on one bit, a solid track is
    // indistinguishable from the filled part of it.
    Renderer::fill_rect_dither(track, true);

    let knob = slider_knob(metrics, rect, value, max);
    let filled = (knob.x() + knob.width() / 2 - track.x()).clamp(0, track.width());
    if filled > 0 {
        Renderer::fill_rect(
            Rect::new(track.x(), track.y(), filled, track.height()),
            true,
        );
    }

    // **The knob carries the selection**, for the reasons on `draw_slider`.
    // Filled before it is outlined either way, so the track does not show
    // through.
    if state == ControlState::Idle {
        Renderer::fill_rect(knob, false);
    } else {
        Renderer::fill_rect_dither(knob, true);
    }
    Renderer::stroke_rect(knob);

    // An open control adds one outline. The knob's shade already says the keys
    // are here; this says they are *moving the value*.
    if state == ControlState::Editing {
        Renderer::stroke_rect(rect);
    }
}

/// A thumb down the right edge, proportional to how much is showing.
///
/// Draws nothing when everything already fits — a full-height bar tells the
/// reader there is more to see when there is not.
pub fn draw_scroll_indicator(
    metrics: &Metrics,
    rect: Rect,
    content: i32,
    visible: i32,
    offset: i32,
) {
    if content <= visible || visible <= 0 || content <= 0 {
        return;
    }

    let x = rect.x() + rect.width() - metrics.scrollbar_width - metrics.scrollbar_inset;
    let track = Rect::new(x, rect.y(), metrics.scrollbar_width, rect.height());
    Renderer::fill_rect_dither(track, true);

    // A minimum so a very long page still shows something grabbable, and a
    // maximum so a short track does not get a thumb longer than itself. The
    // indicator is drawn *after* the scroll view lifts its clip, so a spill
    // lands on whatever sits below the viewport rather than being trimmed away.
    let thumb_height = ((track.height() as i64 * visible as i64 / content as i64)
        .max(metrics.min_touch_size as i64 / 4) as i32)
        .min(track.height());
    let travel = (track.height() - thumb_height).max(0);
    let scrollable = (content - visible).max(1);
    let along = (travel as i64 * offset.clamp(0, scrollable) as i64 / scrollable as i64) as i32;

    Renderer::fill_rect(
        Rect::new(track.x(), track.y() + along, track.width(), thumb_height),
        true,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Metrics;

    const M: Metrics = Metrics::DEFAULT;

    /// The two ends, exactly, because a knob that stops short reads as a
    /// slider that cannot reach its own maximum.
    #[test]
    fn the_knob_reaches_both_ends() {
        let rect = Rect::new(16, 100, 448, 40);
        let left = slider_knob(&M, rect, 0, 100);
        let right = slider_knob(&M, rect, 100, 100);

        assert_eq!(
            left.x(),
            rect.x() + M.slider_side_inset,
            "hard left at zero"
        );
        assert_eq!(
            right.x() + right.width(),
            rect.x() + rect.width() - M.slider_side_inset,
            "hard right at the maximum"
        );
    }

    /// Monotonic: a larger value never moves the knob left. A rounding change
    /// that broke this would look like a slider that jitters backwards.
    #[test]
    fn the_knob_never_moves_backwards() {
        let rect = Rect::new(0, 0, 300, 40);
        let mut previous = i32::MIN;
        for value in 0..=100 {
            let x = slider_knob(&M, rect, value, 100).x();
            assert!(x >= previous, "value {value} moved the knob left");
            previous = x;
        }
    }

    /// A widget smaller than its own knob, in **both** directions.
    ///
    /// `travel` saturates at 1 rather than 0 — matching `xpui::value_at`,
    /// which divides by the same number. And the knob is clamped to the rect
    /// on each axis independently: the first version of this test used a rect
    /// 40 tall against a 22-tall knob, so the height clamp never bound and
    /// removing it passed. A slider in a short row would then paint a knob
    /// overhanging whatever sits above and below.
    #[test]
    fn a_widget_smaller_than_the_knob_keeps_the_knob_inside_it() {
        for rect in [
            Rect::new(10, 0, 8, 40),  // narrow
            Rect::new(10, 0, 200, 6), // short
            Rect::new(10, 0, 8, 6),   // both
        ] {
            for value in [0, 50, 100] {
                let knob = slider_knob(&M, rect, value, 100);
                assert!(
                    knob.x() >= rect.x(),
                    "left edge escaped, {rect:?} at {value}"
                );
                assert!(
                    knob.x() + knob.width() <= rect.x() + rect.width(),
                    "right edge escaped, {rect:?} at {value}"
                );
                assert!(
                    knob.y() >= rect.y(),
                    "top edge escaped, {rect:?} at {value}"
                );
                assert!(
                    knob.y() + knob.height() <= rect.y() + rect.height(),
                    "bottom edge escaped, {rect:?} at {value}"
                );
            }
        }
    }

    /// `max = 0` is a control with nothing to choose. It must not divide.
    #[test]
    fn a_zero_maximum_does_not_divide_by_it() {
        let knob = slider_knob(&M, Rect::new(0, 0, 200, 40), 5, 0);
        assert_eq!(knob.x(), M.slider_side_inset);
    }
}
