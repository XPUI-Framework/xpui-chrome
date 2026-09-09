//! The themed list: how tall a row is, how many fit, and how one is drawn.

use xpui::host::RowField;
use xpui::{Font, Rect, Renderer};

use super::text::{centred_y, draw_truncated};
use crate::metrics::Metrics;

/// How many of `rows` will actually be painted into `rect`.
///
/// `draw_list` stops before a row that does not fit, so the space it leaves
/// can be up to one row's stride. A caller that needs to know where the list
/// really ends has to ask rather than divide.
pub fn rows_that_fit<'a>(
    metrics: &Metrics,
    rect: Rect,
    rows: usize,
    row: &dyn Fn(usize, RowField) -> Option<&'a str>,
) -> usize {
    let height = row_height(metrics, rows, row);
    let stride = height + metrics.list_row_gap;
    (0..rows)
        .take_while(|index| {
            let top = rect.y() + stride * *index as i32;
            top + height <= rect.y() + rect.height()
        })
        .count()
}

/// The height every row in this list gets.
///
/// Uniform across the list, and chosen by whether **any** row carries a
/// subtitle — the same rule `xpui`'s `List` measures with, so the two agree
/// about how tall a list is.
pub fn row_height<'a>(
    metrics: &Metrics,
    rows: usize,
    row: &dyn Fn(usize, RowField) -> Option<&'a str>,
) -> i32 {
    let has_subtitle = (0..rows).any(|index| row(index, RowField::Subtitle).is_some());
    if has_subtitle {
        metrics.list_row_height_with_subtitle
    } else {
        metrics.list_row_height
    }
}

/// The themed list. Only rows that fit entirely are drawn.
pub fn draw_list<'a>(
    metrics: &Metrics,
    rect: Rect,
    rows: usize,
    selected: i32,
    row: &dyn Fn(usize, RowField) -> Option<&'a str>,
) {
    let height = row_height(metrics, rows, row);
    let stride = height + metrics.list_row_gap;

    for index in 0..rows {
        let top = rect.y() + stride * index as i32;
        // A partially visible row reads as a rendering fault rather than as
        // "there is more below", which is what the scroll indicator is for.
        if top + height > rect.y() + rect.height() {
            break;
        }
        let bounds = Rect::new(rect.x(), top, rect.width(), height);
        draw_row(metrics, bounds, index as i32 == selected, index, row);
    }
}

pub(super) fn draw_row<'a>(
    metrics: &Metrics,
    bounds: Rect,
    selected: bool,
    index: usize,
    row: &dyn Fn(usize, RowField) -> Option<&'a str>,
) {
    let marker = metrics.selection_marker_width;

    if selected {
        // A bar down the leading edge, plus an outline. Not an inverted fill:
        // text is always ink, so ink-on-ink would erase the label.
        Renderer::fill_rect(
            Rect::new(bounds.x(), bounds.y(), marker, bounds.height()),
            true,
        );
        Renderer::stroke_rect(bounds);
    }

    let text_x = bounds.x() + marker + metrics.spacing_small * 2;
    let mut room = bounds.width() - (text_x - bounds.x()) - metrics.spacing_small * 2;

    let title_font = Font::ui();
    let subtitle = row(index, RowField::Subtitle);

    if let Some(value) = row(index, RowField::Value) {
        let value_font = Font::ui();
        let width = value_font.text_width(value).min(room.max(0));
        draw_truncated(
            bounds.x() + bounds.width() - metrics.spacing_small * 2 - width,
            centred_y(bounds, value_font),
            value,
            value_font,
            width,
        );
        room -= width + metrics.spacing_small * 2;
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
