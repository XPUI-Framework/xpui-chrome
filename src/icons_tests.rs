//! What `icons.rs`'s private geometry has to be true of.

use super::half_width;

/// The integer answer is the real one: the largest `h` with
/// `h² + row² <= r²`. Checked against that definition rather than against
/// a table, so the test says what the function means.
#[test]
fn half_width_is_the_largest_that_stays_inside_the_circle() {
    for radius in 0..24 {
        for row in -radius..=radius {
            let half = half_width(radius, row);
            assert!(
                half * half + row * row <= radius * radius,
                "r={radius} row={row}: {half} is outside the circle"
            );
            let next = half + 1;
            assert!(
                next * next + row * row > radius * radius,
                "r={radius} row={row}: {next} would also fit, so {half} is short"
            );
        }
    }
}

/// Outside the circle entirely — no row, no width, no panic.
#[test]
fn a_row_past_the_radius_has_no_width() {
    assert_eq!(half_width(5, 6), 0);
    assert_eq!(half_width(0, 0), 0);
}

/// A circle's widest row is its middle; an approximation that insets the
/// last rows makes a square disc.
#[test]
fn the_widest_row_is_the_middle_one() {
    let radius = 12;
    let widest = (-radius..=radius)
        .max_by_key(|&row| half_width(radius, row))
        .unwrap();
    assert_eq!(widest, 0);
    assert!(half_width(radius, 0) > half_width(radius, radius - 1));
}

/// A solid book's spine is paper over its ink cover. Drawn in ink, as the
/// outline's is, it vanishes into the fill and the glyph is a black square.
#[test]
fn a_solid_book_keeps_its_spine() {
    use super::{Icon, draw_icon};
    use xpui::Point;
    use xpui::host::IconRef;
    use xpui::testing::{self, DrawOp, RectKind};

    testing::install();
    testing::reset();
    let solid = IconRef {
        variant: 1,
        size: 16,
        ..Icon::Book.into()
    };
    draw_icon(Point::new(0, 0), solid);

    let fills: Vec<_> = testing::ops_log()
        .into_iter()
        .filter_map(|op| match op {
            DrawOp::Rect {
                rect,
                kind: RectKind::Filled,
                black,
            } => Some((rect, black)),
            _ => None,
        })
        .collect();
    let [(cover, true), (spine, false)] = fills[..] else {
        panic!("expected an ink cover, then a paper spine: {fills:?}");
    };
    assert_eq!(spine.width(), 1);
    assert!(spine.x() > cover.x() && spine.x() < cover.x() + cover.width() - 1);
    assert!(spine.y() > cover.y() && spine.y() + spine.height() < cover.y() + cover.height());
}
