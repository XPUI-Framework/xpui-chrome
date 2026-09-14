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
//! use xpui_chrome::{KeyRow, Labels, Metrics};
//!
//! static METRICS: Metrics = Metrics::DEFAULT;
//! static LABELS: Labels = Labels::ENGLISH;
//! static KEYS: KeyRow = KeyRow::READER;
//!
//! xpui_chrome::plain_chrome! {
//!     for MyBackend,
//!     metrics: |_backend| &METRICS,
//!     labels: |_backend| &LABELS,
//!     keys: |_backend| &KEYS,
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
//! Nine of the trait's eleven methods are also plain functions, so a backend
//! that has *some* components of its own can take only the ones it lacks.
//! The other two are not on offer: `metric` is answered by
//! [`Metrics::metric`], and `request_update` is one of the four things a
//! backend passes in.
//!
//! # How it reaches the canvas
//!
//! Through `xpui`'s installed host, like a widget does — not through a `&self`
//! parameter. So these functions paint through whichever backend is installed,
//! which is the one that called them. Nothing here needs to know what that is.

#![cfg_attr(target_os = "none", no_std)]
#![deny(missing_docs)]

mod icons;
mod labels;
mod metrics;
mod paint;
mod presets;

pub use icons::{Icon, draw_icon, icon_size};
pub use labels::Labels;
pub use metrics::Metrics;
pub use paint::{
    PopupLayout, draw_button_hints, draw_header, draw_list, draw_option_popup, draw_progress_bar,
    draw_scroll_indicator, draw_slider, draw_sub_header, option_popup_row_rect, popup_layout,
    row_height, rows_that_fit,
};
/// Re-exported so a backend needs one import for the three things
/// [`plain_chrome!`] asks it for, though this one is `xpui`'s: a key row is a
/// fact about hardware, not about painting.
pub use xpui::host::{KeyRow, RowKey};

/// Writes a complete [`Chrome`](xpui::host::Chrome) implementation for a
/// backend that supplies only primitives.
///
/// `request_update` has no sensible default — only the backend knows how to
/// get pixels onto a panel — so it is one of the four things you pass in.
/// Each is a closure over the backend, so one carrying its own can answer
/// `|backend| &backend.metrics` rather than reaching for a global; why the
/// other three — `metrics`, `labels`, `keys` — are separate is on
/// [`draw_button_hints`].
///
/// ```rust
/// # use xpui_chrome::{KeyRow, Labels, Metrics};
/// # struct MyBackend;
/// # impl MyBackend { fn mark_dirty(&self) {} }
/// # static METRICS: Metrics = Metrics::DEFAULT;
/// # static LABELS: Labels = Labels::ENGLISH;
/// # static KEYS: KeyRow = KeyRow::READER;
/// xpui_chrome::plain_chrome! {
///     for MyBackend,
///     metrics: |_backend| &METRICS,
///     labels: |_backend| &LABELS,
///     keys: |_backend| &KEYS,
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
        metrics: |$binding:ident| $metrics:expr,
        labels: |$lb:ident| $labels:expr,
        keys: |$kb:ident| $keys:expr,
        request_update: |$this:ident| $update:expr $(,)?
    ) => {
        $crate::__plain_chrome!(
            [$($generics)*], $backend,
            $binding, $metrics, $lb, $labels, $kb, $keys, $this, $update
        );
    };

    (
        for $backend:ty,
        metrics: |$binding:ident| $metrics:expr,
        labels: |$lb:ident| $labels:expr,
        keys: |$kb:ident| $keys:expr,
        request_update: |$this:ident| $update:expr $(,)?
    ) => {
        $crate::__plain_chrome!(
            [], $backend,
            $binding, $metrics, $lb, $labels, $kb, $keys, $this, $update
        );
    };
}

