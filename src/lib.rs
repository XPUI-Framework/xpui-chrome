//! Themed components for [`xpui`], painted from drawing primitives alone.
//!
//! `xpui` asks a backend to paint eight things it deliberately has no opinion
//! about: a list, a dialog, a slider, a progress bar, a header, a sub-header,
//! a button-hint bar and a scroll indicator. A backend sitting on a component
//! library answers by calling that library. A backend sitting on a *drawing*
//! library — `embedded_graphics`, a raw framebuffer — has nothing to call, and
//! would otherwise have to write a widget toolkit before it could show
//! anything at all.
//!
//! This crate is that toolkit, once, for all of them.
//!
//! ```rust
//! # struct MyBackend;
//! # impl MyBackend { fn flush(&self) {} }
//! use xpui_chrome::Tokens;
//!
//! static TOKENS: Tokens = Tokens::DEFAULT;
//!
//! xpui_chrome::plain_chrome! {
//!     for MyBackend,
//!     tokens: |_backend| &TOKENS,
//!     request_update: |backend| backend.flush(),
//! }
//! ```
//!
//! # Why a macro rather than a blanket impl
//!
//! `impl<T: Canvas + TextMetrics> Chrome for T` is what this obviously wants to
//! be, and it is illegal: both the trait and the type parameter are foreign to
//! this crate, so the orphan rule rejects it (`E0210`). The macro writes the
//! same impl into the backend's own crate, where it is allowed.
//!
//! Every method is also available as a plain function, so a backend that has
//! *some* components of its own can take only the ones it lacks — which is
//! exactly what the FreeInkUI backend does for the two pieces FreeInkUI has no
//! component for.
//!
//! # How it reaches the canvas
//!
//! Through `xpui`'s installed host, like a widget does — not through a `&self`
//! parameter. So these functions paint through whichever backend is installed,
//! which is the one that called them. Nothing here needs to know what that is.

#![cfg_attr(target_os = "none", no_std)]

extern crate alloc;

mod icons;
mod paint;
mod row;
mod tokens;

pub use icons::{Icon, draw_icon, icon_size};
pub use paint::{
    PopupLayout, draw_button_hints, draw_header, draw_list, draw_option_popup, draw_progress_bar,
    draw_scroll_indicator, draw_slider, draw_sub_header, option_popup_row_rect, popup_layout,
    row_height, rows_that_fit,
};
pub use row::{READER_ROW, RowKey};
pub use tokens::Tokens;

/// Writes a complete [`Chrome`](xpui::host::Chrome) implementation for a
/// backend that supplies only primitives.
///
/// `request_update` has no sensible default — only the backend knows how to
/// get pixels onto a panel — so it is the one thing you pass in.
///
/// `tokens` is a closure over the backend too, so one that carries its own can
/// answer `|backend| &backend.tokens` rather than reaching for a global.
///
/// ```rust
/// # use xpui_chrome::Tokens;
/// # struct MyBackend;
/// # impl MyBackend { fn mark_dirty(&self) {} }
/// # static TOKENS: Tokens = Tokens::DEFAULT;
/// xpui_chrome::plain_chrome! {
///     for MyBackend,
///     tokens: |_backend| &TOKENS,
///     request_update: |backend| backend.mark_dirty(),
/// }
/// ```
#[macro_export]
macro_rules! plain_chrome {
    // A generic backend. The parameters go in brackets rather than angle
    // brackets: `for <D: DrawTarget> ..` would be parsed as the start of a
    // qualified path (`<D as Trait>::Assoc`) and fail before this ever
    // matched, and a fragment parser that has started cannot back out.
    (
        generic: [$($generics:tt)*],
        for $backend:ty,
        tokens: |$binding:ident| $tokens:expr,
        request_update: |$this:ident| $update:expr $(,)?
    ) => {
        $crate::__plain_chrome!([$($generics)*], $backend, $binding, $tokens, $this, $update);
    };

    (
        for $backend:ty,
        tokens: |$binding:ident| $tokens:expr,
        request_update: |$this:ident| $update:expr $(,)?
    ) => {
        $crate::__plain_chrome!([], $backend, $binding, $tokens, $this, $update);
    };
}

