# Contributing to `xpui-chrome`

## Building it

`rust-toolchain.toml` pins the toolchain and the two bare-metal targets, so
`cargo build` on a fresh clone installs what it needs. The only dependency is
`xpui`, fetched from its repository on `main`.

```bash
cargo test                           # the suite, on a laptop
./build-and-test.sh                  # everything CI checks
```

## The gate

A change is not finished until `./build-and-test.sh` passes. It is the same
command CI runs, so a green run locally means what a green tick means there.
The checks are listed in [`AGENTS.md`](../AGENTS.md) and implemented in
[`xtask/`](../xtask/); `./build-and-test.sh fix` formats in place first.

Two things bite here more than anywhere else:

- **Every panel-dependent size a painter uses comes from `Metrics`.** A row
  height as a literal at a call site is right on one panel and wrong on six;
  a one-pixel border is not that kind of number.
- **The goldens are call transcripts, not pixels.** Thirty-nine of them, in
  `tests/snapshots/`; a test compares what a component asked the host to draw.
  A change to how a component paints changes them on purpose:

  ```bash
  UPDATE_SNAPSHOTS=1 cargo test        # accept the new transcripts, then READ the diff
  ```

  A blessed golden is an assertion you have made. What the pixels look like
  is proved downstream, by
  [`xpui-gallery`](https://github.com/XPUI-Framework/xpui-gallery)'s board
  captures.

## The review

Five steps, in order, none skipped:

1. The gate passes, with the real exit code read.
2. The [code-reviewer](../.claude/agents/code-reviewer.md) agent reviews the
   change — every finding resolved, not noted.
3. The [docs-reviewer](../.claude/agents/docs-reviewer.md) agent reviews the
   prose, last: it runs every command a document gives and resolves every
   snippet against the API.
4. The author reviews the code and looks at it in the simulator or on a board.
5. They say commit.

A test that cannot fail is worse than no test. Before adding one, break the
code on purpose and confirm the test notices. Prefer an assertion that pins
a relationship — a dialog row answers taps in exactly the rect it was
painted in — over one that pins a number.

## Commits

The subject says what was done — imperative, under fifty characters, one
concern. The body says what changed and why, in under about ten lines,
carrying the fact that is not in the diff. Nothing about how the bug was
found. No self-attribution.

## Working across the repositories

`xpui-embedded-graphics` in `xpui-backends`, the simulator and the gallery
depend on this crate through a `git` dependency on `main`, so a change to a
public item or to what a component paints reaches all three. The FreeInkUI
backend answers `Chrome` over its own C ABI and never sees one. Before
pushing one, run the umbrella:

```bash
for d in ../xpui*/; do git -C "$d" fetch --quiet --all; done
cd ../xpui-dev && ./build-and-test.sh cross
```

It builds every crate from the sibling checkouts on disk and says which one
broke. `cross` is that repository's gate, not this one's — run it from there,
not here. The fetch first, because its link check resolves every
`github.com/XPUI-Framework/…` URL against each sibling's `origin/main`, and a
stale remote is a stale answer. `xpui`'s [`docs/orientation.md`](https://github.com/XPUI-Framework/xpui-framework/blob/main/docs/orientation.md)
describes the layout it expects.

## Where to read first

[The reference](reference.md) is what each function paints and what it paints
from; [design.md](design.md) is why the choices are what they are.
