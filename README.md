# Transit Diagram Maker

A grid based vector editor for topologically sound metro and transit maps.
The document model, geometry, undo/redo history, `.transit` serialization and
SVG rendering are written in **Rust** and compiled to **WebAssembly**; the
dark, Photoshop-style workspace is built with **Vue 3 + TypeScript**.

See [`docs/requirements.md`](docs/requirements.md) for the specification.

## Features

- **Infinite canvas** with a square grid, hand tool / space-drag / middle-drag
  panning, scroll panning and `Ctrl + Scroll` zoom around the cursor.
- **Line pen** (`P`): click cells to add points; unaligned points are routed with
  one straight run and one 45° run, and every turn gets a smooth curve.
  Plain, hollow and dashed styles; butt or round caps (roundness adjustable).
- **Erase tool** (`E`, shares the pen slot – long press to switch): cuts lines
  into separate segment layers inside the same line group.
- **Stations** (`S`): dot stations that snap to the line center, tick stations
  aligned with the line border, and pill stations painted across several lines
  with a constant-radius outline. Each station gets a name label grouped with it.
- **Text** (`T`) with inline editing and **pictograms** (`I`): wheelchair,
  airport, ferry, national rail, metro, parking, bus, tram, bicycle, taxi,
  information and interchange. Icons default to the dot station size.
- **Selection** (`V`): click, Shift+click, marquee, drag to move, click a
  selected object again to get rotate handles (grid content rotates in 45°
  steps), arrow-key nudging, duplicate, delete.
- **Layers panel**: visibility, locking, rename, groups, ungroup, drag & drop
  re-ordering; the top of the list is painted on top.
- **History**: `Ctrl+Z` / `Ctrl+Shift+Z` (also `Ctrl+Y`); drags, erase strokes
  and slider scrubs are single steps.
- **Everything is configurable**: Document settings (grid size, line width,
  curve radius, cap roundness, station sizes, label defaults, export options)
  are stored in the document; Preferences (zoom, nudges, colors, history size…)
  and fully rebindable keyboard shortcuts are stored in the browser.
- **Files**: save/open lossless `.transit` (versioned JSON) and export a
  standalone SVG that renders exactly like the canvas.

## Getting started

Requirements: Rust (stable) with the `wasm32-unknown-unknown` target,
[`wasm-pack`](https://rustwasm.github.io/wasm-pack/), Node.js 20+.

```bash
rustup target add wasm32-unknown-unknown
cd web
npm install
npm run build:wasm   # compiles crates/transit-wasm into web/src/wasm/pkg
npm run dev          # http://localhost:5173
```

`npm run build` produces a static site in `web/dist`.

> On WSL with the repository on a Windows drive, start the dev server with
> `VITE_POLL=1 npm run dev` so file changes are picked up.

## Project layout

```
crates/
  transit-core/   document model, geometry, editor + history, rendering, I/O
  transit-wasm/   wasm-bindgen API (JSON in/out) used by the web app
web/
  src/core/       engine bridge, types, viewport math, shortcuts, actions
  src/stores/     editor state, commands, preferences
  src/tools/      pointer/keyboard behavior of every tool
  src/components/ menu bar, options bar, tools, canvas, panels, dialogs
  tests/unit/     Vitest unit + component tests (real WASM engine)
  tests/e2e/      Playwright end-to-end + visual regression tests
```

All geometry is stored in grid cell units: a line is a list of turning cells,
a station references the cells it covers, so changing the grid size scales the
whole map consistently.

## Tests

```bash
cargo test                 # Rust unit + integration tests
cd web
npm test                   # Vitest unit & component tests
npm run typecheck
npx playwright test        # end-to-end + visual snapshots (uses local Chrome)
```

Playwright uses the installed Google Chrome by default; set
`PW_CHANNEL=chromium` to use Playwright's bundled browser
(`npx playwright install chromium`). Visual baselines live in
`web/tests/e2e/visual.spec.ts-snapshots/`; refresh them with
`npx playwright test --update-snapshots` after intentional UI changes.

## Git workflow

- `main` – released state
- `dev` – integration branch
- `feature/*` – new work, merged into `dev` with `--no-ff`
- `bugfix/*` – fixes, merged into `dev` with `--no-ff`
