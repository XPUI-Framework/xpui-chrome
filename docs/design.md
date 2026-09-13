# Design decisions

The arguments behind choices the code states in one sentence. Each section
names the item that carries the sentence. What the components are, and what
a backend hands them, is in [components.md](components.md).

## The knob carries the focus mark

`draw_slider` has to tell three states apart on one bit of colour, and the
knob is the one part of the control still painted paper — so idle leaves it
so, focus fills it with a dither, and an open control adds one outline around
the whole control.

Two other marks were built and rejected by eye. A bar down the leading edge
lands where a stepper draws its `-` and reads as part of the glyph. A box
around a wide, mostly empty control is the heaviest thing on the panel. And a
second *shade* for the track is not available at all: `fill_rect_dither`'s
flag is a parity, not a density, so both of its values are the same 50%
checkerboard on opposite squares. That is what left the knob as the only place
with a change of shade still in it.

## A macro rather than a blanket impl

`impl<T: Canvas + TextMetrics> Chrome for T` is what `plain_chrome!` obviously
wants to be, and it is illegal: both the trait and the type parameter are
foreign to this crate, so the orphan rule rejects it (`E0210`). The macro
writes the same impl into the backend's own crate, where it is allowed. Which
of the trait's methods are also plain functions, for a backend that wants
only some, is in [components.md](components.md).

## Selection is a marker, not an inversion

`Canvas::draw_text` always paints ink, so filling a row black and drawing the
label over it erases the label. The selected row gets a bar down its leading
edge and an outline instead; `Metrics::selection_marker_width` is its width.

## One layout function serves the dialog and its hit-testing

`draw_option_popup` paints the rows and `option_popup_row_rect` reports where
they are; both go through `popup_layout`. Deriving that geometry twice is how
a dialog ends up painting row 3 where row 2 responds to a touch — a failure
that looks completely fine until somebody taps it.
`every_dialog_row_is_painted_where_hit_testing_says_it_is` selects each row
in turn and asserts that the rect `option_popup_row_rect` reports is exactly
the one the row was outlined in, so a pixel of drift in position or size
fails it.

## It depends on `xpui` and nothing else

Not on a backend, not on a board. What a component paints with are the
drawing primitives a backend hands it — `Renderer` and `Font` — and
everything it needs to know about a panel arrives as a `Metrics`. A
dependency on a backend would make every other backend depend on one of its
peers; a dependency on a board would put a device name below the seam `xpui`
keeps every device above.
