//! What the keys along the bottom edge mean, left to right.
//!
//! A hint bar asks two questions — how many slots to divide its band into, and
//! which word goes in each — and a board answers both with its row. They were
//! one question once, inferred from the key count: three keys was taken to
//! mean a badge with no key to spare for Back. That held until a board arrived
//! with three keys along the bottom *and* an up/down pair elsewhere, which has
//! a key for Back and gives it the first slot. Under the old reading every
//! label on those boards sat one key to the left of what it named.
//!
//! The words themselves are [`Tokens::standard_hints`](crate::Tokens), because
//! only a product knows what language its user reads. This is only which of
//! those words belongs over which key.

/// What one key along the bottom edge does.
///
/// The vocabulary a row is written in. Which *word* each of these paints is
/// [`Tokens::standard_hints`], because only a product knows what language its
/// user reads; this is only which of those words belongs over which key.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RowKey {
    Back,
    Confirm,
    /// Walks the list backwards. `Up` on a reader.
    Previous,
    /// Walks the list forwards. `Down` on a reader.
    Next,
    /// A key with no word in the hint vocabulary — either nothing is mapped to
    /// it, or what is has no label, as a Power key does. Drawn blank.
    Unassigned,
}

/// The row every board had before any of them said otherwise: a reader's four
/// keys, Back leftmost. Every token set starts here.
pub const READER_ROW: &[RowKey] = &[
    RowKey::Back,
    RowKey::Confirm,
    RowKey::Previous,
    RowKey::Next,
];
