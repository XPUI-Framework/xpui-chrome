//! Placing a line of text inside a band, and cutting one that does not fit.

use xpui::{Font, Rect, Renderer};

/// Vertically centres a line of `font` inside `band`.
///
/// Text is drawn from its top-left, so a label centred by eye against a band's
/// midpoint sits half a line too low.
pub(super) fn centred_y(band: Rect, font: Font) -> i32 {
    band.y() + (band.height() - font.line_height()).max(0) / 2
}

/// Draws `text` clipped to `width`, cutting whole characters and appending an
/// ellipsis rather than letting it run into whatever is beside it.
pub(super) fn draw_truncated(x: i32, y: i32, text: &str, font: Font, width: i32) {
    if width <= 0 || !font.is_available() {
        return;
    }
    if font.text_width(text) <= width {
        Renderer::draw_text(xpui::Point::new(x, y), text, font.id(), font.style());
        return;
    }

    const ELLIPSIS: &str = "…";
    let room = width - font.text_width(ELLIPSIS);
    if room < 0 {
        // Not even the ellipsis fits. Drawing a partial glyph is worse than
        // drawing nothing, because it reads as a rendering fault.
        return;
    }

    // The longest prefix that fits, found by halving rather than by walking.
    // Each probe is a `text_width`, which on a backend across an FFI boundary
    // is a call into C++ — per truncated label, per row, per frame. Walking is
    // O(n) probes; this is O(log n), and both are exact.
    //
    // `low` and `high` are byte offsets kept on char boundaries: a multi-byte
    // character cut in half is not a `&str`, and slicing one panics. Snapping
    // a midpoint costs at most three steps and no list of boundaries.
    let mut low = 0;
    let mut fitted = 0;
    let mut high = text.len();
    while low < high {
        // Biased upward, so the loop cannot stall on `low == high - 1`.
        let mut mid = low + (high - low).div_ceil(2);
        while !text.is_char_boundary(mid) {
            mid += 1;
        }
        let measured = font.text_width(&text[..mid]);
        if measured <= room {
            low = mid;
            fitted = measured;
        } else {
            high = mid - 1;
            while !text.is_char_boundary(high) {
                high -= 1;
            }
        }
    }

    // Two runs, not one joined string: joining would allocate on every paint.
    let prefix = &text[..low];
    if !prefix.is_empty() {
        Renderer::draw_text(xpui::Point::new(x, y), prefix, font.id(), font.style());
    }
    Renderer::draw_text(
        xpui::Point::new(x + fitted, y),
        ELLIPSIS,
        font.id(),
        font.style(),
    );
}
