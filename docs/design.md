# Design decisions

The arguments behind choices the code states in one sentence. Each section
names the item that carries the sentence.

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
writes the same impl into the backend's own crate, where it is allowed. Nine
of the trait's eleven methods are also plain functions, so a backend with some
components of its own can take only the ones it lacks; `metric` is answered
by `Metrics::metric`, and `request_update` is one of the four things a backend
passes in, because only the backend knows how to get pixels onto a panel.
