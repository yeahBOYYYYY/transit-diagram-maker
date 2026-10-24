# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Transit Diagram Maker: a grid-based vector editor for metro maps. The spec is `docs/requirements.md` (a "Photoshop-like" dark UI is a top priority there). Rust core → WebAssembly, Vue 3 + TypeScript front end in `web/`.

`docs/bugs.md` is the user's bug list, one dated section per commit hash (newest first). "Fix the bugs for this commit" means the section for the current `HEAD`; ask when an item leaves the fix open.

## Commands

Rust (from the repo root):
```bash
cargo test                                   # all Rust tests
cargo test -p transit-core --test editor     # one integration test file
cargo test -p transit-core pill              # tests whose name matches "pill"
cargo clippy --all-targets                   # expected to be warning-free
```
Do not run `cargo fmt`: the code is not rustfmt-clean (long lines, compact literals, no `rustfmt.toml`), so it rewrites nearly every file. Match the surrounding style by hand.

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
npx playwright test --update-snapshots      # refresh failing visual baselines (Linux-specific PNGs)
npx playwright test visual --update-snapshots=all   # rewrite every baseline (small UI changes pass the 2% tolerance)
```
Playwright uses the locally installed Google Chrome (`channel: 'chrome'`); `PW_CHANNEL=chromium` switches to the bundled browser. Unit and e2e tests both need the wasm package built first. Outside CI the e2e config reuses a server already listening on :5174, so a leftover vite (e.g. started with `npx vite` and killed only at the `npx` level) can serve stale code; kill stray `vite.js` processes if tests behave as if your change isn't there (`pkill -f '[v]ite.js'`: a plain `pkill -f vite.js` also matches, and kills, the shell running it).

Unit tests run components in happy-dom, which can crash while patching `OptionsBar` from one tool's branch to another (real Chrome is fine); mount a fresh component per tool instead of switching tools on a mounted one.

## Architecture

**The Rust engine owns the document; the web app is a view over it.** `crates/transit-core` holds the model, geometry, history, rendering and I/O. `crates/transit-wasm` is a thin `wasm-bindgen` wrapper where structured data crosses as JSON strings and ids as `f64` (to avoid JS `bigint`). `web/src/core/engine.ts` (`Engine`) is the typed TS mirror of that API, and `web/src/core/types.ts` mirrors the serde types. When you change a Rust type or API, update all three layers.

Core concepts (`transit-core`):
- **Everything is in cell units.** A cell `(x,y)` has its center at `(x+0.5, y+0.5)`. Lines are lists of *turning* cells whose consecutive points are 8-way aligned; `geom::normalize_path` routes, dedups and compresses them. Pixels only appear at render time (`× DocSettings::cell_px`, which is never saved): the canvas always uses `CELL_PX` = 24 (`web/src/core/viewport.ts` mirrors it) and an SVG export may pick its own scale (`render::export_svg_with`). Free positions (text, icons, station offsets) are stored with 0.1-cell precision (`Vec2::snapped`), and every rotation is a multiple of 45° (`editor::snap_rotation`). Everything snaps to the cells by default: a non-free `translate` moves text and icons in whole cells next to grid content, or puts their top-left anchor on a cell center on their own; Alt (`free = true`) moves in tenths. New text and icons are placed at the cell center (`tools::placeAt`).
- **Layer tree** (`model.rs`, `tree.rs`): `Document.root` is ordered **bottom-to-top** (later = painted on top). `Editor::tree()` returns the reverse, top-first order that the layers panel shows. `move_node`/`restack` indices are bottom-to-top; `web/src/core/tree.ts::stackIndex` converts between the two. Group roles: `Line { line_id }` (all segments and stations of a line), `Station` (a station plus its label), `Generic`. Empty Line/Station groups are auto-pruned; generic groups are kept.
- **Stations:** `Station::offset` is a free displacement from the line cell set by Alt-drag (`translate(.., free = true)`); a normal move snaps the station back and clears it. Ticks sit in a cell corner (`along` = -1/+1, centered on diagonals, see `geom::tick_center`). Pills are a set of cells edited with `set_pill_cells` (must stay 8-connected and touch a line).
- **Line identity:** every segment carries `line_id`. Erasing splits one segment layer into several layers with the same `line_id`. Each segment has its own style, so a line can change style along its way, but the color is shared: a color patch to one segment recolors the whole line (`Editor::update_content`), and `Editor::update_line_style` patches every segment (used when the line group is selected). Segments joining another line keep their style and take its color. Stations reference `line_ids` and are re-snapped onto their line after transforms. Labels (`TextBox.station_id`) follow a moved station unless the label itself was also moved.
- **History** (`editor.rs`): every mutation goes through `Editor::apply`, which snapshots the whole `Document`, rolls back on error, and records only if something changed. Drags use transactions: `begin()`, then on each move `rebase()` + re-apply the absolute delta, then `commit()` (or `cancel()`). That is why a drag, an erase stroke or a slider scrub is one undo step. Group expand/collapse deliberately bypasses history.
- **Rendering** (`render.rs`): the same SVG markup feeds the live canvas (`render_fragments`, one fragment per visible leaf) and `export_svg`, so the canvas and the export match. Line caps are separate filled shapes (roundness comes from `cap_roundness`). Stroke kinds are plain, hollow (an `inner_color` channel), dashed and inner dashed (a plain line with `inner_color` dashes down its middle); dash patterns are fitted so a line starts and ends on a full dash, and `dash_start_offset`/`dash_end_offset` keep the ends plain (`geom::dash_array`). Text `align` is visual (start = left) even for `direction = rtl`, so `text_svg` swaps the SVG anchor. Dot and pill stations are a "blob skeleton" stroked with round joins (a Minkowski sum, giving a constant-radius outline). Diagonal links are skipped when an orthogonal neighbor already bridges the two cells, and 2×2 blocks are filled.
- `.transit.svg` (`io.rs`) = the SVG export at 24 px per cell plus the editor data: each layer-tree `<g>` starts with a `<transit:group|segment|station|text|icon>` element (namespace `io::NS`) holding the node as attributes; settings and uploaded icons sit in `<metadata><transit:document>`, fonts in `<style transit:font=…>`. Reading and writing are hand-written (no serde), so a new model field must be added to both `io.rs` functions. Enum attributes do go through their serde names (`enum_name`/`enumeration`), so renaming an enum value needs a `#[serde(alias = "old")]` to keep older files loading (e.g. `hollow_dashed` → `InnerDashed`). `validate` repairs ids, settings and paths on load. SVG export options (scale, margin, background) are `ExportOptions`, kept in the web preferences, not in `DocSettings`.
- **Uploaded assets** (`assets.rs`): fonts (base64 data URLs behind `Arc<str>` so undo snapshots stay cheap) and SVG icons live in `Document.assets`. Uploaded SVG is untrusted and goes through `assets::sanitize_svg` (an element/attribute allowlist) on upload and on load; never render user SVG that skipped it.

