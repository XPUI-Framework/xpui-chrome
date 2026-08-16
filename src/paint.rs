//! The eight components, painted from primitives.

use xpui::host::{Hint, RowField};
use xpui::{Font, Rect, Renderer};

use crate::tokens::Tokens;

/// Vertically centres a line of `font` inside `band`.
///
/// Text is drawn from its top-left, so a label centred by eye against a band's
/// midpoint sits half a line too low.
fn centred_y(band: Rect, font: Font) -> i32 {
    band.y() + (band.height() - font.line_height()).max(0) / 2
}

/// Draws `text` clipped to `width`, cutting whole characters and appending an
/// ellipsis rather than letting it run into whatever is beside it.
fn draw_truncated(x: i32, y: i32, text: &str, font: Font, width: i32) {
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

/// The title band, with a rule under it.
pub fn draw_header(tokens: &Tokens, title: Option<&str>, subtitle: Option<&str>) {
    let width = Renderer::screen_size().width;
    let band = Rect::new(0, tokens.top_padding, width, tokens.header_height);
    let font = Font::ui().bold();
    let inset = tokens.content_side_padding;

    let mut room = band.width() - inset * 2;

    if let Some(subtitle) = subtitle {
        let small = Font::ui_small();
        // Clamped to half the band. Unclamped, a long subtitle is positioned
        // at `right - its own width`, which goes negative and paints off the
        // left edge — and then leaves `room` negative, so the *title* is
        // dropped entirely and the header comes back showing only the
        // subtitle. Both halves of that failure are silent.
        let allowance = ((band.width() - inset * 2) / 2).max(0);
        let subtitle_width = small.text_width(subtitle).min(allowance);
        draw_truncated(
            band.x() + band.width() - inset - subtitle_width,
            centred_y(band, small),
            subtitle,
            small,
            subtitle_width,
        );
        room -= subtitle_width + tokens.vertical_spacing;
    }

    if let Some(title) = title {
        draw_truncated(band.x() + inset, centred_y(band, font), title, font, room);
    }

    // The rule is what separates the header from content on a panel with no
    // colour to separate them with.
    let rule = band.y() + band.height();
    Renderer::draw_line(xpui::Point::new(0, rule), xpui::Point::new(width - 1, rule));
}

/// A group heading: the label, and a rule across the rest of the line.
pub fn draw_sub_header(tokens: &Tokens, rect: Rect, label: &str, right: Option<&str>) {
    let font = Font::ui_small().bold();
    let mut room = rect.width();

    if let Some(right) = right {
        let plain = Font::ui_small();
        // Clamped for the same reason the header's subtitle is: unclamped it
        // is placed at `right - its own width` and walks off the left of its
        // own rect, taking the heading with it.
        let allowance = (rect.width() / 2).max(0);
        let width = plain.text_width(right).min(allowance);
        draw_truncated(
            rect.x() + rect.width() - width,
            rect.y(),
            right,
            plain,
            width,
        );
        room -= width + tokens.spacing_small;
    }

    draw_truncated(rect.x(), rect.y(), label, font, room);

    // A hairline along the baseline of the heading, starting after the label,
    // which is what makes a heading read as a divider rather than a row.
    let label_width = font.text_width(label).min(room.max(0));
    let start = rect.x() + label_width + tokens.spacing_small;
    // `- 1`: a rect's right edge is exclusive, so `x + width` is the first
    // pixel *outside* it.
    let end = rect.x() + room - 1;
    if end > start {
        let y = rect.y() + rect.height() - 1;
        Renderer::draw_line(xpui::Point::new(start, y), xpui::Point::new(end, y));
    }
}

/// The four hints along the bottom, one per quarter.
pub fn draw_button_hints(
    tokens: &Tokens,
    back: &Hint,
    confirm: &Hint,
    previous: &Hint,
    next: &Hint,
) {
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

    let slot_width = band.width() / 4;
    let y = centred_y(band, font);

    for (index, hint) in [back, confirm, previous, next].into_iter().enumerate() {
        // `None` from `label()` means "your own standard label for this slot";
        // `Some("")` means the screen asked for it to be blank.
        let label = match hint.label() {
            None => tokens.standard_hints[index],
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

/// Height of one row, chosen by whether any row carries a subtitle — the same
/// rule `xpui`'s `List` measures with.
/// How many of `rows` will actually be painted into `rect`.
///
/// `draw_list` stops before a row that does not fit, so the space it leaves can
/// be most of a row — 224 pixels on a 480x800 panel. A caller that needs to
/// know where the list really ends has to ask rather than divide.
pub fn rows_that_fit<'a>(
    tokens: &Tokens,
    rect: Rect,
    rows: usize,
    row: &dyn Fn(usize, RowField) -> Option<&'a str>,
) -> usize {
    let height = row_height(tokens, rows, row);
    let stride = height + tokens.list_row_gap;
    (0..rows)
        .take_while(|index| {
            let top = rect.y() + stride * *index as i32;
            top + height <= rect.y() + rect.height()
        })
        .count()
}

/// The height every row in this list gets.
pub fn row_height<'a>(
    tokens: &Tokens,
    rows: usize,
    row: &dyn Fn(usize, RowField) -> Option<&'a str>,
) -> i32 {
    let has_subtitle = (0..rows).any(|index| row(index, RowField::Subtitle).is_some());
    if has_subtitle {
        tokens.list_row_height_with_subtitle
    } else {
        tokens.list_row_height
    }
}

/// The themed list. Only rows that fit entirely are drawn.
pub fn draw_list<'a>(
    tokens: &Tokens,
    rect: Rect,
    rows: usize,
    selected: i32,
    row: &dyn Fn(usize, RowField) -> Option<&'a str>,
) {
    let height = row_height(tokens, rows, row);
    let stride = height + tokens.list_row_gap;

    for index in 0..rows {
        let top = rect.y() + stride * index as i32;
        // A partially visible row reads as a rendering fault rather than as
        // "there is more below", which is what the scroll indicator is for.
        if top + height > rect.y() + rect.height() {
            break;
        }
        let bounds = Rect::new(rect.x(), top, rect.width(), height);
        draw_row(tokens, bounds, index as i32 == selected, index, row);
    }
}

fn draw_row<'a>(
    tokens: &Tokens,
    bounds: Rect,
    selected: bool,
    index: usize,
    row: &dyn Fn(usize, RowField) -> Option<&'a str>,
) {
    let marker = tokens.selection_marker_width;

    if selected {
        // A bar down the leading edge, plus an outline. Not an inverted fill:
        // text is always ink, so ink-on-ink would erase the label.
        Renderer::fill_rect(
            Rect::new(bounds.x(), bounds.y(), marker, bounds.height()),
            true,
        );
        Renderer::stroke_rect(bounds);
    }

    let text_x = bounds.x() + marker + tokens.spacing_small * 2;
    let mut room = bounds.width() - (text_x - bounds.x()) - tokens.spacing_small * 2;

    let title_font = Font::ui();
    let subtitle = row(index, RowField::Subtitle);

    if let Some(value) = row(index, RowField::Value) {
        let value_font = Font::ui();
        let width = value_font.text_width(value).min(room.max(0));
        draw_truncated(
            bounds.x() + bounds.width() - tokens.spacing_small * 2 - width,
            centred_y(bounds, value_font),
            value,
            value_font,
            width,
        );
        room -= width + tokens.spacing_small * 2;
    }

    let Some(title) = row(index, RowField::Title) else {
        return;
    };

    match subtitle {
        // Two lines: the pair is centred as a block, not each line separately.
        Some(subtitle) => {
            let small = Font::ui_small();
            let block = title_font.line_height() + small.line_height();
            let top = bounds.y() + (bounds.height() - block).max(0) / 2;
            draw_truncated(text_x, top, title, title_font, room);
            draw_truncated(
                text_x,
                top + title_font.line_height(),
                subtitle,
                small,
                room,
            );
        }
        None => draw_truncated(
            text_x,
            centred_y(bounds, title_font),
            title,
            title_font,
            room,
        ),
    }
}

