# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Transit Diagram Maker: a grid-based vector editor for metro maps. The spec is `docs/requirements.md` (a "Photoshop-like" dark UI is a top priority there). Rust core → WebAssembly, Vue 3 + TypeScript front end in `web/`.

## Commands

Rust (from the repo root):
```bash
cargo test                                   # all Rust tests
cargo test -p transit-core --test editor     # one integration test file
cargo test -p transit-core pill              # tests whose name matches "pill"
cargo clippy --all-targets                   # expected to be warning-free
```

Web (from `web/`):
```bash
npm run build:wasm      # REQUIRED after any Rust change: regenerates web/src/wasm/pkg (gitignored)
npm run dev             # http://localhost:5173 ; on WSL with the repo on /mnt/*, use VITE_POLL=1 npm run dev
npm test                # vitest (tests/unit), loads the real wasm via initSync in tests/unit/setup.ts
npx vitest run tests/unit/tools.spec.ts -t "eraser"   # single file / test name
npm run typecheck       # vue-tsc
npm run build           # typecheck + vite build to web/dist
npx playwright test                         # e2e (tests/e2e), starts its own vite on :5174
npx playwright test editing -g "rotate"     # single e2e file / test
npx playwright test --update-snapshots      # refresh visual baselines (Linux-specific PNGs)
```
Playwright uses the locally installed Google Chrome (`channel: 'chrome'`); `PW_CHANNEL=chromium` switches to the bundled browser. Unit and e2e tests both need the wasm package built first.

## Architecture

**The Rust engine owns the document; the web app is a view over it.** `crates/transit-core` holds the model, geometry, history, rendering and I/O. `crates/transit-wasm` is a thin `wasm-bindgen` wrapper where structured data crosses as JSON strings and ids as `f64` (to avoid JS `bigint`). `web/src/core/engine.ts` (`Engine`) is the typed TS mirror of that API, and `web/src/core/types.ts` mirrors the serde types. When you change a Rust type or API, update all three layers.

Core concepts (`transit-core`):
- **Everything is in cell units.** A cell `(x,y)` has its center at `(x+0.5, y+0.5)`. Lines are lists of *turning* cells whose consecutive points are 8-way aligned; `geom::normalize_path` routes, dedups and compresses them. Pixels only appear at render time (`× settings.grid_size`), so changing the grid size scales the whole map.
- **Layer tree** (`model.rs`, `tree.rs`): `Document.root` is ordered **bottom-to-top** (later = painted on top). `Editor::tree()` returns the reverse, top-first order that the layers panel shows. `move_node`/`restack` indices are bottom-to-top; `web/src/core/tree.ts::stackIndex` converts between the two. Group roles: `Line { line_id }` (all segments and stations of a line), `Station` (a station plus its label), `Generic`. Empty Line/Station groups are auto-pruned; generic groups are kept.
- **Line identity:** every segment carries `line_id`. Erasing splits one segment layer into several layers with the same `line_id`. A style patch to any segment applies to all of its line's segments (`Editor::update_content`). Stations reference `line_ids` and are re-snapped onto their line after transforms. Labels (`TextBox.station_id`) follow a moved station unless the label itself was also moved.
- **History** (`editor.rs`): every mutation goes through `Editor::apply`, which snapshots the whole `Document`, rolls back on error, and records only if something changed. Drags use transactions: `begin()`, then on each move `rebase()` + re-apply the absolute delta, then `commit()` (or `cancel()`). That is why a drag, an erase stroke or a slider scrub is one undo step. Group expand/collapse deliberately bypasses history.
- **Rendering** (`render.rs`): the same SVG markup feeds the live canvas (`render_fragments`, one fragment per visible leaf) and `export_svg`, so the canvas and the export match. Line caps are separate filled shapes (roundness comes from `cap_roundness`). Dot and pill stations are a "blob skeleton" stroked with round joins (a Minkowski sum, giving a constant-radius outline). Diagonal links are skipped when an orthogonal neighbor already bridges the two cells, and 2×2 blocks are filled.
- `.transit` = versioned JSON envelope (`io.rs`); `validate` repairs ids, settings and paths on load.

Web app (`web/src`):
- `stores/editor.ts` is the central reactive store, used as a singleton via `useEditor()` / `provideEditor()`. Mutate through `store.run(e => …)`, which calls the engine, refreshes the mirrored `fragments`/`tree`/`settings`, and turns thrown engine errors into a status-bar message. Selection, view, tool, overlays and text-edit state live only in TS.
- `core/actions.ts` is the catalogue of every user action (labels and default shortcuts). `stores/commands.ts::runAction` implements them; the menu bar, the keyboard handler and the shortcut editor all go through it. Add new commands there.
- `tools/index.ts` holds each tool's pointer/keyboard behavior as plain objects. `components/CanvasView.vue` dispatches events to the active tool. It does hit-testing with `elementsFromPoint` on `g[data-id]` fragments, measures selection bounds with DOM `getBBox` (falling back to engine bounds), and handles the global keydown.
- `stores/prefs.ts` holds app preferences, shortcuts and tool defaults, persisted to localStorage. These are separate from the document settings, which live in the Rust `DocSettings` and are saved and undoable.
- `window.__transit` exposes the store; e2e tests use it to read state and to convert cells to screen coordinates (`tests/e2e/helpers.ts`).

## Git workflow

`main` = releases, `dev` = integration. New work goes on `feature/*` and fixes on `bugfix/*`, each branched from `dev` and merged back with `--no-ff`.
