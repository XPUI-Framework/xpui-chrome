# Plain chrome

`xpui` deliberately has no opinion about what a list row looks like. It asks,
and a backend answers through its `Chrome` implementation. A backend sitting on
a component library answers by calling that library, and a Rust screen ends up
pixel-identical to a native one. A backend sitting on a *drawing* library has
nothing to call, and this macro is what it answers with: one invocation writes
the whole implementation from four things the backend passes in.

![A whole panel painted by the chrome this crate writes: a Settings header reading 12:04, a Display group of three rows with Frontlight focused, a Reading group with a slider and a progress bar, a scroll indicator down the right edge, and Back, Select, Up and Down along the bottom](https://raw.githubusercontent.com/XPUI-Framework/xpui-gallery/main/gallery/tests/screenshots/reference/chrome_overview.png)

[The host contract](https://github.com/XPUI-Framework/xpui-framework/blob/main/docs/host.md)
is what a backend implements, and `Chrome` is the fifth of its traits. This page
is the macro that writes that one. Why it is a macro rather than a blanket
implementation is in [design.md](../design.md#a-macro-rather-than-a-blanket-impl).

## Topics

| | |
|---|---|
| [`plain_chrome!`](#plain_chrome) | Writes a complete `Chrome` implementation for a backend that supplies only primitives. |

## `plain_chrome!`

Writes a complete `Chrome` implementation for a backend that supplies only primitives.

```text
macro_rules! plain_chrome
```

```text
plain_chrome! {
    for Backend,
    metrics: |backend| metrics,
    labels: |backend| labels,
    keys: |backend| keys,
    request_update: |backend| expression,
}

plain_chrome! {
    generic: [parameters],
    for Backend<parameters>,
    metrics: |backend| metrics,
    labels: |backend| labels,
    keys: |backend| keys,
    request_update: |backend| expression,
}
```

The arguments are given in this order, and the trailing comma is optional.

| Argument | Meaning |
|---|---|
| `generic` | Optional. The implementation's generic parameters with their bounds, `[D: DrawTarget]`, spliced into `impl<…>`. |
| `for` | The backend type the implementation is for. It must implement `Canvas` and `TextMetrics`, because everything painted goes through them. |
| `metrics` | A `&Metrics`: every number the components paint to. See [`Metrics`](metrics-and-labels.md#metrics). |
| `labels` | A `&Labels`: the words on the hint bar. See [`Labels`](metrics-and-labels.md#labels). |
| `keys` | A `&KeyRow`: which key sits in each slot of the hint bar, left to right. See [`KeyRow`](metrics-and-labels.md#re-exports). |
| `request_update` | An expression of type `()` that gets pixels onto the panel. |

### The four things a backend passes in

It asks for four things, because four parties own them. The **metrics** size
the chrome and come from whoever knows the panel. The **labels** are words, and
only an application knows what language its user reads. The **key row** says
which word sits over which key, which is a fact about the hardware. And
`request_update` is there because only a backend knows how to get pixels onto a
panel: it has no sensible default.

All four are written as closures over the backend, not plain values, so a
backend carrying its own can answer `|backend| &backend.metrics`. Two backends
in one process may be driving two different panels, in two different languages,
with two different key rows, and globals would give them one of each between
them. Why `metrics`, `labels` and `keys` are separate from each other is on
[`draw_button_hints`](painting.md#draw_button_hints).

> [!NOTE]
> Each `|backend| …` is macro syntax, not a closure value: write it inline,
> never as the name of a closure. `backend` is bound to `&self`, and the
> expression is evaluated again on every call, so keep it a field read or a
> `static`. What `metrics`, `labels` and `keys` answer must be a reference that
> lives as long as `self`.

> [!WARNING]
> The generic parameters go in square brackets, never angle brackets.
> `for <D: DrawTarget> Backend<D>` is parsed as the start of a qualified path
> (`<D as Trait>::Assoc`) and fails before the macro can match it.

### What it writes

One `impl Chrome for Backend`, whose eleven methods each call this crate:

| `Chrome` method | Answered by |
|---|---|
| `metric` | [`Metrics::metric`](metrics-and-labels.md#metricsmetric), on what `metrics` returns |
| `draw_header` | [`draw_header`](painting.md#draw_header) |
| `draw_sub_header` | [`draw_sub_header`](painting.md#draw_sub_header) |
| `draw_button_hints` | [`draw_button_hints`](painting.md#draw_button_hints), with `metrics`, `labels` and `keys` |
| `draw_progress_bar` | [`draw_progress_bar`](painting.md#draw_progress_bar) |
| `draw_slider` | [`draw_slider`](painting.md#draw_slider) |
| `draw_scroll_indicator` | [`draw_scroll_indicator`](painting.md#draw_scroll_indicator) |
| `draw_list` | [`draw_list`](painting.md#draw_list) |
| `draw_option_popup` | [`draw_option_popup`](painting.md#draw_option_popup) |
| `option_popup_row_rect` | [`option_popup_row_rect`](painting.md#option_popup_row_rect) |
| `request_update` | the `request_update` expression |

The implementation needs no `use` in the backend's crate: the macro names every
type through this crate.

**Taking only some.** Nine of the trait's eleven methods are also plain
functions, so a backend with *some* components of its own can take only the
ones it lacks rather than the whole implementation. The other two are not on
offer: `metric` is answered by `Metrics::metric`, and `request_update` is one of
the four things a backend passes in. No backend takes that route yet:
`xpui-embedded-graphics` takes all nine through the macro, and the FreeInkUI
backend answers `Chrome` over its own C ABI without depending on this crate.

**Example — values in statics**

```rust
# struct MyBackend;
# impl MyBackend { fn flush(&self) {} }
use xpui_chrome::{KeyRow, Labels, Metrics};

static METRICS: Metrics = Metrics::DEFAULT;
static LABELS: Labels = Labels::ENGLISH;
static KEYS: KeyRow = KeyRow::READER;

xpui_chrome::plain_chrome! {
    for MyBackend,
    metrics: |_backend| &METRICS,
    labels: |_backend| &LABELS,
    keys: |_backend| &KEYS,
    request_update: |backend| backend.flush(),
}
```

**Example — a backend carrying its own**

```rust
use core::cell::Cell;
use xpui::host::{Chrome, ThemeMetric};
use xpui_chrome::{KeyRow, Labels, Metrics};

struct Badge {
    metrics: Metrics,
    labels: Labels,
    keys: KeyRow,
    dirty: Cell<bool>,
}

xpui_chrome::plain_chrome! {
    for Badge,
    metrics: |badge| &badge.metrics,
    labels: |badge| &badge.labels,
    keys: |badge| &badge.keys,
    request_update: |badge| badge.dirty.set(true),
}

let badge = Badge {
    metrics: Metrics::for_panel(296, 128),
    labels: Labels::for_panel(296, 128),
    keys: KeyRow::READER,
    dirty: Cell::new(false),
};
assert_eq!(badge.metric(ThemeMetric::ListRowHeight), 24);
badge.request_update();
assert!(badge.dirty.get());
```

**Example — a generic backend**

```rust
use xpui_chrome::{KeyRow, Labels, Metrics};

/// A backend over any display type.
struct Panel<D> {
    display: D,
    metrics: Metrics,
}

static LABELS: Labels = Labels::ENGLISH;
static KEYS: KeyRow = KeyRow::READER;

xpui_chrome::plain_chrome! {
    generic: [D: Send],
    for Panel<D>,
    metrics: |panel| &panel.metrics,
    labels: |_panel| &LABELS,
    keys: |_panel| &KEYS,
    request_update: |_panel| {},
}
# let _ = Panel { display: (), metrics: Metrics::DEFAULT }.display;
```

**Example — what the written implementation paints**

It paints through the installed host, so a call on the backend lands on
whichever canvas is installed: here, `xpui`'s recording fake.

```rust
use xpui::host::{Chrome, Hint};
use xpui_chrome::{KeyRow, Labels, Metrics};

struct Plain;

xpui_chrome::plain_chrome! {
    for Plain,
    metrics: |_backend| &Metrics::DEFAULT,
    labels: |_backend| &Labels::ENGLISH,
    keys: |_backend| &KeyRow::READER,
    request_update: |_backend| {},
}

xpui::testing::install();
xpui::testing::reset();
Plain.draw_button_hints(&Hint::Standard, &Hint::Done, &Hint::None, &Hint::None);

let text = xpui::testing::drawn_text();
let words: Vec<&str> = text.iter().map(|(_, _, word, _, _)| word.as_str()).collect();
assert_eq!(words, ["Back", "Done"]);
```

**See also:** [`Metrics`](metrics-and-labels.md#metrics), [`Labels`](metrics-and-labels.md#labels), [`draw_button_hints`](painting.md#draw_button_hints), [painting](painting.md)
