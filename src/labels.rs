//! The words on the hint bar.
//!
//! Separate from [`Metrics`](crate::Metrics) because they are not measurements
//! and separate from the key row because they are not hardware: only the
//! application knows what language its user reads. This crate ships English
//! because something has to be shipped, and every one of these is meant to be
//! replaced.

/// What a hint slot says.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Labels {
    /// The four standard hint words, in meaning order, for when a screen passes
    /// [`Hint::Standard`](xpui::host::Hint), in meaning order: back, confirm,
    /// previous, next.
    ///
    /// Which of these words lands over which key is the device's
    /// [`KeyRow`](xpui::KeyRow), not this.
    pub standard_hints: [&'static str; 4],
    /// The words a value control's mode needs, which the four above have no
    /// room for: opening one, keeping what it reads, and putting it back.
    ///
    /// Separate because the four are chosen by *which key* a slot sits over
    /// and these are chosen by what the framework is doing — no key implies
    /// "Edit".
    pub mode_hints: [&'static str; 3],
}

impl Labels {
    /// English, in the words a full-width band has room for.
    pub const ENGLISH: Labels = Labels {
        standard_hints: ["Back", "Select", "Up", "Down"],
        mode_hints: ["Edit", "Done", "Cancel"],
    };

    /// The same, shorter, for a band that has to hold them in less room.
    ///
    /// Used by a 320-pixel panel whose row is three keys — about 106 pixels a
    /// slot, against 120 on a 480-pixel panel with a reader's four.
    pub const ENGLISH_NARROW: Labels = Labels {
        standard_hints: ["Back", "OK", "Up", "Down"],
        mode_hints: ["Edit", "Done", "Cancel"],
    };

    /// Shorter still, for the narrowest band here: a 296-pixel strip divided
    /// by three keys, about 98 pixels a slot.
    ///
    /// Where a word has to give, abbreviating beats truncating — a reader can
    /// learn `Dn` and cannot learn `Canc…`. Which words need it is a judgement
    /// about the face in use, not a calculation; these are the ones the small
    /// preset was built with.
    pub const ENGLISH_SHORT: Labels = Labels {
        standard_hints: ["Back", "OK", "Up", "Dn"],
        mode_hints: ["Edit", "Done", "Undo"],
    };
}

impl Labels {
    /// The words that go with [`Metrics::for_panel`](crate::Metrics::for_panel).
    ///
    /// The same dispatch, deliberately: a panel small enough to need
    /// [`Metrics::SMALL`](crate::Metrics::SMALL) is small enough to need the
    /// words that preset was measured with, and picking one without the other
    /// is how a hint bar ends up overrunning its slot. `Metrics` and `Labels` are separate types
    /// because they have separate owners, not because a caller choosing by
    /// panel size should have to choose twice.
    pub const fn for_panel(width: i32, height: i32) -> Labels {
        let _ = width;
        if height <= 160 {
            Labels::ENGLISH_SHORT
        } else if height <= 320 {
            Labels::ENGLISH_NARROW
        } else {
            Labels::ENGLISH
        }
    }
}

impl Default for Labels {
    fn default() -> Self {
        Labels::ENGLISH
    }
}
