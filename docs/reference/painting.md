# Painting

The eight themed components `xpui` asks a backend to paint, each a plain
function a backend can call itself, and the layout functions that tell a caller
where a list or a dialog lands. They paint from drawing primitives alone.

![A whole panel painted by these functions: a Settings header reading 12:04, a Display group of three rows with Frontlight marked, a Reading group with a slider and a progress bar, a scroll indicator down the right edge, and Back, Select, Up and Down along the bottom](https://raw.githubusercontent.com/XPUI-Framework/xpui-gallery/main/gallery/tests/screenshots/reference/chrome_overview.png)

| What `xpui` asks for | What paints it |
|---|---|
| `draw_list` | rows, subtitles, values, and the selected-row marker |
| `draw_option_popup` and `option_popup_row_rect` | a centred dialog, and where its rows are |
| `draw_slider` | a dithered track, a fill, and a knob carrying the control's state: paper when idle, dithered once the keys are on it, with the whole control outlined as well when it is open |
| `draw_progress_bar` | an outline and a proportional fill |
| `draw_header` | a title band and its rule |
| `draw_sub_header` | a group heading with a trailing rule |
| `draw_button_hints` | one slot per key along the bottom, each word over the key it names |
| `draw_scroll_indicator` | a thumb, and nothing at all when everything fits |

**They paint through the installed host**, like a widget does, not through a
`&self` parameter. So each paints through whichever backend is installed, which
is the one that called it, and nothing here needs to know what that is. Every
example on this page installs `xpui`'s recording fake, a 480x800 panel, and
reads back what was drawn.

[Plain chrome](plain-chrome.md) wires all of these into a backend in one macro.
[The host contract](https://github.com/XPUI-Framework/xpui-framework/blob/main/docs/host.md)
is the trait they answer.

## Topics

| | |
|---|---|
| [`draw_header`](#draw_header) | The title band, with a rule under it. |
| [`draw_sub_header`](#draw_sub_header) | A group heading: the label, and a rule across the rest of the line. |
| [`draw_list`](#draw_list) | The themed list, drawing only the rows that fit entirely inside `rect`. |
| [`row_height`](#row_height) | The height every row in this list gets. |
| [`rows_that_fit`](#rows_that_fit) | How many of `rows` will actually be painted into `rect`. |
| [`draw_option_popup`](#draw_option_popup) | A centred dialog: a cleared frame, a border, a title, and its options. |
| [`option_popup_row_rect`](#option_popup_row_rect) | Screen rect of one dialog row, for hit-testing. |
| [`popup_layout`](#popup_layout) | Where a dialog titled `title` with `count` options lands on the panel. |
| [`PopupLayout`](#popuplayout) | Where a dialog and its rows land. |
| [`draw_slider`](#draw_slider) | Track, fill and knob — and, when the keys are on it, the knob says so. |
| [`draw_progress_bar`](#draw_progress_bar) | A determinate bar: an outline, filled up to `current`. |
| [`draw_scroll_indicator`](#draw_scroll_indicator) | A thumb down the right edge, proportional to how much is showing. |
| [`draw_button_hints`](#draw_button_hints) | The hints along the bottom, one per key the device says it has. |

## `draw_header`

The title band, with a rule under it.

```text
pub fn draw_header(metrics: &Metrics, title: Option<&str>, subtitle: Option<&str>)
```

![A header band reading Settings on the left in bold and 12:04 on the right in a smaller face, over a rule the width of the panel](https://raw.githubusercontent.com/XPUI-Framework/xpui-gallery/main/gallery/tests/screenshots/reference/chrome_header.png)

| Parameter | Meaning |
|---|---|
| `metrics` | The band sits `top_padding` from the top and is `header_height` tall; the text is inset by `content_side_padding`. |
| `title` | Drawn bold, left-aligned and centred in the band. `None` draws no title. |
| `subtitle` | Drawn in the small face, right-aligned. `None` draws none. |

The band spans the panel's width, and the rule under it is what separates the
header from content on a panel with no colour to separate them with. Text too
long for its room is cut at a whole character and ends in an ellipsis.

The subtitle is drawn first and never takes more than half the band, so a long
one is cut rather than pushing the title off the panel. The title gets what is
left, less `vertical_spacing`.

**Example — a title**

```rust
use xpui::testing::{self, DrawOp};
use xpui::Point;
use xpui_chrome::{Metrics, draw_header};

# testing::install();
# testing::reset();
draw_header(&Metrics::DEFAULT, Some("Settings"), None);

let text = testing::drawn_text();
assert_eq!(text.len(), 1);
assert_eq!((text[0].0, text[0].2.as_str()), (16, "Settings"));
// The rule, along the band's lower edge.
assert!(testing::ops_log().contains(&DrawOp::Line {
    from: Point::new(0, 48),
    to: Point::new(479, 48),
}));
```

**Example — a subtitle too long for the band**

```rust
use xpui::testing;
use xpui_chrome::{Metrics, draw_header};

# testing::install();
# testing::reset();
let long = "Synchronising with the library on the other side of the house";
draw_header(&Metrics::DEFAULT, Some("Settings"), Some(long));

let text = testing::drawn_text();
let words: Vec<&str> = text.iter().map(|(_, _, word, _, _)| word.as_str()).collect();
assert!(words.contains(&"Settings"), "the title survives: {words:?}");
assert!(words.contains(&"…"), "and the subtitle is cut");
```

**See also:** [`draw_sub_header`](#draw_sub_header), [`Metrics::content_top`](metrics-and-labels.md#metricscontent_top)

## `draw_sub_header`

A group heading: the label, and a rule across the rest of the line.

```text
pub fn draw_sub_header(metrics: &Metrics, rect: Rect, label: &str, right: Option<&str>)
```

![A group heading reading Display in bold, a rule across the line after it, and 3 settings at the right end](https://raw.githubusercontent.com/XPUI-Framework/xpui-gallery/main/gallery/tests/screenshots/reference/chrome_sub_header.png)

| Parameter | Meaning |
|---|---|
| `metrics` | `spacing_small` is the gap between the label, the rule and the right-hand text. |
| `rect` | Where the heading goes. The text is drawn from its top; the rule runs along its last row of pixels. |
| `label` | Drawn in the small face, bold. |
| `right` | Drawn in the small face at the right end, never taking more than half of `rect`. `None` lets the rule run to the end. |

The rule starts after the label and runs to the right-hand text or the end of
`rect`, which is what makes a heading read as a divider rather than as a row.
How tall the rect should be is `ThemeMetric::SubHeaderHeight`, which
[`Metrics::metric`](metrics-and-labels.md#metricsmetric) answers with
`sub_header_height` or the small face's line height, whichever is larger.

**Example — a heading with a count**

```rust
use xpui::testing::{self, DrawOp};
use xpui::{Point, Rect};
use xpui_chrome::{Metrics, draw_sub_header};

# testing::install();
# testing::reset();
draw_sub_header(&Metrics::DEFAULT, Rect::new(16, 60, 448, 17), "Display", Some("3 items"));

let text = testing::drawn_text();
let placed: Vec<(i32, &str)> = text.iter().map(|(x, _, word, _, _)| (*x, word.as_str())).collect();
// The fake's small face is 5 pixels a character.
assert_eq!(placed, [(429, "3 items"), (16, "Display")]);
assert!(testing::ops_log().contains(&DrawOp::Line {
    from: Point::new(55, 76),
    to: Point::new(424, 76),
}));
```

**See also:** [`draw_header`](#draw_header), [`draw_list`](#draw_list)

## `draw_list`

The themed list, drawing only the rows that fit entirely inside `rect`.

```text
pub fn draw_list<'a>(
    metrics: &Metrics,
    rect: Rect,
    rows: usize,
    selected: i32,
    row: &dyn Fn(usize, RowField) -> Option<&'a str>,
)
```

![Three rows, Frontlight reading On, Sleep after reading 5 min and Free heap reading 182 KB, with Sleep after marked by a bar down its leading edge and an outline](https://raw.githubusercontent.com/XPUI-Framework/xpui-gallery/main/gallery/tests/screenshots/reference/chrome_list.png)

| Parameter | Meaning |
|---|---|
| `metrics` | Row height, the gap between rows, the marker's width, and `spacing_small` either side of the text. |
| `rect` | The list's area. Rows are laid from its top, as wide as it is. |
| `rows` | How many rows there are. |
| `selected` | The row to mark, zero-based. Any number that is not a row, `-1` included, marks none. |
| `row` | Called with a row's index and a `RowField`; `None` omits that field. |

Every row is the same height: `list_row_height`, or
`list_row_height_with_subtitle` if **any** row has a subtitle, the rule
[`row_height`](#row_height) states and `xpui`'s `List` measures with. Rows are
`list_row_gap` apart. The title is drawn left in the interface face, a subtitle
under it in the small face with the pair centred as a block, and a value
right-aligned in the interface face. The value is placed first, and the title
gets the room left beside it; either is cut with an ellipsis when it runs out.

**Selection is a marker, not an inversion.** Text is always ink, so filling the
row black would erase its label. The selected row gets a bar
`selection_marker_width` wide down its leading edge and an outline around it.

**A row that would be cut is not drawn.** A partly visible row reads as a
rendering fault rather than as "there is more below", which is what the scroll
indicator is for. So a list can leave up to a row's stride empty at the bottom
of `rect`; [`rows_that_fit`](#rows_that_fit) says how many rows it drew.

![Two titles, Middlemarch and The Odyssey, inside an outlined rect with room for most of a third row, left empty](https://raw.githubusercontent.com/XPUI-Framework/xpui-gallery/main/gallery/tests/screenshots/reference/chrome_list_fit.png)

**Example — a settings list**

```rust
use xpui::host::RowField;
use xpui::testing::{self, RectKind};
use xpui::Rect;
use xpui_chrome::{Metrics, draw_list};

const ROWS: [(&str, &str); 3] = [("Frontlight", "On"), ("Sleep after", "5 min"), ("Free heap", "182 KB")];

let row = |index: usize, field: RowField| {
    let (title, value) = ROWS.get(index)?;
    match field {
        RowField::Title => Some(*title),
        RowField::Value => Some(*value),
        RowField::Subtitle => None,
    }
};

# testing::install();
# testing::reset();
draw_list(&Metrics::DEFAULT, Rect::new(16, 60, 448, 400), ROWS.len(), 0, &row);

let text = testing::drawn_text();
let words: Vec<&str> = text.iter().map(|(_, _, word, _, _)| word.as_str()).collect();
assert_eq!(words, ["On", "Frontlight", "5 min", "Sleep after", "182 KB", "Free heap"]);

// The marker and the outline, on the first row only.
let rects = testing::drawn_rects();
assert_eq!(rects, [(16, 60, 4, 40, RectKind::Filled), (16, 60, 448, 40, RectKind::Stroked)]);
```

**Example — a row that does not fit**

```rust
use xpui::host::RowField;
use xpui::testing;
use xpui::Rect;
use xpui_chrome::{Metrics, draw_list};

let titles = ["One", "Two", "Three"];
let row = |index: usize, field: RowField| match field {
    RowField::Title => titles.get(index).copied(),
    _ => None,
};

# testing::install();
# testing::reset();
// Two 40-pixel rows 4 apart take 84 pixels; a third would end at 128.
draw_list(&Metrics::DEFAULT, Rect::new(0, 0, 400, 100), titles.len(), -1, &row);

let text = testing::drawn_text();
let words: Vec<&str> = text.iter().map(|(_, _, word, _, _)| word.as_str()).collect();
assert_eq!(words, ["One", "Two"]);
```

**See also:** [`row_height`](#row_height), [`rows_that_fit`](#rows_that_fit), [`draw_scroll_indicator`](#draw_scroll_indicator)

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

**See also:** [`draw_list`](#draw_list), [`rows_that_fit`](#rows_that_fit)

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

[`draw_list`](#draw_list) stops before a row that does not fit, so the space it
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

**See also:** [`draw_list`](#draw_list), [`Metrics::list_rows_for`](metrics-and-labels.md#metricslist_rows_for)

## `draw_option_popup`

A centred dialog: a cleared frame, a border, a title, and its options.

```text
pub fn draw_option_popup<'a>(
    metrics: &Metrics,
    title: &str,
    options: &dyn Fn(usize) -> Option<&'a str>,
    count: usize,
    selected: i32,
)
```

![A settings list with a header and hint bar, and over it a centred dialog titled Sleep after offering 1 min, 5 min, marked, 15 min and Never](https://raw.githubusercontent.com/XPUI-Framework/xpui-gallery/main/gallery/tests/screenshots/reference/chrome_option_popup.png)

| Parameter | Meaning |
|---|---|
| `metrics` | `dialog_border`, `dialog_padding`, `dialog_width_percent`, and `list_row_height` for each option. |
| `title` | Drawn bold above the options. An empty title takes no room. |
| `options` | Called with each index below the visible count; `None` leaves that row blank. |
| `count` | How many options there are. `0` draws nothing. |
| `selected` | The option to mark, as a list row is marked. Any number that is not a visible row marks none. |

Where the dialog goes is [`popup_layout`](#popup_layout)'s answer, and nothing
else's. It clears its frame to paper first, because a dialog sits over content
and its own background is the only thing making it legible; then it draws
`dialog_border` rings, the title, and each option as a one-line list row.

An empty dialog draws nothing: an empty frame looks like a dialog that failed to
load. `xpui`'s `Modal` guards this too, but these functions are public.

> [!NOTE]
> A dialog with more options than the panel is tall stops at the rows that fit,
> and does not scroll. Options at or past [`PopupLayout::visible`](#popuplayout)
> are neither painted nor given a rect, so a `selected` there marks nothing.

![A dialog titled Language as tall as the panel, showing the first eighteen of twenty-four languages with Cymraeg marked, and no more](https://raw.githubusercontent.com/XPUI-Framework/xpui-gallery/main/gallery/tests/screenshots/reference/chrome_popup_clamped.png)

**Example — a picker**

```rust
use xpui::testing::{self, RectKind};
use xpui_chrome::{Metrics, draw_option_popup};

const FACES: [&str; 3] = ["Serif", "Sans", "Mono"];

# testing::install();
# testing::reset();
draw_option_popup(&Metrics::DEFAULT, "Typeface", &|index| FACES.get(index).copied(), FACES.len(), 1);

// First the cleared frame, then the two rings of its border.
let rects = testing::drawn_rects();
assert_eq!(rects[0], (48, 311, 384, 177, RectKind::Filled));
assert_eq!(rects[1], (48, 311, 384, 177, RectKind::Stroked));
assert_eq!(rects[2], (49, 312, 382, 175, RectKind::Stroked));

let text = testing::drawn_text();
let words: Vec<&str> = text.iter().map(|(_, _, word, _, _)| word.as_str()).collect();
assert_eq!(words, ["Typeface", "Serif", "Sans", "Mono"]);
```

**Example — no options, no dialog**

```rust
use xpui::testing;
use xpui_chrome::{Metrics, draw_option_popup};

# testing::install();
# testing::reset();
draw_option_popup(&Metrics::DEFAULT, "Typeface", &|_| None, 0, 0);
assert!(testing::ops_log().is_empty());
```

**See also:** [`option_popup_row_rect`](#option_popup_row_rect), [`popup_layout`](#popup_layout), [`draw_list`](#draw_list)

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
| `metrics`, `title`, `count` | The same as the [`draw_option_popup`](#draw_option_popup) call being hit-tested. |
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

**See also:** [`draw_option_popup`](#draw_option_popup), [`popup_layout`](#popup_layout)

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

**See also:** [`PopupLayout`](#popuplayout), [`draw_option_popup`](#draw_option_popup), [`option_popup_row_rect`](#option_popup_row_rect)

## `PopupLayout`

Where a dialog and its rows land.

```text
pub struct PopupLayout
```

![A settings list with a header and hint bar, and over it a centred dialog titled Sleep after offering 1 min, 5 min, marked, 15 min and Never](https://raw.githubusercontent.com/XPUI-Framework/xpui-gallery/main/gallery/tests/screenshots/reference/chrome_option_popup.png)

What [`popup_layout`](#popup_layout) answers. Both
[`draw_option_popup`](#draw_option_popup) and
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

## `draw_slider`

Track, fill and knob — and, when the keys are on it, the knob says so.

```text
pub fn draw_slider(metrics: &Metrics, rect: Rect, value: i32, max: i32, state: ControlState)
```

![Three sliders at the same value: idle with a paper knob, focused with a dithered knob, and open with a dithered knob and an outline around the whole control](https://raw.githubusercontent.com/XPUI-Framework/xpui-gallery/main/gallery/tests/screenshots/reference/chrome_slider.png)

| Parameter | Meaning |
|---|---|
| `metrics` | `slider_side_inset`, `slider_track_height`, `slider_knob_width` and `slider_knob_height`. |
| `rect` | The whole control. The track runs across its middle, inset at each end. |
| `value` | Where the knob sits, clamped to `0..=max`. |
| `max` | The value at the right end. `0` or less draws nothing: no range, no position to show. |
| `state` | What the keys will do to it next, one of the three `ControlState`s below. |

The track is dithered rather than solid, because on one bit a solid track is
indistinguishable from the filled part of it. The fill runs to the knob's centre.

**The knob carries the focus mark**, not the track. Three states have to be told
apart on one bit of colour, and the knob is the one part still painted paper:

| State | The knob | The control |
|---|---|---|
| `Idle` | paper, outlined | as it is |
| `Focused` | dithered, outlined | as it is |
| `Editing` | dithered, outlined | outlined as well |

The knob's shade already says the keys are here; the outline around an open
control says they are *moving the value*. On a device with no Left/Right pair
that outline is the only sign that Previous and Next now move the value.
[design.md](../design.md#the-knob-carries-the-focus-mark) says why no other mark
would do.

The knob's position uses the same numbers `xpui` converts a touch to a value
with, `slider_side_inset` and `slider_knob_width`, so the two agree. It stays
inside `rect` even when `rect` is narrower or shorter than the knob.

**Example — each state**

```rust
use xpui::host::ControlState;
use xpui::testing::{self, RectKind};
use xpui::Rect;
use xpui_chrome::{Metrics, draw_slider};

let rect = Rect::new(16, 100, 448, 40);
# testing::install();

testing::reset();
draw_slider(&Metrics::DEFAULT, rect, 25, 100, ControlState::Idle);
assert_eq!(
    testing::drawn_rects(),
    [
        (24, 117, 432, 6, RectKind::Dither),  // the track
        (24, 117, 111, 6, RectKind::Filled),  // the fill, to the knob's centre
        (128, 109, 14, 22, RectKind::Filled), // the knob, cleared to paper
        (128, 109, 14, 22, RectKind::Stroked),
    ]
);

testing::reset();
draw_slider(&Metrics::DEFAULT, rect, 25, 100, ControlState::Editing);
let rects = testing::drawn_rects();
assert_eq!(rects[2], (128, 109, 14, 22, RectKind::Dither));
assert_eq!(rects.last(), Some(&(16, 100, 448, 40, RectKind::Stroked)));
```

**Example — no range**

```rust
use xpui::host::ControlState;
use xpui::testing;
use xpui::Rect;
use xpui_chrome::{Metrics, draw_slider};

# testing::install();
# testing::reset();
draw_slider(&Metrics::DEFAULT, Rect::new(16, 100, 448, 40), 5, 0, ControlState::Focused);
assert!(testing::ops_log().is_empty());
```

**See also:** [`draw_progress_bar`](#draw_progress_bar), [`Metrics::slider_knob_width`](metrics-and-labels.md#metrics)

## `draw_progress_bar`

A determinate bar: an outline, filled up to `current`.

```text
pub fn draw_progress_bar(_metrics: &Metrics, rect: Rect, current: u32, total: u32)
```

![Four progress bars: empty, about a third filled, full, and a taller one about a third filled](https://raw.githubusercontent.com/XPUI-Framework/xpui-gallery/main/gallery/tests/screenshots/reference/chrome_progress_bar.png)

| Parameter | Meaning |
|---|---|
| `_metrics` | Not read: the bar is as tall as `rect`. `ThemeMetric::ProgressBarHeight` is how `xpui` asks how tall to make it. |
| `rect` | The bar, outline included. |
| `current` | How far along, clamped to `total`. |
| `total` | The whole. `0` draws the outline alone. |

The fill sits one pixel inside the outline, and is
`(width - 2) * current / total` pixels wide, rounded down. A rect two pixels
wide or less has no inside, and draws the outline alone.

**Example — a quarter of the way**

```rust
use xpui::testing::{self, RectKind};
use xpui::Rect;
use xpui_chrome::{Metrics, draw_progress_bar};

# testing::install();
# testing::reset();
draw_progress_bar(&Metrics::DEFAULT, Rect::new(16, 100, 202, 6), 85, 340);
assert_eq!(
    testing::drawn_rects(),
    [(16, 100, 202, 6, RectKind::Stroked), (17, 101, 50, 4, RectKind::Filled)]
);
```

**Example — nothing to measure against**

```rust
use xpui::testing::{self, RectKind};
use xpui::Rect;
use xpui_chrome::{Metrics, draw_progress_bar};

# testing::install();
# testing::reset();
draw_progress_bar(&Metrics::DEFAULT, Rect::new(16, 100, 202, 6), 12, 0);
assert_eq!(testing::drawn_rects(), [(16, 100, 202, 6, RectKind::Stroked)]);
```

**See also:** [`draw_slider`](#draw_slider)

## `draw_scroll_indicator`

A thumb down the right edge, proportional to how much is showing.

```text
pub fn draw_scroll_indicator(
    metrics: &Metrics,
    rect: Rect,
    content: i32,
    visible: i32,
    offset: i32,
)
```

![Four book titles with the first marked, and a dithered track down the right edge with a solid thumb a little way down it](https://raw.githubusercontent.com/XPUI-Framework/xpui-gallery/main/gallery/tests/screenshots/reference/chrome_scroll_indicator.png)

| Parameter | Meaning |
|---|---|
| `metrics` | `scrollbar_width`, `scrollbar_inset` from the right edge of `rect`, and a thumb at least a quarter of `min_touch_size` long. |
| `rect` | The scrolling region. The track runs down its whole height. |
| `content` | How tall everything is. |
| `visible` | How much of it the region shows. |
| `offset` | How far down the region is scrolled, clamped to `content - visible`. |

It draws nothing when everything already fits, or when `visible` or `content` is
not positive: a full-height bar tells the reader there is more to see when there
is not. Otherwise a dithered track, and over it a solid thumb
`height * visible / content` long that reaches the bottom of the track at the
largest offset.

The thumb is never shorter than a quarter of `min_touch_size`, so a very long
page still shows something, and never longer than the track. `xpui` draws the
indicator after its scroll view lifts the clip, so the region's right edge is
where it lands, not the panel's.

**Example — the top and the bottom of a page twice the window**

```rust
use xpui::testing::{self, RectKind};
use xpui::Rect;
use xpui_chrome::{Metrics, draw_scroll_indicator};

let window = Rect::new(0, 60, 480, 700);
# testing::install();

testing::reset();
draw_scroll_indicator(&Metrics::DEFAULT, window, 1400, 700, 0);
assert_eq!(
    testing::drawn_rects(),
    [(474, 60, 4, 700, RectKind::Dither), (474, 60, 4, 350, RectKind::Filled)]
);

testing::reset();
draw_scroll_indicator(&Metrics::DEFAULT, window, 1400, 700, 700);
assert_eq!(testing::drawn_rects()[1], (474, 410, 4, 350, RectKind::Filled));
```

**Example — everything fits**

```rust
use xpui::testing;
use xpui::Rect;
use xpui_chrome::{Metrics, draw_scroll_indicator};

# testing::install();
# testing::reset();
draw_scroll_indicator(&Metrics::DEFAULT, Rect::new(0, 60, 480, 700), 300, 700, 0);
assert!(testing::ops_log().is_empty());
```

**See also:** [`draw_list`](#draw_list)

## `draw_button_hints`

The hints along the bottom, one per key the device says it has.

```text
pub fn draw_button_hints(
    metrics: &Metrics,
    labels: &Labels,
    keys: &KeyRow,
    back: &Hint,
    confirm: &Hint,
    previous: &Hint,
    next: &Hint,
)
```

![A hint bar under a rule reading Back, Select, Up and Down, each centred in a quarter of the width](https://raw.githubusercontent.com/XPUI-Framework/xpui-gallery/main/gallery/tests/screenshots/reference/chrome_button_hints.png)

| Parameter | Meaning |
|---|---|
| `metrics` | The band is `button_hints_height` tall along the bottom of the panel. `0` or less draws nothing. |
| `labels` | The words a `Hint` that is not text resolves to. See [`Labels`](metrics-and-labels.md#labels). |
| `keys` | The device's key row, left to right: how many slots, and which job each has. See [`KeyRow`](metrics-and-labels.md#re-exports). |
| `back`, `confirm`, `previous`, `next` | What the screen says about each key, by meaning. |

Three things, because three parties own them: the band's size is the caller's
`Metrics`, the words are its `Labels`, and which word sits over which key is the
device's `KeyRow`.

**The band is divided by the keys the device has**, the length of its row. Each
slot is labelled for its key's job, never its position: a row that puts Confirm
first gets Confirm's hint first, and a row that omits Back does not shift Back's
hint onto Confirm's key. A slot whose key is `RowKey::Unassigned`, and a job the
row has no key for, stay blank, because naming a key the device does not have
sends a person looking for it.

![A hint bar for a three-key row reading Back and OK over the first two thirds, and nothing over the third key](https://raw.githubusercontent.com/XPUI-Framework/xpui-gallery/main/gallery/tests/screenshots/reference/chrome_button_hints_keys.png)

What each hint draws:

| `Hint` | Draws |
|---|---|
| `Standard` | the slot's own word from `labels.standard_hints`: back, confirm, previous, next |
| `Edit` | `labels.mode_hints[0]` |
| `Done` | `labels.mode_hints[1]` |
| `Cancel` | `labels.mode_hints[2]` |
| `Text(label)` | `label`, as the screen gave it |
| `None` | nothing: the slot is blank |

Each word is centred in its slot in the small face, and cut with an ellipsis
when it does not fit. A rule above the band matches the one under the header.
A device whose keys are not a row along the bottom reserves no band, and
[`Metrics::without_button_hints`](metrics-and-labels.md#metricswithout_button_hints)
is how its metrics say so.

![A hint bar reading Cancel, Done, Up and Down, as a value control shows while it is open](https://raw.githubusercontent.com/XPUI-Framework/xpui-gallery/main/gallery/tests/screenshots/reference/chrome_button_hints_open.png)

**Example — a reader's four keys**

```rust
use xpui::host::{Hint, KeyRow};
use xpui::testing;
use xpui_chrome::{Labels, Metrics, draw_button_hints};

# testing::install();
# testing::reset();
let standard = Hint::Standard;
draw_button_hints(&Metrics::DEFAULT, &Labels::ENGLISH, &KeyRow::READER, &standard, &standard, &standard, &standard);

// Four 120-pixel slots across 480, each word centred; the small face is 5 pixels a character.
let text = testing::drawn_text();
let placed: Vec<(i32, &str)> = text.iter().map(|(x, _, word, _, _)| (*x, word.as_str())).collect();
assert_eq!(placed, [(50, "Back"), (165, "Select"), (295, "Up"), (410, "Down")]);
```

**Example — a value control is open**

```rust
use xpui::host::{Hint, KeyRow};
use xpui::testing;
use xpui_chrome::{Labels, Metrics, draw_button_hints};

# testing::install();
# testing::reset();
draw_button_hints(
    &Metrics::DEFAULT,
    &Labels::ENGLISH,
    &KeyRow::READER,
    &Hint::Cancel,
    &Hint::Done,
    &Hint::None,
    &Hint::text("Undo"),
);

let text = testing::drawn_text();
let words: Vec<&str> = text.iter().map(|(_, _, word, _, _)| word.as_str()).collect();
assert_eq!(words, ["Cancel", "Done", "Undo"]);
```

**Example — a row of three, Confirm first**

```rust
use xpui::host::{Hint, KeyRow, RowKey};
use xpui::testing;
use xpui_chrome::{Labels, Metrics, draw_button_hints};

const BADGE: KeyRow = KeyRow::new(&[RowKey::Confirm, RowKey::Back, RowKey::Unassigned]);

# testing::install();
# testing::reset();
let standard = Hint::Standard;
draw_button_hints(&Metrics::DEFAULT, &Labels::ENGLISH_NARROW, &BADGE, &standard, &standard, &standard, &standard);

// Three 160-pixel slots: OK over the first key, Back over the second, and no
// slot for Previous or Next.
let text = testing::drawn_text();
let placed: Vec<(i32, &str)> = text.iter().map(|(x, _, word, _, _)| (*x, word.as_str())).collect();
assert_eq!(placed, [(75, "OK"), (230, "Back")]);
```

**Example — no band**

```rust
use xpui::host::{Hint, KeyRow};
use xpui::testing;
use xpui_chrome::{Labels, Metrics, draw_button_hints};

# testing::install();
# testing::reset();
let touch = Metrics::DEFAULT.without_button_hints();
let standard = Hint::Standard;
draw_button_hints(&touch, &Labels::ENGLISH, &KeyRow::READER, &standard, &standard, &standard, &standard);
assert!(testing::ops_log().is_empty());
```

**See also:** [`Labels`](metrics-and-labels.md#labels), [`KeyRow`](metrics-and-labels.md#re-exports), [`plain_chrome!`](plain-chrome.md#plain_chrome)