/// Where a dialog and its rows land.
///
/// Both [`draw_option_popup`] and [`option_popup_row_rect`] go through this.
/// Deriving the geometry twice is how a dialog ends up painting row 3 where
/// row 2 responds to a touch, and nothing about that failure looks wrong until
/// somebody taps it.
pub struct PopupLayout {
    pub frame: Rect,
    pub first_row: Rect,
    pub stride: i32,
    /// How many rows actually fit on the panel.
    ///
    /// A dialog with more options than the screen is tall is drawn as tall as
    /// it fits and no taller. Without this the frame runs off the bottom and
    /// the rows past the edge are painted where nobody can see or touch them —
    /// while `Modal` still gives every one of them a focus stop, so the
    /// highlight walks off the panel. A twenty-entry language picker reaches
    /// this on a 480x800 panel.
    pub visible: usize,
}

pub fn popup_layout(tokens: &Tokens, title: &str, count: usize) -> PopupLayout {
    let screen = Renderer::screen_size();
    let title_font = Font::ui().bold();
    let border = tokens.dialog_border;
    let padding = tokens.dialog_padding;

    let title_band = if title.is_empty() {
        0
    } else {
        title_font.line_height() + padding
    };

    let row_height = tokens.list_row_height.max(1);
    let chrome = border * 2 + padding * 2 + title_band;
    // Never taller than the panel, and never fewer than one row — a dialog
    // showing nothing is not an improvement on one showing too much.
    let room = (screen.height - chrome).max(row_height);
    let visible = count.min((room / row_height).max(1) as usize);

    let body = row_height * visible as i32;
    let height = chrome + body;
    let width = (screen.width * tokens.dialog_width_percent / 100).min(screen.width);

    let frame = Rect::new(
        (screen.width - width) / 2,
        (screen.height - height).max(0) / 2,
        width,
        height,
    );

    let inner = border + padding;
    PopupLayout {
        frame,
        first_row: Rect::new(
            frame.x() + inner,
            frame.y() + inner + title_band,
            frame.width() - inner * 2,
            row_height,
        ),
        stride: row_height,
        visible,
    }
}

