//! The title band and a group heading, each with the rule that separates it
//! from what follows.

use xpui::{Font, Rect, Renderer};

use super::text::{centred_y, draw_truncated};
use crate::metrics::Metrics;

/// The title band, with a rule under it.
pub fn draw_header(metrics: &Metrics, title: Option<&str>, subtitle: Option<&str>) {
    let width = Renderer::screen_size().width;
    let band = Rect::new(0, metrics.top_padding, width, metrics.header_height);
    let font = Font::ui().bold();
    let inset = metrics.content_side_padding;

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
        room -= subtitle_width + metrics.vertical_spacing;
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
pub fn draw_sub_header(metrics: &Metrics, rect: Rect, label: &str, right: Option<&str>) {
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
        room -= width + metrics.spacing_small;
    }

    draw_truncated(rect.x(), rect.y(), label, font, room);

    // A hairline along the baseline of the heading, starting after the label,
    // which is what makes a heading read as a divider rather than a row.
    let label_width = font.text_width(label).min(room.max(0));
    let start = rect.x() + label_width + metrics.spacing_small;
    // `- 1`: a rect's right edge is exclusive, so `x + width` is the first
    // pixel *outside* it.
    let end = rect.x() + room - 1;
    if end > start {
        let y = rect.y() + rect.height() - 1;
        Renderer::draw_line(xpui::Point::new(start, y), xpui::Point::new(end, y));
    }
}
