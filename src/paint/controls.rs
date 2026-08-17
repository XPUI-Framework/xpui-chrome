//! The components that show a value: hints, a progress bar, a slider and a
//! scroll indicator.

use xpui::host::Hint;
use xpui::{Font, Rect, Renderer};

use super::text::{centred_y, draw_truncated};
use crate::tokens::Tokens;

/// The hints along the bottom, one per key the board says it has.
pub fn draw_button_hints(
    tokens: &Tokens,
    back: &Hint,
    confirm: &Hint,
    previous: &Hint,
    next: &Hint,
) {
    // A board whose keys are not a row along the bottom reserves no band, and
    // there is nothing to draw in it. Labelling keys that are not there is
    // worse than labelling none: it tells you to press something the device
    // does not have.
    if tokens.button_hints_height <= 0 {
        return;
    }

    let size = Renderer::screen_size();
    let band = Rect::new(
        0,
        size.height - tokens.button_hints_height,
        size.width,
        tokens.button_hints_height,
    );
    let font = Font::ui_small();

    // A rule above, matching the one under the header.
    Renderer::draw_line(
        xpui::Point::new(0, band.y()),
        xpui::Point::new(size.width - 1, band.y()),
    );

    // A three-key board has no key for Back — it is a double press of the
    // first — so its slots start at Confirm and its labels start with them.
    let three = tokens.hint_slots == 3;
    let hints: &[&Hint] = if three {
        &[confirm, previous, next]
    } else {
        &[back, confirm, previous, next]
    };

    // The band is divided by the keys the board *has*, not by the labels there
    // are to write. A board with five keys and four things to say leaves the
    // last slot blank; dividing by four instead would slide every label off the
    // key it names.
    let slot_width = band.width() / tokens.hint_slots.max(1) as i32;
    let y = centred_y(band, font);

    for (index, hint) in hints.iter().enumerate().take(tokens.hint_slots as usize) {
        // `None` from `label()` means "your own standard label for this slot";
        // `Some("")` means the screen asked for it to be blank.
        let label = match hint.label() {
            None => tokens.standard_hints[if three { index + 1 } else { index }],
            Some("") => continue,
            Some(text) => text,
        };
        let slot = Rect::new(
            band.x() + slot_width * index as i32,
            band.y(),
            slot_width,
            band.height(),
        );
        let text_width = font.text_width(label).min(slot.width());
        draw_truncated(
            slot.x() + (slot.width() - text_width) / 2,
            y,
            label,
            font,
            slot.width() - tokens.spacing_small,
        );
    }
}

/// A determinate bar: an outline, filled up to `current`.
pub fn draw_progress_bar(_tokens: &Tokens, rect: Rect, current: u32, total: u32) {
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
/// reads [`Tokens::slider_side_inset`] and `slider_knob_width` through
/// `ThemeMetric`. The two agree because they use the same two numbers.
fn slider_knob(tokens: &Tokens, rect: Rect, value: i32, max: i32) -> Rect {
    let inset = tokens.slider_side_inset;
    // `.max(1)`, matching `xpui::value_at`, not `.max(0)`. The two divide by
    // the same number to convert between a position and a value; if one
    // saturates at 0 and the other at 1 they disagree on a track too narrow
    // for its own knob, and the knob sits outside the widget while the value
    // snaps between the extremes.
    let travel = (rect.width() - inset * 2 - tokens.slider_knob_width).max(1);
    let along = if max > 0 {
        (travel as i64 * value.clamp(0, max) as i64 / max as i64) as i32
    } else {
        0
    };
    // Kept inside the widget even when the widget is narrower than the knob,
    // so it cannot overlap whatever sits beside it.
    let width = tokens.slider_knob_width.min(rect.width());
    Rect::new(
        (rect.x() + inset + along).min(rect.x() + rect.width() - width),
        rect.y() + (rect.height() - tokens.slider_knob_height).max(0) / 2,
        width,
        tokens.slider_knob_height.min(rect.height()),
    )
}

/// Track, fill and knob.
pub fn draw_slider(tokens: &Tokens, rect: Rect, value: i32, max: i32) {
    if max <= 0 || rect.width() <= 0 || rect.height() <= 0 {
        // No range means no position to show. `Slider` guards this too, but
        // these functions are public and a backend may call them directly.
        return;
    }
    let inset = tokens.slider_side_inset;
    let track = Rect::new(
        rect.x() + inset,
        rect.y() + (rect.height() - tokens.slider_track_height).max(0) / 2,
        (rect.width() - inset * 2).max(0),
        tokens.slider_track_height.min(rect.height()),
    );

    // Dithered rather than solid: on one bit, a solid track is
    // indistinguishable from the filled part of it.
    Renderer::fill_rect_dither(track, true);

    let knob = slider_knob(tokens, rect, value, max);
    let filled = (knob.x() + knob.width() / 2 - track.x()).clamp(0, track.width());
    if filled > 0 {
        Renderer::fill_rect(
            Rect::new(track.x(), track.y(), filled, track.height()),
            true,
        );
    }

    // Cleared before it is outlined, so the track does not show through.
    Renderer::fill_rect(knob, false);
    Renderer::stroke_rect(knob);
}

/// A thumb down the right edge, proportional to how much is showing.
///
/// Draws nothing when everything already fits — a full-height bar tells the
/// reader there is more to see when there is not.
pub fn draw_scroll_indicator(tokens: &Tokens, rect: Rect, content: i32, visible: i32, offset: i32) {
    if content <= visible || visible <= 0 || content <= 0 {
        return;
    }

    let x = rect.x() + rect.width() - tokens.scrollbar_width - tokens.scrollbar_inset;
    let track = Rect::new(x, rect.y(), tokens.scrollbar_width, rect.height());
    Renderer::fill_rect_dither(track, true);

    // A minimum so a very long page still shows something grabbable, and a
    // maximum so a short track does not get a thumb longer than itself. The
    // indicator is drawn *after* the scroll view lifts its clip, so a spill
    // lands on whatever sits below the viewport rather than being trimmed away.
    let thumb_height = ((track.height() as i64 * visible as i64 / content as i64)
        .max(tokens.min_touch_size as i64 / 4) as i32)
        .min(track.height());
    let travel = (track.height() - thumb_height).max(0);
    let scrollable = (content - visible).max(1);
    let along = (travel as i64 * offset.clamp(0, scrollable) as i64 / scrollable as i64) as i32;

    Renderer::fill_rect(
        Rect::new(track.x(), track.y() + along, track.width(), thumb_height),
        true,
    );
}
