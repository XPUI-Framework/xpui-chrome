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

    // Char boundaries, never byte offsets: a multi-byte character cut in half
    // is not a `&str`, and slicing one panics.
    //
    // Found by halving rather than by walking. Each probe is a `text_width`,
    // which on a backend across an FFI boundary is a call into C++ — per
    // truncated label, per row, per frame. Walking is O(n) probes; this is
    // O(log n), and both are exact.
    let boundaries: alloc::vec::Vec<usize> = text
        .char_indices()
        .map(|(index, _)| index)
        .chain(core::iter::once(text.len()))
        .collect();

    let mut low = 0;
    let mut high = boundaries.len() - 1;
    while low < high {
        // Biased upward, so the loop cannot stall on `low == high - 1`.
        let mid = low + (high - low).div_ceil(2);
        if font.text_width(&text[..boundaries[mid]]) <= room {
            low = mid;
        } else {
            high = mid - 1;
        }
    }

    let mut buffer = alloc::string::String::from(&text[..boundaries[low]]);
    buffer.push('…');
    Renderer::draw_text(xpui::Point::new(x, y), &buffer, font.id(), font.style());
}
