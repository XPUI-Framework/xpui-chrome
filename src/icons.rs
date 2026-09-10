//! Icons drawn from lines and rectangles.
//!
//! A backend sitting on a drawing library has no asset set, and
//! `Canvas::draw_icon` quietly does nothing — an `Icon` leaves a hole where a
//! screen expected a glyph, and `icon_size` reporting 0 means the layout does
//! not even reserve space for it. These are drawn, not stored: no bitmaps, no
//! font, nothing in flash, and at 16 to 32 pixels on a 1-bit panel a drawn
//! glyph and a stored one look much the same.
//!
//! `IconRef::kind` is opaque to the framework: the host decides what 3 means.
//! [`Icon`] is this crate's answer; a backend that ships real assets maps the
//! same names onto those instead.

use xpui::host::IconRef;
use xpui::{Point, Rect, Renderer};

/// The icons this crate can draw.
///
/// `#[repr(u16)]` so a screen can write `Icon::Sun` and the number that crosses
/// to the backend is this enum's discriminant — the mapping is the enum, rather
/// than a table somewhere that has to agree with one.
#[repr(u16)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Icon {
    /// Brightness, a frontlight, daytime.
    Sun = 0,
    /// Night, sleep, a dark theme.
    Moon = 1,
    /// Settings.
    Gear = 2,
    /// Back, or previous.
    ChevronLeft = 3,
    /// Forward, or next, or "this row opens something".
    ChevronRight = 4,
    /// Up, or previous.
    ChevronUp = 5,
    /// Down, or next, or "this opens below".
    ChevronDown = 6,
    /// A confirmation, or a setting that is on.
    Check = 7,
    /// A dismissal, or a setting that is off.
    Cross = 8,
    /// Charge; every variant past the first fills the body.
    Battery = 9,
    /// Wireless, in the usual three arcs.
    Wifi = 10,
    /// A book, a library, a document.
    Book = 11,
}

impl Icon {
    /// Every icon. `from_kind` looks a kind up here, so a variant missing
    /// from this array is undrawable: `icon_size` answers 0 and `draw_icon`
    /// returns early.
    pub const ALL: [Icon; 12] = [
        Icon::Sun,
        Icon::Moon,
        Icon::Gear,
        Icon::ChevronLeft,
        Icon::ChevronRight,
        Icon::ChevronUp,
        Icon::ChevronDown,
        Icon::Check,
        Icon::Cross,
        Icon::Battery,
        Icon::Wifi,
        Icon::Book,
    ];

    /// The number that crosses to a backend.
    pub const fn kind(self) -> u16 {
        self as u16
    }

    fn from_kind(kind: u16) -> Option<Icon> {
        Icon::ALL.into_iter().find(|icon| icon.kind() == kind)
    }
}

impl From<Icon> for IconRef {
    fn from(icon: Icon) -> IconRef {
        IconRef::new(icon.kind())
    }
}

/// The smallest icon worth drawing. Below this the strokes collide and a gear
/// and a sun are the same smudge, so a backend is better off drawing nothing.
const MINIMUM: i32 = 8;

/// The edge length this crate would actually draw, or 0 if it would draw
/// nothing — which is how the framework knows to reserve no space.
///
/// Rounded down to an even number: every glyph here is symmetric about its own
/// centre, and an odd edge puts that centre half a pixel off, which on a 1-bit
/// panel is the difference between a straight line and a stepped one.
pub fn icon_size(icon: IconRef) -> i32 {
    if Icon::from_kind(icon.kind).is_none() || icon.size < MINIMUM {
        return 0;
    }
    icon.size - (icon.size % 2)
}

/// Draws `icon` with its top-left corner at `origin`.
pub fn draw_icon(origin: Point, icon: IconRef) {
    let size = icon_size(icon);
    let Some(kind) = Icon::from_kind(icon.kind) else {
        return;
    };
    if size == 0 {
        return;
    }

    let bounds = Rect::new(origin.x, origin.y, size, size);
    // `variant` is "solid rather than outline" where a glyph has both.
    let solid = icon.variant != 0;

    match kind {
        Icon::Sun => sun(bounds, solid),
        Icon::Moon => moon(bounds, solid),
        Icon::Gear => gear(bounds, solid),
        Icon::ChevronLeft => chevron(bounds, -1, 0),
        Icon::ChevronRight => chevron(bounds, 1, 0),
        Icon::ChevronUp => chevron(bounds, 0, -1),
        Icon::ChevronDown => chevron(bounds, 0, 1),
        Icon::Check => check(bounds),
        Icon::Cross => cross(bounds),
        Icon::Battery => battery(bounds, solid),
        Icon::Wifi => wifi(bounds),
        Icon::Book => book(bounds, solid),
    }
}

