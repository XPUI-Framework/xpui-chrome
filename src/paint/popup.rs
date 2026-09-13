//! The centred option dialog, and where each of its rows lands.

use xpui::host::RowField;
use xpui::{Font, Rect, Renderer};

use super::list::draw_row;
use super::text::draw_truncated;
use crate::metrics::Metrics;

/// Where a dialog and its rows land.
///
/// Both [`draw_option_popup`] and [`option_popup_row_rect`] go through this.
/// Deriving the geometry twice is how a dialog ends up painting row 3 where
/// row 2 responds to a touch, and nothing about that failure looks wrong until
/// somebody taps it.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PopupLayout {
    /// The dialog's outer rectangle, border included.
    pub frame: Rect,
    /// The first option's row; the rest follow at `stride`.
    pub first_row: Rect,
    /// From one row's top to the next.
    pub stride: i32,
    /// How many rows actually fit on the panel.
    ///
    /// A dialog with more options than the screen is tall is drawn as tall as
    /// it fits and no taller, rather than running its frame off the bottom
    /// edge. It does not scroll: an option at or past `visible` is neither
    /// painted nor given a rect, so a `selected` there paints no highlight at
    /// all. A twenty-entry language picker reaches this on a 480x800 panel.
    pub visible: usize,
}

/// Where a dialog titled `title` with `count` options lands on the panel.
pub fn popup_layout(metrics: &Metrics, title: &str, count: usize) -> PopupLayout {
    let screen = Renderer::screen_size();
    let title_font = Font::ui().bold();
    let border = metrics.dialog_border;
    let padding = metrics.dialog_padding;

    let title_band = if title.is_empty() {
        0
    } else {
        title_font.line_height() + padding
    };

    let row_height = metrics.list_row_height.max(1);
    let chrome = border * 2 + padding * 2 + title_band;
    // Never taller than the panel, and never fewer than one row — a dialog
    // showing nothing is not an improvement on one showing too much.
    let room = (screen.height - chrome).max(row_height);
    let visible = count.min((room / row_height).max(1) as usize);

    let body = row_height * visible as i32;
    let height = chrome + body;
    let width = (screen.width * metrics.dialog_width_percent / 100).min(screen.width);

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
    metrics: &Metrics,
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
    let layout = popup_layout(metrics, title, count);

    // Cleared first: a dialog sits over content, and its own background is the
    // only thing making it legible.
    Renderer::fill_rect(layout.frame, false);
    for ring in 0..metrics.dialog_border {
        Renderer::stroke_rect(Rect::new(
            layout.frame.x() + ring,
            layout.frame.y() + ring,
            layout.frame.width() - ring * 2,
            layout.frame.height() - ring * 2,
        ));
    }

    if !title.is_empty() {
        let font = Font::ui().bold();
        let inner = metrics.dialog_border + metrics.dialog_padding;
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
            metrics,
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
    metrics: &Metrics,
    title: &str,
    _options: &dyn Fn(usize) -> Option<&'a str>,
    count: usize,
    index: usize,
) -> Option<Rect> {
    if index >= count {
        return None;
    }
    let layout = popup_layout(metrics, title, count);
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