/// The body both [`plain_chrome!`] arms expand to, with the generics spliced
/// in. Separate only because a macro cannot capture `impl<..>` as one fragment.
#[doc(hidden)]
#[macro_export]
macro_rules! __plain_chrome {
    (
        [$($generics:tt)*], $backend:ty,
        $binding:ident, $metrics:expr, $lb:ident, $labels:expr, $kb:ident, $keys:expr,
        $this:ident, $update:expr
    ) => {
        impl<$($generics)*> $crate::__private::Chrome for $backend {
            fn metric(&self, metric: $crate::__private::ThemeMetric) -> i32 {
                { let $binding = self; $metrics }.metric(metric)
            }

            fn draw_header(&self, title: Option<&str>, subtitle: Option<&str>) {
                $crate::draw_header({ let $binding = self; $metrics }, title, subtitle)
            }

            fn draw_sub_header(
                &self,
                rect: $crate::__private::Rect,
                label: &str,
                right: Option<&str>,
            ) {
                $crate::draw_sub_header({ let $binding = self; $metrics }, rect, label, right)
            }

            fn draw_button_hints(
                &self,
                back: &$crate::__private::Hint,
                confirm: &$crate::__private::Hint,
                previous: &$crate::__private::Hint,
                next: &$crate::__private::Hint,
            ) {
                $crate::draw_button_hints(
                    { let $binding = self; $metrics },
                    { let $lb = self; $labels },
                    { let $kb = self; $keys },
                    back, confirm, previous, next,
                )
            }

            fn draw_progress_bar(&self, rect: $crate::__private::Rect, current: u32, total: u32) {
                $crate::draw_progress_bar({ let $binding = self; $metrics }, rect, current, total)
            }

            fn draw_slider(
                &self,
                rect: $crate::__private::Rect,
                value: i32,
                max: i32,
                state: $crate::__private::ControlState,
            ) {
                $crate::draw_slider({ let $binding = self; $metrics }, rect, value, max, state)
            }

            fn draw_scroll_indicator(
                &self,
                rect: $crate::__private::Rect,
                content: i32,
                visible: i32,
                offset: i32,
            ) {
                $crate::draw_scroll_indicator({ let $binding = self; $metrics }, rect, content, visible, offset)
            }

            fn draw_list<'a>(
                &self,
                rect: $crate::__private::Rect,
                rows: usize,
                selected: i32,
                row: &dyn Fn(usize, $crate::__private::RowField) -> Option<&'a str>,
            ) {
                $crate::draw_list({ let $binding = self; $metrics }, rect, rows, selected, row)
            }

            fn draw_option_popup<'a>(
                &self,
                title: &str,
                options: &dyn Fn(usize) -> Option<&'a str>,
                count: usize,
                selected: i32,
            ) {
                $crate::draw_option_popup({ let $binding = self; $metrics }, title, options, count, selected)
            }

            fn option_popup_row_rect<'a>(
                &self,
                title: &str,
                options: &dyn Fn(usize) -> Option<&'a str>,
                count: usize,
                index: usize,
            ) -> Option<$crate::__private::Rect> {
                $crate::option_popup_row_rect({ let $binding = self; $metrics }, title, options, count, index)
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
    pub use xpui::host::{Chrome, ControlState, Hint, RowField, ThemeMetric};
}

/// The crate's prose, compiled: a page that does not build is worse than
/// none.
#[cfg(doctest)]
mod guides {
    #[doc = include_str!("../README.md")]
    pub mod readme {}
    #[doc = include_str!("../docs/components.md")]
    pub mod components {}
    #[doc = include_str!("../docs/design.md")]
    pub mod design {}
    #[doc = include_str!("../docs/reference.md")]
    pub mod reference {}
    #[doc = include_str!("../docs/reference/plain-chrome.md")]
    pub mod reference_plain_chrome {}
    #[doc = include_str!("../docs/reference/painting.md")]
    pub mod reference_painting {}
    #[doc = include_str!("../docs/reference/metrics-and-labels.md")]
    pub mod reference_metrics_and_labels {}
    #[doc = include_str!("../docs/reference/icons.md")]
    pub mod reference_icons {}
}