// -- the glyphs ------------------------------------------------------------

fn centre(bounds: Rect) -> Point {
    Point::new(
        bounds.x() + bounds.width() / 2,
        bounds.y() + bounds.height() / 2,
    )
}

/// Half-width of a circle of `radius` at `row` rows from its centre.
///
/// Integer throughout: `no_std` has no `sqrt` without pulling in `libm`, and
/// the largest `h` with `h² + row² <= r²` is the same answer.
fn half_width(radius: i32, row: i32) -> i32 {
    let limit = radius * radius - row * row;
    if limit <= 0 {
        return 0;
    }
    let mut half = 0;
    while (half + 1) * (half + 1) <= limit {
        half += 1;
    }
    half
}

fn disc(bounds: Rect, radius: i32, filled: bool) {
    let middle = centre(bounds);
    for row in -radius..=radius {
        let half = half_width(radius, row);
        if half <= 0 {
            continue;
        }
        if filled {
            Renderer::fill_rect(
                Rect::new(middle.x - half, middle.y + row, half * 2, 1),
                true,
            );
            continue;
        }

        // Outline: the two edge pixels of the row — plus the whole row where
        // the circle turns over, or the cap comes out as two loose dots.
        let above = half_width(radius, row - 1).min(half_width(radius, row + 1));
        if half - above > 1 {
            Renderer::fill_rect(
                Rect::new(middle.x - half, middle.y + row, half * 2, 1),
                true,
            );
        } else {
            Renderer::fill_rect(Rect::new(middle.x - half, middle.y + row, 1, 1), true);
            Renderer::fill_rect(Rect::new(middle.x + half - 1, middle.y + row, 1, 1), true);
        }
    }
}

fn sun(bounds: Rect, solid: bool) {
    let middle = centre(bounds);
    let radius = bounds.width() / 4;
    disc(bounds, radius, solid);

    let reach = bounds.width() / 2 - 1;
    let inner = radius + radius / 2;
    for (dx, dy) in [
        (0, -1),
        (1, 0),
        (0, 1),
        (-1, 0),
        (1, -1),
        (1, 1),
        (-1, 1),
        (-1, -1),
    ] {
        Renderer::draw_line(
            Point::new(middle.x + dx * inner, middle.y + dy * inner),
            Point::new(middle.x + dx * reach, middle.y + dy * reach),
        );
    }
}

/// A crescent: a filled disc with a second disc bitten out of it.
///
/// Always filled, whatever `variant` asked for: an *outlined* crescent is an
/// arc with the bite erasing half of it, which reads as a bracket.
fn moon(bounds: Rect, _solid: bool) {
    let radius = bounds.width() / 2 - 1;
    disc(bounds, radius, true);

    // Offset by half a radius and slightly larger, which is what leaves a
    // crescent rather than a disc with a dent in it.
    let middle = centre(bounds);
    bite(
        Point::new(middle.x + radius / 2, middle.y - radius / 6),
        radius,
    );
}

/// Clears a disc, so whatever it overlaps is cut away.
fn bite(middle: Point, radius: i32) {
    for row in -radius..=radius {
        let half = half_width(radius, row);
        if half <= 0 {
            continue;
        }
        Renderer::fill_rect(
            Rect::new(middle.x - half, middle.y + row, half * 2, 1),
            false,
        );
    }
}

fn gear(bounds: Rect, solid: bool) {
    let middle = centre(bounds);
    let radius = bounds.width() / 3;
    disc(bounds, radius, solid);

    // Four teeth. Eight would be truer to a gear and indistinguishable at
    // sixteen pixels.
    let tooth = (bounds.width() / 8).max(2);
    for (dx, dy) in [(0, -1), (1, 0), (0, 1), (-1, 0)] {
        let x = middle.x + dx * radius - tooth / 2;
        let y = middle.y + dy * radius - tooth / 2;
        Renderer::fill_rect(Rect::new(x, y, tooth, tooth), true);
    }
}

