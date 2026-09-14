# Icons

Twelve icons drawn from lines and rectangles, for a backend with no asset set.
A backend on a drawing library has no bitmaps to answer `Canvas::draw_icon`
with, so an icon leaves a hole, and an `icon_size` of 0 means the layout does
not even reserve space for it. These are drawn, not stored: no bitmaps, no font,
nothing in flash, and at 16 to 32 pixels on a 1-bit panel a drawn glyph and a
stored one look much the same.

![The twelve icons in two rows, outlined above and solid below: sun, moon, gear, four chevrons, check, cross, battery, wifi and book](https://raw.githubusercontent.com/XPUI-Framework/xpui-gallery/main/gallery/tests/screenshots/reference/chrome_icons.png)

They are not one of the eight components: `xpui` asks for an icon by kind,
through the host's `Canvas`, and `IconRef::kind` is opaque to the framework: the
host decides what 3 means. `Icon` is this crate's answer. A backend that ships
real assets maps the same names onto those instead.

## Topics

| | |
|---|---|
| [`Icon`](#icon) | The icons this crate can draw. |
| [`draw_icon`](#draw_icon) | Draws `icon` with its top-left corner at `origin`. |
| [`icon_size`](#icon_size) | The edge length this crate would actually draw, or 0 if it would draw nothing — which is how the framework knows to reserve no space. |

## `Icon`

The icons this crate can draw.

```text
pub enum Icon
```

![The twelve icons in two rows, outlined above and solid below: sun, moon, gear, four chevrons, check, cross, battery, wifi and book](https://raw.githubusercontent.com/XPUI-Framework/xpui-gallery/main/gallery/tests/screenshots/reference/chrome_icons.png)

`#[repr(u16)]`, so a screen can write `Icon::Sun` and the number that crosses to
the backend is this enum's discriminant: the mapping is the enum, rather than a
table somewhere that has to agree with one. `Icon` converts into `IconRef`, in
its first variant at a 32-pixel edge, so it goes wherever `xpui` takes an icon.

| Variant | | Kind | Solid variant |
|---|---|---|---|
| `Icon::Sun` | Brightness, a frontlight, daytime. | 0 | a filled disc |
| `Icon::Moon` | Night, sleep, a dark theme. | 1 | the same: always filled |
| `Icon::Gear` | Settings. | 2 | a filled disc |
| `Icon::ChevronLeft` | Back, or previous. | 3 | the same |
| `Icon::ChevronRight` | Forward, or next, or "this row opens something". | 4 | the same |
| `Icon::ChevronUp` | Up, or previous. | 5 | the same |
| `Icon::ChevronDown` | Down, or next, or "this opens below". | 6 | the same |
| `Icon::Check` | A confirmation, or a setting that is on. | 7 | the same |
| `Icon::Cross` | A dismissal, or a setting that is off. | 8 | the same |
| `Icon::Battery` | Charge; every variant past the first fills the body. | 9 | a filled body |
| `Icon::Wifi` | Wireless, in the usual three arcs. | 10 | the same |
| `Icon::Book` | A book, a library, a document. | 11 | a filled cover |

Any `IconRef::variant` other than 0 asks for the solid one. The moon is always
filled: an outlined crescent is an arc with the bite erasing half of it, which
reads as a bracket.

**Example — an icon in a screen**

```rust
use xpui::{HStack, Icon as Glyph, IconToggle, hstack};
use xpui_chrome::Icon;

#[derive(Clone, Copy)]
enum Msg {
    Light(bool),
}

# xpui::testing::install();
let row: HStack<Msg> = hstack![8;
    Glyph::new(Icon::Book),
    IconToggle::new(Icon::Sun, true).on_change(Msg::Light),
];
```

**Example — what crosses to the backend**

```rust
use xpui::host::IconRef;
use xpui_chrome::Icon;

let wifi: IconRef = Icon::Wifi.into();
assert_eq!((wifi.kind, wifi.variant, wifi.size), (10, 0, 32));

let solid = IconRef { variant: 1, size: 24, ..Icon::Battery.into() };
assert_eq!(solid.kind, Icon::Battery.kind());
```

### Every icon

#### `Icon::ALL`

Every icon, in the order of their kinds.

```text
pub const ALL: [Icon; 12] = [
    Icon::Sun,
    Icon::Moon,
    Icon::Gear,
    Icon::ChevronLeft,
    Icon::ChevronRight,
    Icon::ChevronUp,
    Icon::ChevronDown,
    Icon::Check,
    Icon::Cross,
    Icon::Battery,
    Icon::Wifi,
    Icon::Book,
]
```

A kind is looked up in this array, so a variant missing from it is undrawable:
[`icon_size`](#icon_size) answers 0 and [`draw_icon`](#draw_icon) returns early.

```rust
use xpui_chrome::Icon;

for (index, icon) in Icon::ALL.into_iter().enumerate() {
    assert_eq!(usize::from(icon.kind()), index);
}
```

### The number

#### `Icon::kind`

The number that crosses to a backend.

```text
pub const fn kind(self) -> u16
```

The discriminant, and what `IconRef::kind` holds for this icon.

```rust
use xpui_chrome::Icon;

assert_eq!(Icon::Sun.kind(), 0);
assert_eq!(Icon::Book.kind(), 11);
```

**See also:** [`draw_icon`](#draw_icon), [`icon_size`](#icon_size)

## `draw_icon`

Draws `icon` with its top-left corner at `origin`.

```text
pub fn draw_icon(origin: Point, icon: IconRef)
```

![A book and a sun each drawn at 8, 16, 24, 32 and 48 pixels, left to right, sitting on one line](https://raw.githubusercontent.com/XPUI-Framework/xpui-gallery/main/gallery/tests/screenshots/reference/chrome_icon_sizes.png)

| Parameter | Meaning |
|---|---|
| `origin` | The top-left corner of the icon's square. |
| `icon` | `kind` is one of `Icon`'s kinds; `variant` other than 0 asks for the solid one; `size` is the edge to draw at, as [`icon_size`](#icon_size) rounds it. |

It paints through the installed host with lines and rectangles alone, so a
backend with no bitmap font still has icons. It draws nothing for a kind that is
not an `Icon`, or for a size [`icon_size`](#icon_size) answers 0 to. A backend
answers `Canvas::draw_icon` and `Canvas::icon_size` with this function and its
neighbour.

**Example — a check at 16 pixels**

```rust
use xpui::host::IconRef;
use xpui::testing::{self, DrawOp};
use xpui::Point;
use xpui_chrome::{Icon, draw_icon};

# testing::install();
# testing::reset();
draw_icon(Point::new(100, 100), IconRef { size: 16, ..Icon::Check.into() });

let strokes = testing::ops_log()
    .into_iter()
    .filter(|op| matches!(op, DrawOp::Line { .. }))
    .count();
assert_eq!(strokes, 2);
```

**Example — nothing to draw**

```rust
use xpui::host::IconRef;
use xpui::testing;
use xpui::Point;
use xpui_chrome::{Icon, draw_icon};

# testing::install();
# testing::reset();
draw_icon(Point::new(0, 0), IconRef::new(99));
draw_icon(Point::new(0, 0), IconRef { size: 6, ..Icon::Sun.into() });
assert!(testing::ops_log().is_empty());
```

**See also:** [`icon_size`](#icon_size), [`Icon`](#icon)

## `icon_size`

The edge length this crate would actually draw, or 0 if it would draw nothing — which is how the framework knows to reserve no space.

```text
pub fn icon_size(icon: IconRef) -> i32
```

0 for a kind that is not an `Icon`, and 0 below 8 pixels, where the strokes
collide and a gear and a sun are the same smudge. Otherwise the requested size
rounded down to an even number: every glyph here is symmetric about its own
centre, and an odd edge puts that centre half a pixel off, which on a 1-bit panel
is the difference between a straight line and a stepped one. It needs no host.

```rust
use xpui::host::IconRef;
use xpui_chrome::{Icon, icon_size};

let at = |size: i32| icon_size(IconRef { size, ..Icon::Gear.into() });
assert_eq!(at(32), 32);
assert_eq!(at(25), 24, "rounded down to an even edge");
assert_eq!(at(8), 8);
assert_eq!(at(7), 0, "too small to be worth drawing");
assert_eq!(icon_size(IconRef::new(99)), 0, "not an icon this crate has");
```

**See also:** [`draw_icon`](#draw_icon)
