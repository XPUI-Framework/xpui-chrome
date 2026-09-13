# `xpui-chrome`

## What this is, and what it may not become

The eight themed components `xpui` asks a backend to paint — a list, a
dialog, a slider, a progress bar, a header, a sub-header, a button-hint bar
and a scroll indicator — painted from drawing primitives alone, through the
`Renderer` and `Font` façades of whichever host is installed. One macro,
`plain_chrome!`, writes a backend's whole `Chrome` implementation from four
things it passes in.

**It depends on `xpui` and nothing else, and may not grow a second
dependency.** Not a drawing library, not a backend, not a board crate: what it
paints with are the primitives a backend hands it, and a number it paints to
comes from `Metrics`, never from a literal at a call site. It has no opinion
about which backend called it.

## The gate

```bash
./build-and-test.sh          # everything below
./build-and-test.sh fix      # the same, formatting in place first
```

```text
format · file sizes · crates are tested · READMEs warn · prose is compiled · documented paths resolve · rustdoc links resolve · documented commands resolve · lint · tests · doctests · README sections · AGENTS.md · published crates deny missing_docs · comment blocks · comment narration
```

There is no `all` mode; this list is the whole of it, and a last stage,
`the gate is documented`, compares it to what ran. Run it before saying a
change is done, and read the real exit code.

## What only this repository checks

Nothing a sibling lacks. The two bare-metal clippy runs under `lint`, on
`riscv32imc` and `thumbv6m`, lint this crate alone — `-p xpui-chrome`, on
the targets `BARE_METAL` names; they are the only checks that reach its
`no_std` paths before a firmware build does.

## Style that bites here

- **`no_std`.** `alloc::` types explicitly; the tests run on the host through
  `xpui`'s `testing` feature.
- **Every panel-dependent size comes from `Metrics`.** A row height as a
  literal is wrong on every panel but the one it was written against; three
  presets and `Metrics::for_panel` exist so it never has to. A one-pixel
  border is not that kind of number.
- **Measure text through the host.** `Font::text_width` and `line_height`,
  never an estimate — a label reserved by guess overruns its slot.
- **One layout function serves the dialog's painter and its hit-test.** The
  rows and their rects come from `popup_layout`, read by `draw_option_popup`
  and `option_popup_row_rect` alike. Two copies of one geometry drift by a
  pixel and paint row 3 where row 2 answers a tap.
- **The knob carries the focus mark**, not the track; `docs/design.md` says
  why, and why a second shade is not available on one bit.
- **Every `pub` item is documented.** `#![deny(missing_docs)]` is on.
- **A file under `src/` is at most 400 lines.** `xtask/src/tree.rs`
  and `src/icons.rs` are both within fifteen lines of it.
- **Twenty-six of `tests/paint.rs`'s forty tests assert on recorded draw
  calls**; nine of the other fourteen are `Metrics` arithmetic. The
  thirty-nine goldens in `tests/snapshots/` are call transcripts. A
  documentation change never touches them; if one changes, something else
  did.

## Where the documentation lives, and what proves each piece

| Document | Proven by |
|---|---|
| [`README.md`](README.md) | its `rust` fence is a doctest, mounted by `src/lib.rs` |
| [`docs/README.md`](docs/README.md) | its paths resolve; the README-heading check exempts it, because it is the index of `docs/`, not a front page |
| [`docs/components.md`](docs/components.md) | `documented paths resolve`; `src/lib.rs` mounts it, which compiles nothing while it carries no `rust` fence |
| [`docs/design.md`](docs/design.md) | mounted by `src/lib.rs`; likewise |
| [`docs/contributing.md`](docs/contributing.md) | every path and command it gives resolves; the umbrella command is `xpui-dev`'s |
| `AGENTS.md` | the stage list above is compared to what the gate runs |
| every `///` and `//!` | `rustdoc links resolve`, and the two comment checks |

## Git

Never stage, never commit, never push without being asked, each time. The
index is the reviewer's queue; leave new work unstaged. No self-attribution
in a commit message. Never rewrite a commit that exists; a correction is a new
commit. The rules that apply to all ten repositories, and the five review
steps, are in [`xpui`'s `docs/orientation.md`](https://github.com/XPUI-Framework/xpui-framework/blob/main/docs/orientation.md);
how a change is built and reviewed here is in
[`docs/contributing.md`](docs/contributing.md).
