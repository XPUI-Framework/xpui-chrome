# Row and dialog layout

Where a list's rows and a dialog's rows land, answered without painting
anything. [`draw_list`](painting-lists.md#draw_list) and
[`draw_option_popup`](painting-lists.md#draw_option_popup) paint from these same
answers, so a caller that measures a list, or hit-tests a dialog, agrees with
what was painted to the pixel.

![A settings list with a header and hint bar, and over it a centred dialog titled Sleep after offering 1 min, 5 min, marked, 15 min and Never](https://raw.githubusercontent.com/XPUI-Framework/xpui-gallery/main/gallery/tests/screenshots/reference/chrome_option_popup.png)

| What a caller asks | What answers it |
|---|---|
| how tall each row of a list is | `row_height` |
| how many rows of a list are painted | `rows_that_fit` |
| where a dialog and its rows land | `popup_layout`, answering a `PopupLayout` |
| which dialog row a tap is on | `option_popup_row_rect` |

The two list functions need no host. The dialog functions read the panel's size
and measure the title through the installed host, so their examples install
`xpui`'s recording fake, a 480x800 panel.

[Painting lists](painting-lists.md) is what draws from these answers.
[Plain chrome](plain-chrome.md) wires `option_popup_row_rect` into a backend's
`Chrome` implementation.

## Topics

| | |
|---|---|
| [`row_height`](#row_height) | The height every row in this list gets. |
| [`rows_that_fit`](#rows_that_fit) | How many of `rows` will actually be painted into `rect`. |
| [`popup_layout`](#popup_layout) | Where a dialog titled `title` with `count` options lands on the panel. |
| [`PopupLayout`](#popuplayout) | Where a dialog and its rows land. |
| [`option_popup_row_rect`](#option_popup_row_rect) | Screen rect of one dialog row, for hit-testing. |

## `row_height`

The height every row in this list gets.

```text
pub fn row_height<'a>(
    metrics: &Metrics,
    rows: usize,
    row: &dyn Fn(usize, RowField) -> Option<&'a str>,
) -> i32
```

![Two taller rows, Wi-Fi over Home network reading On and marked, and Bluetooth over No devices reading Off](https://raw.githubusercontent.com/XPUI-Framework/xpui-gallery/main/gallery/tests/screenshots/reference/chrome_list_subtitle.png)

`list_row_height_with_subtitle` when any of the `rows` answers
`RowField::Subtitle`, and `list_row_height` otherwise. Uniform across the list,
and the same rule `xpui`'s `List` measures with, so the two agree about how tall
a list is. It asks `row` only for subtitles, and needs no host.

**Example — one subtitle makes every row taller**

```rust
use xpui::host::RowField;
use xpui_chrome::{Metrics, row_height};

let plain = |_: usize, field: RowField| match field {
    RowField::Title => Some("Wi-Fi"),
    _ => None,
};
let second_has_one = |index: usize, field: RowField| match field {
    RowField::Title => Some("Wi-Fi"),
    RowField::Subtitle if index == 1 => Some("Home network"),
    _ => None,
};

assert_eq!(row_height(&Metrics::DEFAULT, 3, &plain), 40);
assert_eq!(row_height(&Metrics::DEFAULT, 3, &second_has_one), 56);
```

**See also:** [`draw_list`](painting-lists.md#draw_list), [`rows_that_fit`](#rows_that_fit)

## `rows_that_fit`

How many of `rows` will actually be painted into `rect`.

```text
pub fn rows_that_fit<'a>(
    metrics: &Metrics,
    rect: Rect,
    rows: usize,
    row: &dyn Fn(usize, RowField) -> Option<&'a str>,
) -> usize
```

![Two titles, Middlemarch and The Odyssey, inside an outlined rect with room for most of a third row, left empty](https://raw.githubusercontent.com/XPUI-Framework/xpui-gallery/main/gallery/tests/screenshots/reference/chrome_list_fit.png)

[`draw_list`](painting-lists.md#draw_list) stops before a row that does not fit, so the space it
leaves can be up to one row's stride. A caller that needs to know where the list
really ends has to ask rather than divide. It takes the same arguments as
`draw_list`, less `selected`, and draws nothing.

**Example — where the list ends**

```rust
use xpui::host::RowField;
use xpui::Rect;
use xpui_chrome::{Metrics, row_height, rows_that_fit};

let row = |_: usize, field: RowField| match field {
    RowField::Title => Some("A book"),
    _ => None,
};
let metrics = Metrics::DEFAULT;
let rect = Rect::new(0, 60, 400, 100);

let drawn = rows_that_fit(&metrics, rect, 5, &row);
let height = row_height(&metrics, 5, &row);
let bottom = rect.y() + drawn as i32 * (height + metrics.list_row_gap) - metrics.list_row_gap;

assert_eq!(drawn, 2);
assert_eq!(bottom, 144, "16 pixels short of the rect's own bottom, 160");
```

**See also:** [`draw_list`](painting-lists.md#draw_list), [`Metrics::list_rows_for`](metrics-and-labels.md#metricslist_rows_for)

## `popup_layout`

Where a dialog titled `title` with `count` options lands on the panel.

```text
pub fn popup_layout(metrics: &Metrics, title: &str, count: usize) -> PopupLayout
```

![A dialog titled Language as tall as the panel, showing the first eighteen of twenty-four languages with Cymraeg marked, and no more](https://raw.githubusercontent.com/XPUI-Framework/xpui-gallery/main/gallery/tests/screenshots/reference/chrome_popup_clamped.png)

The dialog is `dialog_width_percent` of the panel's width, centred. Its height is
two borders, two paddings, a title band of the bold face's line height plus
`dialog_padding` (none for an empty title), and `list_row_height` for each
visible row. It reads the panel's size from the installed host, and measures the
title's line through it.

The height is capped by the panel: never taller than it, and never fewer than
one row, since a dialog showing nothing is no improvement on one showing too
much. A twenty-entry language picker reaches the cap on a 480x800 panel.

**Example — a three-option dialog on 480x800**

```rust
use xpui::Rect;
use xpui_chrome::{Metrics, popup_layout};

# xpui::testing::install();
let layout = popup_layout(&Metrics::DEFAULT, "Typeface", 3);

assert_eq!(layout.frame, Rect::new(48, 311, 384, 177));
assert_eq!(layout.first_row, Rect::new(62, 354, 356, 40));
assert_eq!((layout.stride, layout.visible), (40, 3));
```

**Example — more options than the panel holds**

```rust
use xpui_chrome::{Metrics, popup_layout};

# xpui::testing::install();
let layout = popup_layout(&Metrics::DEFAULT, "Language", 30);
assert_eq!(layout.visible, 18);
assert!(layout.frame.height() <= 800);
```

**See also:** [`PopupLayout`](#popuplayout), [`draw_option_popup`](painting-lists.md#draw_option_popup), [`option_popup_row_rect`](#option_popup_row_rect)

## `PopupLayout`

Where a dialog and its rows land.

```text
pub struct PopupLayout
```

![A settings list with a header and hint bar, and over it a centred dialog titled Sleep after offering 1 min, 5 min, marked, 15 min and Never](https://raw.githubusercontent.com/XPUI-Framework/xpui-gallery/main/gallery/tests/screenshots/reference/chrome_option_popup.png)

What [`popup_layout`](#popup_layout) answers. Both
[`draw_option_popup`](painting-lists.md#draw_option_popup) and
[`option_popup_row_rect`](#option_popup_row_rect) go through it, so the rows are
painted where they answer taps. It is `Copy` and `Eq`.

| Field | | Type |
|---|---|---|
| `PopupLayout::frame` | The dialog's outer rectangle, border included. | `Rect` |
| `PopupLayout::first_row` | The first option's row; the rest follow at `stride`. | `Rect` |
| `PopupLayout::stride` | From one row's top to the next. | `i32` |
| `PopupLayout::visible` | How many rows actually fit on the panel. | `usize` |

Row `n` is `first_row` moved down by `stride * n`, for each `n` below `visible`.
A dialog with more options than the panel is tall is drawn as tall as it fits
and no taller, rather than running its frame off the bottom edge.

**Example — row by row**

```rust
use xpui::Rect;
use xpui_chrome::{Metrics, option_popup_row_rect, popup_layout};

# xpui::testing::install();
let layout = popup_layout(&Metrics::DEFAULT, "Typeface", 3);
let options = |_: usize| Some("A face");
for index in 0..layout.visible {
    let row = layout.first_row;
    let expected = Rect::new(row.x(), row.y() + layout.stride * index as i32, row.width(), row.height());
    assert_eq!(option_popup_row_rect(&Metrics::DEFAULT, "Typeface", &options, 3, index), Some(expected));
}
```

**See also:** [`popup_layout`](#popup_layout)

## `option_popup_row_rect`

Screen rect of one dialog row, for hit-testing.

```text
pub fn option_popup_row_rect<'a>(
    metrics: &Metrics,
    title: &str,
    _options: &dyn Fn(usize) -> Option<&'a str>,
    count: usize,
    index: usize,
) -> Option<Rect>
```

![A settings list with a header and hint bar, and over it a centred dialog titled Sleep after offering 1 min, 5 min, marked, 15 min and Never](https://raw.githubusercontent.com/XPUI-Framework/xpui-gallery/main/gallery/tests/screenshots/reference/chrome_option_popup.png)

| Parameter | Meaning |
|---|---|
| `metrics`, `title`, `count` | The same as the [`draw_option_popup`](painting-lists.md#draw_option_popup) call being hit-tested. |
| `_options` | Not read. It is there because `Chrome::option_popup_row_rect` passes it. |
| `index` | The option, zero-based. |

`None` when `index` is past the last option, or past the last row that fits on
the panel, which is not painted and so not touchable.

**One layout function serves the dialog's painter and its hit-test.** The rect
comes from [`popup_layout`](#popup_layout), which `draw_option_popup` reads as
well. Deriving that geometry twice is how a dialog ends up painting row 3 where
row 2 responds to a touch, and nothing about that failure looks wrong until
somebody taps it. The rect is exactly the one a selected row is outlined in.

**Example — which option a tap chose**

```rust
use xpui::Point;
use xpui_chrome::{Metrics, option_popup_row_rect};

const FACES: [&str; 3] = ["Serif", "Sans", "Mono"];
let options = |index: usize| FACES.get(index).copied();

# xpui::testing::install();
let tap = Point::new(100, 410);
let chosen = (0..FACES.len()).find(|&index| {
    option_popup_row_rect(&Metrics::DEFAULT, "Typeface", &options, FACES.len(), index)
        .is_some_and(|row| row.contains(tap))
});
assert_eq!(chosen, Some(1));
assert_eq!(option_popup_row_rect(&Metrics::DEFAULT, "Typeface", &options, FACES.len(), 3), None);
```

**See also:** [`draw_option_popup`](painting-lists.md#draw_option_popup), [`popup_layout`](#popup_layout)
