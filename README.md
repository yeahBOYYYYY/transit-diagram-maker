# Transit Diagram Maker (Standalone Rust/WASM Edition)

A specialized, high-performance vector editor designed to construct complex, topologically sound metro and transit maps. The application is architected around a procedural node-graph paradigm, powered by a **Rust** backend compiled to **WebAssembly (WASM)** with a **Vue 3 / TypeScript** dark-mode web presentation layer.

---

## 🏗️ Architecture & Core Features

```
                ┌──────────────────────────────────────────────────┐
                │          Vue 3 / TypeScript Thin Client          │
                │  - Left Toolbar (Select, Pen, Station, Erase, T) │
                │  - Center Infinite Viewport & Canvas Renderer    │
                │  - Right Contextual Parameter & Node Graph Panel │
                └─────────────────────────┬────────────────────────┘
                                          │ Serialized JSON Actions (EditorAction)
                                          ▼
                ┌──────────────────────────────────────────────────┐
                │             WASM Message Bridge (JS/Rust)        │
                │  - Universal C-ABI Memory Exports                │
                │  - Zero External Dependencies, Pure WebAssembly  │
                └─────────────────────────┬────────────────────────┘
                                          │
                                          ▼
                ┌──────────────────────────────────────────────────┐
                │               Rust Core Engine                   │
                │  - Strict Grid Channel Line Snapping             │
                │  - Quadratic Bezier Corner Filleting (45°/90°)   │
                │  - Dot & Tick Stations (Slinky Stretched Dots)   │
                │  - Line Patterns (Solid, Dashed, Hollow+Inner)   │
                │  - Spatial BVH (Text Label Collision Detection)  │
                │  - Viewport Camera & Infinite f64 Grid Engine    │
                │  - Station & Line Erase Tool Support             │
                │  - Standalone Optimized SVG Vector Exporter      │
                │  - Lossless .transit Document Format             │
                └──────────────────────────────────────────────────┘
```

### 1. Strict Grid Channel Locking & Procedural Auto-Rounding
- **Strict Grid Channels**: All vertical and horizontal lines are strictly locked into single grid lines ($x = x_0$ or $y = y_0$). Diagonal transit tracks connect grid intersections at strict 45° octolinear angles with equal $\Delta x$ and $\Delta y$.
- **Procedural Auto-Rounding**: Sharp corners at 45°/90° intersections are automatically filleted into smooth quadratic Bezier curves (`Q cx cy, x y`) with user-tunable radius.

### 2. Line Patterns (Solid, Dashed, Hollow with Inner Color)
- **Solid**: Continuous colored stroke.
- **Dashed**: Parametric dash length and gap length.
- **Hollow**: Double stroke consisting of an outer border color and a customizable inner color (e.g. white inside red, or yellow inside dark blue).

### 3. Stations: Dot & Tick Only (with Slinky Stretched Dots)
- Strictly allows only **Dot** and **Tick** styles.
- **Slinky Stretched Dots**: A Dot station can be stretched across any number of parallel grid channels (e.g. 1 to 8 grid cells). Its geometry is a continuous rectangular body with semi-circle caps at the first and last grid cell, seamlessly grouping multi-track interchange hubs.

### 4. Erase Tool for Stations & Lines
- Clicking or brushing over a station completely removes it and its attached child text label.
- Clicking a transit line segment slices/erases that segment non-destructively.

### 5. Real-Time BVH Label Collision Detection
- Real-time 2D Bounding Volume Hierarchy (BVH) spatial queries across all text labels. Overlapping labels are flagged with visual error strokes (highlighted in red) and counted in the top navigation bar.

### 6. Standalone .transit & SVG Export
- Lossless JSON serialization format (`.transit`).
- One-click SVG vector export preserving all filleted paths, hollow strokes, slinky dot capsules, and typography.

---

## 🚀 Quick Start

### 1. Run Automated Test Suite
```bash
cargo test
```
All 7 unit tests verify:
- Strict horizontal/vertical grid channel locking
- Quadratic Bezier corner filleting
- Station erasing (station + child text deletion)
- Slinky stretched dot capsule geometry
- Spatial BVH collision detection
- Reversible Action/Response undo/redo history
- Lossless `.transit` document serialization

### 2. Run Web Development Server
```bash
cd web
npm install
npm run dev
```
Open `http://localhost:3000` to start designing transit maps!

---

## ⌨️ Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| **V** | Selection Tool |
| **P** | Transit Line Pen Tool |
| **S** | Station Tool (Dot / Tick) |
| **E** | Erase Tool (Erase Stations or Cut Lines) |
| **T** | Text Label Tool |
| **Enter / Esc** | Commit active line path |
| **Middle Drag** | Pan Viewport Camera |
| **Mouse Wheel** | Zoom In / Out anchored to cursor |
| **Ctrl + Z** | Undo Action |
| **Ctrl + Shift + Z** | Redo Action |
| **Delete / Backspace** | Delete Selected Node(s) |
