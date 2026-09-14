# Reference

The whole of `xpui-chrome`'s public API, by area. It is written for a backend
author: someone answering `xpui`'s `Chrome` trait on top of a drawing library.
[The host contract](https://github.com/XPUI-Framework/xpui-framework/blob/main/docs/host.md)
is what a backend implements, and [design.md](design.md) is why the choices here
are what they are.

## Topics

Each page lists every public name in its area with its declaration, its
parameters, examples you can copy, and a picture of what it draws. The gate
checks every page against the code, so what a page says an item is, the compiler
agrees with, and every `rust` block on them is compiled and run as a doctest.

| Group | Page | Holds |
|---|---|---|
| Backend | [plain chrome](reference/plain-chrome.md) | `plain_chrome!`, and the four things a backend passes in |
| Painting | [painting](reference/painting.md) | the header, the controls and the hint bar: `draw_header`, `draw_sub_header`, `draw_slider`, `draw_progress_bar`, `draw_button_hints` |
| | [painting lists](reference/painting-lists.md) | rows of content: `draw_list`, `draw_option_popup`, `draw_scroll_indicator` |
| Layout & sizing | [layout](reference/layout.md) | where a list's rows and a dialog's rows land: `row_height`, `rows_that_fit`, `popup_layout`, `PopupLayout`, `option_popup_row_rect` |
| | [metrics and labels](reference/metrics-and-labels.md) | `Metrics` and its three presets, `Labels`, and the re-exported `KeyRow` and `RowKey` |
| Icons | [icons](reference/icons.md) | `Icon`, `draw_icon` and `icon_size` |
