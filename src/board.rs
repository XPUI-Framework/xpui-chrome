//! The boards this framework has been run on, as data.
//!
//! A screen never knows which board it is on. What differs is a handful of
//! numbers — how big the panel is, which token preset fits it, whether there
//! is a touchscreen — and those are worth writing down once rather than
//! rediscovering per project.
//!
//! Both the simulator and a real firmware read the same value, which is what
//! makes "develop in a window, then flash it" true rather than aspirational.
//! Nothing here touches hardware: it is a description, not a driver.

use crate::Tokens;

/// A panel, its chrome, and what it can be driven with.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Board {
    /// What to call it, for a window title or a log line.
    pub name: &'static str,
    pub width: i32,
    pub height: i32,
    /// The chrome sized for this panel.
    pub tokens: Tokens,
    /// Whether a finger can reach it. A board with five buttons and no
    /// touchscreen should not have its layout widened to 44-pixel finger
    /// targets, and a screen can ask before offering a drag-only control.
    pub touch: bool,
    /// Roughly how long a full refresh takes, in milliseconds.
    ///
    /// E-ink is the reason the framework repaints only when something changed.
    /// A board that answers 0 is a display fast enough not to care.
    pub refresh_ms: u32,
}

impl Board {
    /// Pimoroni Badger 2040 — RP2040, 296x128 monochrome e-ink (UC8151).
    ///
    /// The panel that made small-panel tokens necessary: the default chrome
    /// leaves 28 pixels of content here, which is not enough for a single list
    /// row.
    pub const BADGER_2040: Board = Board {
        name: "Badger 2040",
        width: 296,
        height: 128,
        tokens: Tokens::SMALL,
        touch: false,
        // A full UC8151 update is close to a second; partial modes are faster
        // but still nothing you would drive an animation with.
        refresh_ms: 900,
    };

    /// Pimoroni Tufty 2040 — RP2040, 320x240 colour IPS LCD (ST7789v).
    ///
    /// Colour hardware running a monochrome framework: the backend maps ink
    /// and background onto any two `Rgb565` values, so the same screens render
    /// black-on-white, or amber-on-black, without a screen knowing.
    pub const TUFTY_2040: Board = Board {
        name: "Tufty 2040",
        width: 320,
        height: 240,
        tokens: Tokens::COMPACT,
        touch: false,
        refresh_ms: 0,
    };

    /// A 480x800 portrait e-reader panel with a touchscreen.
    pub const READER_PORTRAIT: Board = Board {
        name: "Reader (portrait)",
        width: 480,
        height: 800,
        tokens: Tokens::DEFAULT,
        touch: true,
        refresh_ms: 1200,
    };

    /// The same panel on its side.
    pub const READER_LANDSCAPE: Board = Board {
        name: "Reader (landscape)",
        width: 800,
        height: 480,
        tokens: Tokens::DEFAULT,
        touch: true,
        refresh_ms: 1200,
    };

    /// Every board, so an example can offer them all without a table of its
    /// own that would fall behind this one.
    pub const ALL: [Board; 4] = [
        Board::BADGER_2040,
        Board::TUFTY_2040,
        Board::READER_PORTRAIT,
        Board::READER_LANDSCAPE,
    ];

    /// Looks a board up by a short name, for a command line.
    ///
    /// Matching is on a slug rather than the display name so
    /// `--board badger2040` works without quoting.
    pub fn from_slug(slug: &str) -> Option<Board> {
        match slug {
            "badger2040" | "badger" => Some(Board::BADGER_2040),
            "tufty2040" | "tufty" => Some(Board::TUFTY_2040),
            "reader" | "portrait" => Some(Board::READER_PORTRAIT),
            "reader-landscape" | "landscape" => Some(Board::READER_LANDSCAPE),
            _ => None,
        }
    }

    /// The slug [`from_slug`](Board::from_slug) accepts for this board.
    pub const fn slug(&self) -> &'static str {
        match (self.width, self.height) {
            (296, 128) => "badger2040",
            (320, 240) => "tufty2040",
            (800, 480) => "reader-landscape",
            _ => "reader",
        }
    }

    /// A board of an arbitrary size, with the chrome that fits it.
    ///
    /// For a panel not listed here — the point of the framework is that there
    /// will be many.
    pub const fn custom(name: &'static str, width: i32, height: i32, touch: bool) -> Board {
        Board {
            name,
            width,
            height,
            tokens: Tokens::for_panel(width, height),
            touch,
            refresh_ms: 0,
        }
    }

    /// How many list rows this board's content band holds. The number that
    /// decides whether a screen is usable on it at all.
    pub const fn list_rows(&self) -> i32 {
        self.tokens.list_rows_for(self.height)
    }
}