/// An arrowhead pointing along (`dx`, `dy`).
fn chevron(bounds: Rect, dx: i32, dy: i32) {
    let middle = centre(bounds);
    let reach = bounds.width() / 3;
    let tip = Point::new(middle.x + dx * reach, middle.y + dy * reach);

    // Perpendicular to the direction of travel.
    let (px, py) = (dy, dx);
    for side in [-1, 1] {
        Renderer::draw_line(
            tip,
            Point::new(
                middle.x - dx * reach + px * side * reach,
                middle.y - dy * reach + py * side * reach,
            ),
        );
    }
}

fn check(bounds: Rect) {
    let middle = centre(bounds);
    let reach = bounds.width() / 3;
    Renderer::draw_line(
        Point::new(middle.x - reach, middle.y),
        Point::new(middle.x - reach / 3, middle.y + reach),
    );
    Renderer::draw_line(
        Point::new(middle.x - reach / 3, middle.y + reach),
        Point::new(middle.x + reach, middle.y - reach),
    );
}

fn cross(bounds: Rect) {
    let middle = centre(bounds);
    let reach = bounds.width() / 3;
    Renderer::draw_line(
        Point::new(middle.x - reach, middle.y - reach),
        Point::new(middle.x + reach, middle.y + reach),
    );
    Renderer::draw_line(
        Point::new(middle.x + reach, middle.y - reach),
        Point::new(middle.x - reach, middle.y + reach),
    );
}

fn battery(bounds: Rect, solid: bool) {
    let width = bounds.width() * 3 / 4;
    let height = bounds.height() / 2;
    let middle = centre(bounds);
    let body = Rect::new(
        middle.x - width / 2,
        middle.y - height / 2,
        width - 2,
        height,
    );
    Renderer::stroke_rect(body);

    // The terminal, on the right.
    let nub = (height / 3).max(1);
    Renderer::fill_rect(
        Rect::new(body.x() + body.width(), middle.y - nub / 2, 2, nub),
        true,
    );

    if solid {
        Renderer::fill_rect(
            Rect::new(
                body.x() + 2,
                body.y() + 2,
                (body.width() - 4).max(1),
                (body.height() - 4).max(1),
            ),
            true,
        );
    }
}

/// Three arcs over a dot. The arcs are chevrons pointing up, which at this
/// size is what an arc looks like anyway.
fn wifi(bounds: Rect) {
    let middle = centre(bounds);
    let base = bounds.y() + bounds.height() * 3 / 4;

    Renderer::fill_rect(Rect::new(middle.x - 1, base, 2, 2), true);

    for step in 1..=3 {
        let spread = bounds.width() * step / 8;
        let lift = base - bounds.height() * step / 5;
        Renderer::draw_line(
            Point::new(middle.x - spread, lift + spread / 2),
            Point::new(middle.x, lift),
        );
        Renderer::draw_line(
            Point::new(middle.x, lift),
            Point::new(middle.x + spread, lift + spread / 2),
        );
    }
}

fn book(bounds: Rect, solid: bool) {
    let width = bounds.width() * 3 / 4;
    let height = bounds.height() * 3 / 4;
    let middle = centre(bounds);
    let cover = Rect::new(middle.x - width / 2, middle.y - height / 2, width, height);

    if solid {
        Renderer::fill_rect(cover, true);
    } else {
        Renderer::stroke_rect(cover);
    }

    // The spine, a third of the way in — what makes it a book rather than a
    // rectangle.
    Renderer::draw_line(
        Point::new(cover.x() + cover.width() / 3, cover.y()),
        Point::new(
            cover.x() + cover.width() / 3,
            cover.y() + cover.height() - 1,
        ),
    );
}

/// The unit tests, in a file of their own: they test private functions, so
/// they cannot move to `tests/`, and a sibling keeps them beside the code
/// without sharing this file's line budget.
#[cfg(test)]
#[path = "icons_tests.rs"]
mod tests;
