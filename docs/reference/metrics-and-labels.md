# Metrics and labels

What every paint function paints to and with. `Metrics` holds the numbers,
`Labels` the words, and a `KeyRow`, which is `xpui`'s, says which key each word
sits over. Three separate things, because three parties own them: the metrics
come from whoever knows the panel, the words from whoever knows the reader's
language, and the key row from the hardware.

![A hint bar under a rule reading Back, Select, Up and Down, each centred in a quarter of the width](https://raw.githubusercontent.com/XPUI-Framework/xpui-gallery/main/gallery/tests/screenshots/reference/chrome_button_hints.png)

[Plain chrome](plain-chrome.md) is how a backend hands all three over, and
[painting](painting.md) is what each number sizes.

## Topics

| | |
|---|---|
| [`Metrics`](#metrics) | Geometry for the components in this crate. |
| [`Labels`](#labels) | The words on the hint bar: four standard, three for an open value. |
| [`KeyRow`](#re-exports) | `xpui`'s, re-exported: what the keys along a device's bottom edge mean, left to right. |
| [`RowKey`](#re-exports) | `xpui`'s, re-exported: the job of one key in that row. |

## `Metrics`

Geometry for the components in this crate.

```text
pub struct Metrics
```

Every number these functions paint to, and every field a pixel count but one.
They are the values `xpui`'s `ThemeMetric` asks for, in one struct, plus a few
this crate needs and `xpui` does not name, so a backend can hand over a modified
copy without writing a match of its own. **A backend that wants a different
look changes these, not the drawing code.** A row height written as a literal at
a call site is right on one panel and wrong on every other.

Three presets ship, and [`for_panel`](#metricsfor_panel) picks one by the
panel's height. The defaults suit a portrait e-ink panel of roughly 480x800 at
1 bit. `Metrics` is `Copy`, `Eq`, and `Default`, which is `DEFAULT`.

### Fields

| Field | | `DEFAULT` | `COMPACT` | `SMALL` |
|---|---|---|---|---|
| `Metrics::top_padding` | Gap above the header band. | 8 | 2 | 0 |
| `Metrics::header_height` | Height of the header band. | 40 | 24 | 18 |
| `Metrics::vertical_spacing` | The standard gap between stacked elements. | 12 | 4 | 4 |
| `Metrics::spacing_small` | The small step, for space *within* a group. | 4 | 3 | 2 |
| `Metrics::button_hints_height` | Height reserved at the bottom for button hints. | 40 | 22 | 16 |
| `Metrics::content_side_padding` | Space between the panel's side edges and the content. | 16 | 8 | 4 |
| `Metrics::list_row_height` | Height of a one-line list row. | 40 | 30 | 24 |
| `Metrics::list_row_height_with_subtitle` | Height of a list row carrying a subtitle. | 56 | 42 | 34 |
| `Metrics::list_row_gap` | Space left between one row and the next. | 4 | 3 | 2 |
| `Metrics::selection_marker_width` | Width of the bar marking the selected row. | 4 | 3 | 3 |
| `Metrics::sub_header_height` | Height of the band a sub-header needs — the heading's own line, not a list row. | 17 | 14 | 12 |
| `Metrics::progress_bar_height` | Height of the progress bar. | 6 | 5 | 4 |
| `Metrics::min_touch_size` | Smallest comfortably tappable dimension. | 44 | 28 | 24 |
| `Metrics::slider_knob_width` | The slider knob's width. | 14 | 10 | 8 |
| `Metrics::slider_knob_height` | The slider knob's height, and so the least height a slider's rect needs to show the knob whole. | 22 | 18 | 14 |
| `Metrics::slider_side_inset` | Padding the slider track is inset by at each end. | 8 | 6 | 4 |
| `Metrics::slider_track_height` | Height of the slider's track. | 6 | 5 | 4 |
| `Metrics::scrollbar_width` | Width of the scroll indicator. | 4 | 3 | 3 |
| `Metrics::scrollbar_inset` | The scroll indicator's inset from the panel edge. | 2 | 2 | 1 |
| `Metrics::dialog_border` | Border thickness for a dialog. | 2 | 1 | 1 |
| `Metrics::dialog_padding` | Space between a dialog's border and what it holds. | 12 | 6 | 4 |
| `Metrics::dialog_width_percent` | Fraction of the panel width a dialog occupies, as a percentage. | 80 | 88 | 92 |

What the numbers have to keep true:

- **`spacing_small` stays smaller than `vertical_spacing`**, or a heading reads
  as belonging to what sits above it. `SMALL` keeps it so even squeezed to 2
  against 4.
- **`selection_marker_width` is a bar, not an inversion.** `Canvas::draw_text`
  always paints ink, so ink-on-ink would erase the label.
- **`sub_header_height` is a minimum.** `ThemeMetric::SubHeaderHeight` answers
  this or the small face's line height, whichever is larger: fonts are
  swappable, and a 20-pixel face in a 17-pixel band overprints the heading's
  own rule and spills into the content below.
- **`slider_knob_width` and `slider_side_inset` are what `xpui` converts a touch
  to a value with**, so they must be the numbers `draw_slider` paints with.
- **`dialog_width_percent` is the one that is not pixels**, and so the one
  [`scaled`](#metricsscaled) leaves alone.

**Example — a modified copy**

```rust
use xpui_chrome::Metrics;

let roomy = Metrics {
    list_row_height: 48,
    list_row_gap: 6,
    ..Metrics::DEFAULT
};
assert_ne!(roomy, Metrics::DEFAULT);
assert_eq!(Metrics::default(), Metrics::DEFAULT);
```

### Presets

#### `Metrics::DEFAULT`

The defaults every function here uses when a backend supplies nothing.

```text
pub const DEFAULT: Metrics = Metrics
```

For a reader-sized panel of roughly 480x800. Its values are the `DEFAULT` column
above.

#### `Metrics::COMPACT`

For a panel of roughly 320x240 — a Tufty 2040, or any small colour LCD.

```text
pub const COMPACT: Metrics = Metrics
```

Chrome that costs 100 pixels of an 800-pixel panel costs the same 100 of a
240-pixel one, which is 42% of it. Everything shrinks, and touch targets shrink
furthest: a board with five buttons and no touchscreen does not need a 44-pixel
finger target.

#### `Metrics::SMALL`

For a panel of roughly 296x128 — a Badger 2040, or any small e-ink strip.

```text
pub const SMALL: Metrics = Metrics
```

The size where the defaults stop working rather than merely looking cramped: a
40-pixel header, a 40-pixel hint bar and 40-pixel rows leave 28 pixels of
content, and a list refuses to paint a row that does not fit, so the screen
comes back empty. Three rows is the target: a list of two with somewhere to
scroll to.

```rust
use xpui_chrome::Metrics;

assert_eq!(Metrics::DEFAULT.list_rows_for(800), 16);
assert_eq!(Metrics::COMPACT.list_rows_for(240), 5);
assert_eq!(Metrics::SMALL.list_rows_for(128), 3);
assert_eq!(Metrics::DEFAULT.list_rows_for(128), 0, "the defaults leave a badge empty");
```

### Choosing for a device

#### `Metrics::for_panel`

The preset that fits a panel of this size.

```text
pub const fn for_panel(width: i32, height: i32) -> Metrics
```

| Height | Preset |
|---|---|
| 160 or less | `SMALL` |
| 161 to 320 | `COMPACT` |
| above 320 | `DEFAULT` |

Chosen by height, because that is what the chrome eats: a header and a hint bar
cost the same number of pixels on any width, and `width` is not read. The
thresholds are where a preset stops leaving room for three list rows. A backend
with an opinion supplies its own `Metrics` instead: this is a sensible default,
not a rule. [`Labels::for_panel`](#labelsfor_panel) makes the same choice for
the words.

```rust
use xpui_chrome::Metrics;

assert_eq!(Metrics::for_panel(296, 128), Metrics::SMALL);
assert_eq!(Metrics::for_panel(320, 240), Metrics::COMPACT);
assert_eq!(Metrics::for_panel(480, 800), Metrics::DEFAULT);
```

#### `Metrics::for_device`

The whole derivation, in one call, for a caller that knows its device.

```text
pub const fn for_device(width: i32, height: i32, percent: u16, hint_band: bool) -> Metrics
```

| Parameter | Meaning |
|---|---|
| `width`, `height` | The panel, in the orientation it is used in. [`for_panel`](#metricsfor_panel) picks the preset. |
| `percent` | The device's own UI scale, passed to [`scaled`](#metricsscaled). |
| `hint_band` | Whether it has a row of keys worth labelling. `false` applies [`without_button_hints`](#metricswithout_button_hints). |

Here rather than left to each caller because the three steps have to agree:
picking a preset for the panel and then words for something else is how a hint
bar overruns the slot it was measured into. A device that takes Back and Confirm
from a touchscreen reserves nothing at the bottom.

```rust
use xpui_chrome::Metrics;

let touch = Metrics::for_device(480, 800, 120, false);
assert_eq!(touch, Metrics::DEFAULT.scaled(120).without_button_hints());
assert_eq!((touch.list_row_height, touch.button_hints_height), (48, 0));

assert_eq!(Metrics::for_device(296, 128, 100, true), Metrics::SMALL);
```

#### `Metrics::scaled`

The same chrome, sized for a board that asks for larger targets.

```text
pub const fn scaled(&self, percent: u16) -> Metrics
```

| Parameter | Meaning |
|---|---|
| `percent` | A board's UI scale: 100 leaves every number alone, 120 turns a 40-pixel row into 48. |

Every field but `dialog_width_percent` is multiplied and rounded to the nearest
pixel; that one is already a ratio of the panel. A number that was worth a pixel
stays worth one, so a 1-pixel rule never scales to 0, and a 0 stays 0. An
integer percentage because `Metrics` is `Eq` and every preset is `const`, and
because the device targets have no floating-point unit. Words do not scale at
all, which is why `Labels` is a separate thing.

```rust
use xpui_chrome::Metrics;

let large = Metrics::DEFAULT.scaled(120);
assert_eq!(large.list_row_height, 48);
assert_eq!(large.dialog_width_percent, 80);

let tiny = Metrics::DEFAULT.scaled(10);
assert_eq!(tiny.selection_marker_width, 1, "never scaled away");
assert_eq!(Metrics::SMALL.scaled(150).top_padding, 0);
```

#### `Metrics::without_button_hints`

The same chrome with no band reserved for button hints.

```text
pub const fn without_button_hints(&self) -> Metrics
```

For a device whose Back and Confirm come from its touchscreen rather than a row
of keys: a hint names a key, and naming one that is not there sends a person
looking for it. The height is the reservation as well as the drawing, so
[`draw_button_hints`](painting.md#draw_button_hints) draws nothing and the
content gains the space.

```rust
use xpui_chrome::Metrics;

let touch = Metrics::COMPACT.without_button_hints();
assert_eq!(touch.button_hints_height, 0);
assert!(touch.list_rows_for(240) > Metrics::COMPACT.list_rows_for(240));
```

### Reading the content band

#### `Metrics::content_top`

First y below the header that content may use.

```text
pub fn content_top(&self) -> i32
```

`top_padding + header_height + vertical_spacing`.

```rust
use xpui_chrome::Metrics;

assert_eq!(Metrics::DEFAULT.content_top(), 60);
```

#### `Metrics::content_bottom`

First y occupied by the button hints; content must stay above it.

```text
pub fn content_bottom(&self) -> i32
```

Derived from the live panel height rather than stored, so one `Metrics` works on
every panel a backend might be driving. It asks the installed host for the
panel's size.

```rust
use xpui_chrome::Metrics;

xpui::testing::install(); // a 480x800 panel
assert_eq!(Metrics::DEFAULT.content_bottom(), 760);
```

#### `Metrics::list_rows_for`

How many list rows fit in the content band of a panel this tall: the question every preset exists to answer.

```text
pub const fn list_rows_for(&self, panel_height: i32) -> i32
```

The band from [`content_top`](#metricscontent_top) down to the hint bar, divided
into one-line rows and their gaps; the last row needs no gap after it. `0` when
the band has no room. It needs no host, so a board table can check itself in a
`const`. See the [presets](#metricssmall) for each preset's answer.

```rust
use xpui_chrome::Metrics;

const ROWS: i32 = Metrics::SMALL.list_rows_for(128);
assert!(ROWS >= 3);
```

#### `Metrics::metric`

Answers `ThemeMetric`, which is how `xpui` asks for all of this.

```text
pub fn metric(&self, metric: ThemeMetric) -> i32
```

The field of the same name, for every `ThemeMetric` but three:

| `ThemeMetric` | Answer |
|---|---|
| `ContentTop` | [`content_top`](#metricscontent_top) |
| `ContentBottom` | [`content_bottom`](#metricscontent_bottom), which asks the host |
| `SubHeaderHeight` | `sub_header_height`, or the small face's line height when that is larger |

[`plain_chrome!`](plain-chrome.md#plain_chrome) answers `Chrome::metric` with
this. `scrollbar_*`, `slider_track_height`, `selection_marker_width` and the
`dialog_*` fields have no `ThemeMetric`: only this crate's painters read them.

```rust
use xpui::host::ThemeMetric;
use xpui_chrome::Metrics;

xpui::testing::install();
let metrics = Metrics::DEFAULT;
assert_eq!(metrics.metric(ThemeMetric::ListRowHeight), 40);
assert_eq!(metrics.metric(ThemeMetric::ContentTop), 60);

// The fake's small face is 14 pixels tall, so a 10-pixel minimum gives way.
let tight = Metrics { sub_header_height: 10, ..metrics };
assert_eq!(tight.metric(ThemeMetric::SubHeaderHeight), 14);
```

**See also:** [`Labels`](#labels), [painting](painting.md), [`plain_chrome!`](plain-chrome.md#plain_chrome)

## `Labels`

The words on the hint bar: four standard, three for an open value.

```text
pub struct Labels
```

![A hint bar reading Cancel, Done, Up and Down, as a value control shows while it is open](https://raw.githubusercontent.com/XPUI-Framework/xpui-gallery/main/gallery/tests/screenshots/reference/chrome_button_hints_open.png)

Separate from `Metrics` because they are not measurements, and they do not
scale. Separate from the key row because they are not hardware: only the
application knows what language its reader uses. English ships because something
has to, and every one of these words is meant to be replaced by whatever the
application's reader reads. `Labels` is `Copy`, `Eq`, and `Default`, which is
`ENGLISH`.

| Field | | Type |
|---|---|---|
| `Labels::standard_hints` | The four standard hint words, in meaning order — back, confirm, previous, next — for when a screen passes `Hint::Standard`. | `[&'static str; 4]` |
| `Labels::mode_hints` | The words a value control's mode needs, which the four above have no room for: opening one, keeping what it reads, and putting it back. | `[&'static str; 3]` |

Which of the standard words lands over which key is the device's `KeyRow`, not
this. The mode words are separate because the four are chosen by *which key* a
slot sits over, and these by what the framework is doing: no key implies
"Edit". They answer `Hint::Edit`, `Hint::Done` and `Hint::Cancel`, in that
order.

**Example — words of the application's own**

```rust
use xpui::host::{Hint, KeyRow};
use xpui_chrome::{Labels, Metrics, draw_button_hints};

const PORTUGUESE: Labels = Labels {
    standard_hints: ["Voltar", "OK", "Acima", "Abaixo"],
    mode_hints: ["Ajustar", "Pronto", "Cancelar"],
};

# xpui::testing::install();
# xpui::testing::reset();
draw_button_hints(&Metrics::DEFAULT, &PORTUGUESE, &KeyRow::READER, &Hint::Cancel, &Hint::Done, &Hint::Standard, &Hint::Standard);

let text = xpui::testing::drawn_text();
let words: Vec<&str> = text.iter().map(|(_, _, word, _, _)| word.as_str()).collect();
assert_eq!(words, ["Cancelar", "Pronto", "Acima", "Abaixo"]);
```

### English

| Constant | `standard_hints` | `mode_hints` |
|---|---|---|
| `ENGLISH` | Back, Select, Up, Down | Edit, Done, Cancel |
| `ENGLISH_NARROW` | Back, OK, Up, Down | Edit, Done, Cancel |
| `ENGLISH_SHORT` | Back, OK, Up, Dn | Edit, Done, Undo |

#### `Labels::ENGLISH`

English, in the words a full-width band has room for.

```text
pub const ENGLISH: Labels = Labels
```

#### `Labels::ENGLISH_NARROW`

The same, shorter, for a band that has to hold them in less room.

```text
pub const ENGLISH_NARROW: Labels = Labels
```

For a 320-pixel panel whose row is three keys: about 106 pixels a slot, against
120 on a 480-pixel panel with a reader's four.

#### `Labels::ENGLISH_SHORT`

Shorter still, for the narrowest band here: a 296-pixel strip divided by three keys, about 98 pixels a slot.

```text
pub const ENGLISH_SHORT: Labels = Labels
```

Where a word has to give, a shorter word beats a truncated one: `Dn` and `Undo`,
never `Canc…`. Which words need it is a judgement about the face in use, not a
calculation; these are the ones the small preset was built with.

```rust
use xpui_chrome::Labels;

let mut modified = Labels::ENGLISH_SHORT;
modified.mode_hints[2] = "Revert";
assert_eq!(modified.standard_hints, ["Back", "OK", "Up", "Dn"]);
assert_eq!(Labels::default(), Labels::ENGLISH);
```

#### `Labels::for_panel`

The words that go with [`Metrics::for_panel`](#metricsfor_panel).

```text
pub const fn for_panel(width: i32, height: i32) -> Labels
```

The same dispatch, deliberately: a panel small enough to need `Metrics::SMALL`
is small enough to need the words that preset was measured with, and picking one
without the other is how a hint bar ends up overrunning its slot.

| Height | Words |
|---|---|
| 160 or less | `ENGLISH_SHORT` |
| 161 to 320 | `ENGLISH_NARROW` |
| above 320 | `ENGLISH` |

```rust
use xpui_chrome::{Labels, Metrics};

for (width, height) in [(296, 128), (320, 240), (528, 792)] {
    let pair = (Metrics::for_panel(width, height), Labels::for_panel(width, height));
    assert_eq!(pair.0 == Metrics::SMALL, pair.1 == Labels::ENGLISH_SHORT);
}
```

**See also:** [`draw_button_hints`](painting.md#draw_button_hints), [`Metrics`](#metrics)

## Re-exports

`xpui`'s own, re-exported so a backend needs one import for what
[`plain_chrome!`](plain-chrome.md#plain_chrome) asks it for. A key row is a fact
about hardware, not about painting, so it lives in `xpui`, where a board crate
reaches it without this one.

| Name | |
|---|---|
| `KeyRow` | What the keys along a device's bottom edge mean, left to right: `xpui::host::KeyRow`. |
| `RowKey` | The job of one key in that row: `Back`, `Confirm`, `Previous`, `Next`, or `Unassigned`. `xpui::host::RowKey`. |

A key row decides which of the four standard words lands over which key, and
how many slots the hint band divides into. A slot with nothing behind it,
`RowKey::Unassigned`, stays blank, because naming a key the device does not have
sends a person looking for it. `KeyRow::READER` is a reader's four keys, Back
leftmost; `KeyRow::new` takes a device's own, and is `const`, so a board table
can hold one.

![A hint bar for a three-key row reading Back and OK over the first two thirds, and nothing over the third key](https://raw.githubusercontent.com/XPUI-Framework/xpui-gallery/main/gallery/tests/screenshots/reference/chrome_button_hints_keys.png)

```rust
use xpui_chrome::{KeyRow, RowKey};

const BADGE: KeyRow = KeyRow::new(&[RowKey::Back, RowKey::Confirm, RowKey::Unassigned]);

assert_eq!(BADGE.len(), 3);
assert!(!BADGE.contains(RowKey::Next));
assert_eq!(KeyRow::READER.iter().next(), Some(RowKey::Back));
```
