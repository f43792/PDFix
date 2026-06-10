# pdfix

A desktop PDF utility that batch-edits **stroke widths** in PDF files —
primarily to eliminate hairline (sub-pixel) lines that disappear or render
inconsistently when printed or rasterised.

Built with [Tauri 2](https://tauri.app), [SvelteKit](https://kit.svelte.dev/)
and the [mupdf-rs](https://github.com/messense/mupdf-rs) Rust bindings to
[MuPDF](https://mupdf.com/) for PDF rendering and rewriting.

## What it does

For each PDF you drop into the app, every stroked path whose effective
device-space width is at or below a configurable threshold is re-emitted
with a new (typically thicker) width, while:

- **Text content is preserved** (glyphs, fonts, positioning).
- **Images, shades, clipping, layers and structure tree are passed through.**
- Fill colors stay original. Only stroke colors are overridden (default
  black, optional deterministic per-path random for visual debugging).
- Stroke geometry above the threshold is preserved untouched (caps, joins,
  dashes, original width).

Page selection modes:

- **All / Current page** — apply to the whole document.
- **Page range** — `from..=to`.
- **Page rule** — per-page width override using simple expressions
  (`1`, `>=3`, `<5`, `=2`).

Original file disposition:

- **Delete** the original after a successful conversion (default).
- **Rename** to `<name><suffix>.pdf` and write the new file at the
  original path.

In both cases the conversion is staged via a temp file alongside the input,
so a mid-flow failure rolls back to the original file untouched.

Optional **save log** writes a per-input `.txt` report listing every
fill/stroke operation, the substitution decisions, and final totals.

## Settings persistence

All UI settings (hairline threshold, units, page-range mode, page rules,
random-color toggle, save-log toggle, original-file disposition, rename
suffix) are written to disk on change (debounced 500 ms) and re-loaded on
the next launch. If the saved file is missing or corrupt, the app silently
falls back to the bundled defaults — so you can always recover by deleting
the settings file.

| Platform | Settings file location |
|---|---|
| Windows | `%APPDATA%\com.pdfix.app\settings.json` |
| macOS   | `~/Library/Application Support/com.pdfix.app/settings.json` |
| Linux   | `~/.config/com.pdfix.app/settings.json` |

To restore defaults: delete `settings.json` (or use the in-app
"Reset to defaults" affordance, when added).

## Built-in viewer

Click a converted file's name in the list to open the resulting PDF in a
new in-app window (powered by the embedded WebView's native PDF viewer).
The viewer window has its own minimal **File → Close** menu.

## Building

Requirements:

- [Rust](https://www.rust-lang.org/) (stable)
- [Bun](https://bun.sh/) (or npm / pnpm — adjust the scripts)
- The platform prerequisites listed in the
  [Tauri prerequisites guide](https://tauri.app/start/prerequisites/)

Install JS deps:

```bash
bun install
```

Run in development:

```bash
bun tauri dev
```

Production build (creates standalone executable + installers):

```bash
bun tauri build
```

Outputs:

- Standalone exe — `src-tauri/target/release/pdfix.exe` (Windows; equivalent
  on macOS/Linux)
- Installers / bundles — `src-tauri/target/release/bundle/`

The first release build takes a while because `mupdf-sys` compiles MuPDF
from source. Subsequent builds are incremental.

## Testing

Unit + integration tests are run on every PR via CI (see
[`.github/workflows/ci.yml`](.github/workflows/ci.yml)) and gate merging to
`main`.

Run locally:

```bash
# Rust unit + integration tests
cargo test --manifest-path src-tauri/Cargo.toml

# Frontend unit tests
bun test
```

Test fixtures live under `src-tauri/tests/assets/`. The synthetic fixture
PDF (`synthetic.pdf`) is produced by the helper:

```bash
cargo run --manifest-path src-tauri/Cargo.toml --example gen_fixture
```

The customer-provided sample (`reference/B787-50.pdf`) is **not** included
in this repository — it's customer property. Tests rely on the synthetic
fixture only, so they remain reproducible without it.

## Merge policy

Branch protection on `main` requires the CI workflow to pass (both Rust and
frontend test jobs). Open a PR; merge only after green.

## License

Licensed under the **GNU Affero General Public License v3.0 (AGPL-3.0)**.

MuPDF (transitive dependency via `mupdf-rs`) is also distributed under
AGPL-3.0.
