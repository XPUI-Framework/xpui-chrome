# The components, and what they paint from

`xpui` deliberately has no opinion about what a list row looks like. It asks,
and a backend answers. A backend sitting on a component library answers by
calling that library, and a Rust screen ends up pixel-identical to a native
one. A backend sitting on a *drawing* library has nothing to call, and this
crate is what it answers with.

## The eight

| | |
|---|---|
| `draw_list` | rows, subtitles, values, and the selected-row marker |
| `draw_option_popup` + `option_popup_row_rect` | a centred dialog and where its rows are |
| `draw_slider` | dithered track, fill, and a knob carrying the control's state: paper when idle, dithered once the keys are on it, with the whole control outlined as well when it is open |
| `draw_progress_bar` | outline and proportional fill |
| `draw_header` | title band and rule |
| `draw_sub_header` | group heading with a trailing rule |
| `draw_button_hints` | one slot per key along the bottom, each word over the key it names |
| `draw_scroll_indicator` | a thumb, and nothing at all when everything fits |

Every one paints through `xpui`'s installed host, like a widget does — not
through a `&self` parameter — so these functions paint through whichever
backend is installed, which is the one that called them.

## The four things a backend passes in

`plain_chrome!` asks for four things, because four parties own them. The
**metrics** size the chrome and come from whoever knows the panel. The
**labels** are words, and only an application knows what language its user
reads. The **key row** says which word sits over which key, which is a fact
about the hardware. And `request_update` is there because only a backend
knows how to get pixels onto a panel.

All four are closures over the backend, not plain values, so a backend
carrying its own can answer `|backend| &backend.metrics`: two backends in one
process may be driving two different panels, in two different languages,
with two different key rows, and globals would give them one of each between
them.

Nine of the trait's eleven methods are also plain functions, so a backend
with *some* components of its own can take only the ones it lacks rather than
the whole impl. The other two are not on offer: `metric` is answered by
`Metrics::metric`, and `request_update` is one of the four things a backend
passes in. No backend takes that route yet: `xpui-embedded-graphics` takes
all nine through the macro, and the FreeInkUI backend answers `Chrome` over
its own C ABI without depending on this crate. Why it is a macro at all is in
[design.md](design.md).

## The icons

Twelve `Icon`s, `Icon::ALL` in order — arrows, a check, a cross, a battery,
wifi and the rest — drawn by `draw_icon` at the size `icon_size` reports, from
lines and rectangles alone, so a backend with no bitmap font still has them.
They are not one of the eight: `xpui` asks for an icon by kind, and
`from_kind` looks it up here.

## Metrics, labels and a key row

`Metrics` holds every number these functions paint to — the same values
`ThemeMetric` asks for, plus a few this crate needs and `xpui` does not name.
Change those rather than the drawing code. `content_bottom` is derived from the
live panel height rather than stored, so one `Metrics` works on any panel.
Three presets ship: `DEFAULT` for a reader-sized panel, `COMPACT` for a small
colour LCD, `SMALL` for a badge's strip, and `Metrics::for_panel` picks by
height.

`Labels` holds the words the hint bar shows. They are not measurements and they
do not scale; English ships because something has to, and every one of them is
meant to be replaced by whatever the application's user reads.

`KeyRow` is `xpui`'s, not this crate's, and says what the keys along a device's
bottom edge mean, left to right. It decides which of the four standard words
lands over which key — and a slot with nothing behind it stays blank, because
naming a key the device does not have sends a person looking for it.