/// The body both [`plain_chrome!`] arms expand to, with the generics spliced
/// in. Separate only because a macro cannot capture `impl<..>` as one fragment.
#[doc(hidden)]
#[macro_export]
macro_rules! __plain_chrome {
    (
        [$($generics:tt)*], $backend:ty, $binding:ident, $tokens:expr, $this:ident, $update:expr
    ) => {
        impl<$($generics)*> $crate::__private::Chrome for $backend {
            fn metric(&self, metric: $crate::__private::ThemeMetric) -> i32 {
                { let $binding = self; $tokens }.metric(metric)
            }

            fn draw_header(&self, title: Option<&str>, subtitle: Option<&str>) {
                $crate::draw_header({ let $binding = self; $tokens }, title, subtitle)
            }

            fn draw_sub_header(
                &self,
                rect: $crate::__private::Rect,
                label: &str,
                right: Option<&str>,
            ) {
                $crate::draw_sub_header({ let $binding = self; $tokens }, rect, label, right)
            }

            fn draw_button_hints(
                &self,
                back: &$crate::__private::Hint,
                confirm: &$crate::__private::Hint,
                previous: &$crate::__private::Hint,
                next: &$crate::__private::Hint,
            ) {
                $crate::draw_button_hints({ let $binding = self; $tokens }, back, confirm, previous, next)
            }

            fn draw_progress_bar(&self, rect: $crate::__private::Rect, current: u32, total: u32) {
                $crate::draw_progress_bar({ let $binding = self; $tokens }, rect, current, total)
            }

            fn draw_slider(&self, rect: $crate::__private::Rect, value: i32, max: i32) {
                $crate::draw_slider({ let $binding = self; $tokens }, rect, value, max)
            }

            fn draw_scroll_indicator(
                &self,
                rect: $crate::__private::Rect,
                content: i32,
                visible: i32,
                offset: i32,
            ) {
                $crate::draw_scroll_indicator({ let $binding = self; $tokens }, rect, content, visible, offset)
            }

            fn draw_list<'a>(
                &self,
                rect: $crate::__private::Rect,
                rows: usize,
                selected: i32,
                row: &dyn Fn(usize, $crate::__private::RowField) -> Option<&'a str>,
            ) {
                $crate::draw_list({ let $binding = self; $tokens }, rect, rows, selected, row)
            }

            fn draw_option_popup<'a>(
                &self,
                title: &str,
                options: &dyn Fn(usize) -> Option<&'a str>,
                count: usize,
                selected: i32,
            ) {
                $crate::draw_option_popup({ let $binding = self; $tokens }, title, options, count, selected)
            }

            fn option_popup_row_rect<'a>(
                &self,
                title: &str,
                options: &dyn Fn(usize) -> Option<&'a str>,
                count: usize,
                index: usize,
            ) -> Option<$crate::__private::Rect> {
                $crate::option_popup_row_rect({ let $binding = self; $tokens }, title, options, count, index)
            }

            fn request_update(&self) {
                let $this = self;
                $update
            }
        }
    };
}

/// Re-exports the macro expands to, so a backend needs no `use` of its own.
#[doc(hidden)]
pub mod __private {
    pub use xpui::Rect;
    pub use xpui::host::{Chrome, Hint, RowField, ThemeMetric};
}

/// The crate's prose, compiled.
///
/// A README that does not build is worse than none: this crate's only usage
/// example passed the wrong form to its own macro for as long as nothing
/// tried it.
#[cfg(doctest)]
mod guides {
    #[doc = include_str!("../README.md")]
    pub mod readme {}
}
