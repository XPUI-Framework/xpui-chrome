# Painting lists

The three themed components `xpui` asks a backend to paint for rows of content:
a list, a dialog of options over it, and the scroll indicator beside a region
taller than its window. Each is a plain function a backend can call itself,
painted from drawing primitives alone. Where a list's rows or a dialog's rows
land is on [layout](layout.md); the header, the controls and the hint bar are on
[painting](painting.md).

| What `xpui` asks for | What paints it |
|---|---|
| `draw_list` | rows, subtitles, values, and the selected-row marker |
| `draw_option_popup` and `option_popup_row_rect` | a centred dialog, and where its rows are |
| `draw_scroll_indicator` | a thumb, and nothing at all when everything fits |

**They paint through the installed host**, like a widget does, not through a
`&self` parameter, so nothing here needs to know which backend called it. Every
example on this page installs `xpui`'s recording fake, a 480x800 panel, and
reads back what was drawn.

[Plain chrome](plain-chrome.md) wires all of these into a backend in one macro.
[The host contract](https://github.com/XPUI-Framework/xpui-framework/blob/main/docs/host.md)
is the trait they answer.

## Topics

| | |
|---|---|
| [`draw_list`](#draw_list) | The themed list, drawing only the rows that fit entirely inside `rect`. |
| [`draw_option_popup`](#draw_option_popup) | A centred dialog: a cleared frame, a border, a title, and its options. |
| [`draw_scroll_indicator`](#draw_scroll_indicator) | A thumb down the right edge, proportional to how much is showing. |

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
[`row_height`](layout.md#row_height) states and `xpui`'s `List` measures with. Rows are
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
of `rect`; [`rows_that_fit`](layout.md#rows_that_fit) says how many rows it drew.

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

**See also:** [`row_height`](layout.md#row_height), [`rows_that_fit`](layout.md#rows_that_fit), [`draw_scroll_indicator`](#draw_scroll_indicator)

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

Where the dialog goes is [`popup_layout`](layout.md#popup_layout)'s answer, and nothing
else's. It clears its frame to paper first, because a dialog sits over content
and its own background is the only thing making it legible; then it draws
`dialog_border` rings, the title, and each option as a one-line list row.

An empty dialog draws nothing: an empty frame looks like a dialog that failed to
load. `xpui`'s `Modal` guards this too, but these functions are public.

> [!NOTE]
> A dialog with more options than the panel is tall stops at the rows that fit,
> and does not scroll. Options at or past [`PopupLayout::visible`](layout.md#popuplayout)
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

**See also:** [`option_popup_row_rect`](layout.md#option_popup_row_rect), [`popup_layout`](layout.md#popup_layout), [`draw_list`](#draw_list)

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