Web app (`web/src`):
- `stores/editor.ts` is the central reactive store, used as a singleton via `useEditor()` / `provideEditor()`. Mutate through `store.run(e => …)`, which calls the engine, refreshes the mirrored `fragments`/`tree`/`settings`, and turns thrown engine errors into a status-bar message. Selection, view, tool, overlays and text-edit state live only in TS.
- `core/actions.ts` is the catalogue of every user action (labels and default shortcuts). `stores/commands.ts::runAction` implements them; the menu bar, the keyboard handler and the shortcut editor all go through it. Add new commands there.
- `tools/index.ts` holds each tool's pointer/keyboard behavior as plain objects. Each tool has its own button in `ToolsPanel` (no shared slots or flyouts). There is no icon tool: pictograms are added from the Icons panel (click adds one in the middle of the view, drag places it), which also holds the defaults for new pictograms; e2e tests use `dropIcon` in `tests/e2e/helpers.ts`. `components/CanvasView.vue` dispatches events to the active tool. It does hit-testing with `elementsFromPoint` on `g[data-id]` fragments, measures selection bounds with DOM `getBBox` (stations, whose stroked skeletons `getBBox` under-measures, and anything unmeasurable use engine bounds), and handles the global keydown. The canvas captures the pointer on press, so `dblclick` reuses the handle recorded at `pointerdown`.
- `stores/prefs.ts` holds app preferences, shortcuts and tool defaults, persisted to localStorage. These are separate from the document settings, which live in the Rust `DocSettings` and are saved and undoable. `loadPrefs` deep-merges stored values over the defaults and drops unknown keys, so removing a pref or shortcut needs no migration, but a renamed *value* (such as a line kind) must be migrated there.
- `OptionsBar` scrolls sideways when a tool's options don't fit (hints shorten first), so put the always-needed options (like line caps) before the style-specific ones.
- The upper dock panel is one tabbed `PanelFrame` (Properties | Icons, `state.dockTab`) created in `App.vue`; `PropertiesPanel` and `IconsPanel` are its bodies and have no frame of their own.
- UI wording: every user-visible string spells words out, units included ("Line width", "Largest zoom", "pixels", "milliseconds", not "Line W", "Max zoom", "px", "ms"). Ordinary English contractions are fine. This also applies to Rust error messages, which reach the status bar.
- Every `ColorField` has an eyedropper (`core/eyedropper.ts`): the next click takes the color painted on the canvas (or any `[data-swatch]` element) and is swallowed so it doesn't reach a tool.
- Every property shown in the UI has help text in `core/help.ts`, displayed by `HelpTip` (NumberField/ColorField take a `help` prop); add an entry for any new property. A unit test checks the Properties panel, dialogs and tool options for missing ones.
- `window.__transit` exposes the store; e2e tests use it to read state and to convert cells to screen coordinates (`tests/e2e/helpers.ts`). `openApp` removes `showSaveFilePicker`/`showOpenFilePicker` so saving and opening use the download / `<input type=file>` fallbacks Playwright can drive; tests that need the File System Access path install a fake picker.

## Git workflow

`main` = releases, `dev` = integration. New work goes on `feature/*` and fixes on `bugfix/*`, each branched from `dev` and merged back with `--no-ff`. Exception: fixing a `docs/bugs.md` section is one commit directly on `dev` ("Fix the bug list for commit <hash>") that also includes the user's uncommitted edits to `docs/bugs.md`.