/// A centred dialog: a cleared frame, a border, a title, and its options.
pub fn draw_option_popup<'a>(
    tokens: &Tokens,
    title: &str,
    options: &dyn Fn(usize) -> Option<&'a str>,
    count: usize,
    selected: i32,
) {
    if count == 0 {
        // An empty frame is worse than nothing: it looks like a dialog that
        // failed to load. `Modal` guards this, but these are public.
        return;
    }
    let layout = popup_layout(tokens, title, count);

    // Cleared first: a dialog sits over content, and its own background is the
    // only thing making it legible.
    Renderer::fill_rect(layout.frame, false);
    for ring in 0..tokens.dialog_border {
        Renderer::stroke_rect(Rect::new(
            layout.frame.x() + ring,
            layout.frame.y() + ring,
            layout.frame.width() - ring * 2,
            layout.frame.height() - ring * 2,
        ));
    }

    if !title.is_empty() {
        let font = Font::ui().bold();
        let inner = tokens.dialog_border + tokens.dialog_padding;
        draw_truncated(
            layout.frame.x() + inner,
            layout.frame.y() + inner,
            title,
            font,
            layout.frame.width() - inner * 2,
        );
    }

    // `visible`, not `count`: a dialog clamped to the panel must not paint the
    // rows that did not fit, or they land off the bottom edge where nobody can
    // see or touch them.
    for index in 0..layout.visible {
        let Some(label) = options(index) else {
            continue;
        };
        let bounds = Rect::new(
            layout.first_row.x(),
            layout.first_row.y() + layout.stride * index as i32,
            layout.first_row.width(),
            layout.first_row.height(),
        );
        draw_row(
            tokens,
            bounds,
            index as i32 == selected,
            index,
            &|_, field| match field {
                RowField::Title => Some(label),
                _ => None,
            },
        );
    }
}

/// Screen rect of one dialog row, for hit-testing.
pub fn option_popup_row_rect<'a>(
    tokens: &Tokens,
    title: &str,
    _options: &dyn Fn(usize) -> Option<&'a str>,
    count: usize,
    index: usize,
) -> Option<Rect> {
    if index >= count {
        return None;
    }
    let layout = popup_layout(tokens, title, count);
    if index >= layout.visible {
        // Off the bottom of a clamped dialog: not painted, so not touchable.
        return None;
    }
    Some(Rect::new(
        layout.first_row.x(),
        layout.first_row.y() + layout.stride * index as i32,
        layout.first_row.width(),
        layout.first_row.height(),
    ))
}
