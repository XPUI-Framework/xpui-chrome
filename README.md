# `xpui-chrome`

The eight themed components [`xpui`](../../xpui/) asks a backend to paint,
painted from drawing primitives alone.

`xpui` deliberately has no opinion about what a list row looks like. It asks,
and a backend answers. A backend sitting on a component library — FreeInkUI —
answers by calling that library, and a Rust screen ends up pixel-identical to a
native one. A backend sitting on a *drawing* library has nothing to call, and
would otherwise have to write a widget toolkit before it could show anything.

This is that toolkit, once, for all of them:

| | |
|---|---|
| `draw_list` | rows, subtitles, values, and the selected-row marker |
| `draw_option_popup` + `option_popup_row_rect` | a centred dialog and where its rows are |
| `draw_slider` | dithered track, fill, and a knob carrying the control's state: paper when idle, dithered once the keys are on it, with the whole control outlined as well when it is open |
| `draw_progress_bar` | outline and proportional fill |
| `draw_header` | title band and rule |
| `draw_sub_header` | group heading with a trailing rule |
| `draw_button_hints` | four slots along the bottom |
| `draw_scroll_indicator` | a thumb, and nothing at all when everything fits |

## Using it

```rust
# struct MyBackend;
# impl MyBackend { fn flush(&self) {} }
use xpui_chrome::Tokens;

static TOKENS: Tokens = Tokens::DEFAULT;

xpui_chrome::plain_chrome! {
    for MyBackend,
    tokens: |_backend| &TOKENS,
    request_update: |backend| backend.flush(),
}
```

Your backend implements `Canvas`, `TextMetrics`, `InputSource` and `Clock`.
That macro writes the whole of `Chrome`.

`request_update` is the one thing you pass in, because only a backend knows how
to get pixels onto a panel.

Both parameters are closures over the backend, not plain values. `tokens` is
one so a backend carrying its own can answer `|backend| &backend.tokens`: two
backends in one process may be driving two different panels, and a global would
give them one theme between them.

Nine of the trait's eleven methods are also plain functions, so a backend with
*some* components of its own can take only the ones it lacks rather than the
whole impl. The other two are not on offer: `metric` is answered by
`Tokens::metric`, and `request_update` is the one a backend passes in.

No backend here takes that route yet. `xpui-embedded-graphics` takes all nine
through the macro, and the FreeInkUI backend answers `Chrome` over its own C
ABI without depending on this crate.

## Two things worth knowing

**Selection is a marker, not an inversion.** `Canvas::draw_text` always paints
ink, so filling a row black and drawing the label over it erases the label. The
selected row gets a bar down its leading edge and an outline instead.

**One layout function serves the dialog and its hit-testing.**
`draw_option_popup` paints the rows and `option_popup_row_rect` reports where
they are; both go through `popup_layout`. Deriving that geometry twice is how a
dialog ends up painting row 3 where row 2 responds to a touch — a failure that
looks completely fine until somebody taps it. There is a test that catches
drift of two pixels.

## Why a macro rather than a blanket impl

`impl<T: Canvas + TextMetrics> Chrome for T` is what this obviously wants to be,
and it is illegal: both the trait and the type parameter are foreign to this
crate, so the orphan rule rejects it (`E0210`). The macro writes the same impl
into your crate, where it is allowed.

## Tokens

`Tokens` holds every number these functions paint with — the same values
`ThemeMetric` asks for, plus a few this crate needs and `xpui` does not name.
Change those rather than the drawing code. `content_bottom` is derived from the
live panel height rather than stored, so one `Tokens` works on any panel.
