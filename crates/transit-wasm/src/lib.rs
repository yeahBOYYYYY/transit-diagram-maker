//! WebAssembly bindings. Structured values cross the boundary as JSON strings,
//! which keeps the binding surface small and the TypeScript side fully typed.

use serde_json::Value;
use transit_core::{io, render, Cell, Editor, Icon, LineStyle, StationKind, StationOptions, TextBox, Vec2};
use wasm_bindgen::prelude::*;

fn err(e: impl std::fmt::Display) -> JsError {
    JsError::new(&e.to_string())
}

fn parse<T: serde::de::DeserializeOwned>(json: &str) -> Result<T, JsError> {
    serde_json::from_str(json).map_err(err)
}

fn json<T: serde::Serialize>(v: &T) -> String {
    serde_json::to_string(v).unwrap_or_else(|_| "null".into())
}

fn ids(json: &str) -> Result<Vec<u64>, JsError> {
    parse(json)
}

#[wasm_bindgen]
pub struct TransitEditor {
    inner: Editor,
}

impl Default for TransitEditor {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl TransitEditor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> TransitEditor {
        TransitEditor { inner: Editor::new() }
    }

    // ------------------------------------------------------------ files

    #[wasm_bindgen(js_name = newDocument)]
    pub fn new_document(&mut self) {
        let settings = self.inner.settings().clone();
        self.inner.replace_document(transit_core::Document { settings, ..Default::default() });
    }

    #[wasm_bindgen(js_name = loadTransit)]
    pub fn load_transit(&mut self, text: &str) -> Result<(), JsError> {
        let doc = io::from_transit(text).map_err(err)?;
        self.inner.replace_document(doc);
        Ok(())
    }

    #[wasm_bindgen(js_name = saveTransit)]
    pub fn save_transit(&self) -> String {
        io::to_transit(self.inner.document())
    }

    #[wasm_bindgen(js_name = exportSvg)]
    pub fn export_svg(&self) -> String {
        render::export_svg(self.inner.document())
    }

    // ---------------------------------------------------------- queries

    pub fn fragments(&self) -> String {
        json(&render::render_fragments(self.inner.document()))
    }

    pub fn tree(&self) -> String {
        json(&self.inner.tree())
    }

    pub fn settings(&self) -> String {
        json(self.inner.settings())
    }

    pub fn node(&self, id: f64) -> String {
        json(&self.inner.node(id as u64))
    }

    pub fn bounds(&self, ids_json: &str) -> Result<String, JsError> {
        Ok(json(&self.inner.bounds(&ids(ids_json)?)))
    }

    #[wasm_bindgen(js_name = lineHitAt)]
    pub fn line_hit_at(&self, x: i32, y: i32) -> String {
        json(&self.inner.line_hit_at(Cell::new(x, y)))
    }

    #[wasm_bindgen(js_name = nearestLineCell)]
    pub fn nearest_line_cell(&self, x: f64, y: f64, max_dist: f64) -> String {
        json(&self.inner.nearest_line_cell(Vec2::new(x, y), max_dist))
    }

    /// Markup for a line that is still being drawn with the pen.
    #[wasm_bindgen(js_name = previewLine)]
    pub fn preview_line(&self, points_json: &str, style_json: &str) -> Result<String, JsError> {
        let pts: Vec<Cell> = parse(points_json)?;
        let style: LineStyle = parse(style_json)?;
        let s = self.inner.settings();
        let norm = transit_core::geom::normalize_path(&pts, s.route_mode);
        Ok(render::line_svg(&norm, &style, s))
    }

    /// Route between two cells the way the pen does (`mode`: "straight_first" | "diagonal_first").
    pub fn route(&self, ax: i32, ay: i32, bx: i32, by: i32, mode: &str) -> Result<String, JsError> {
        let mode = parse(&format!("\"{mode}\""))?;
        Ok(json(&transit_core::geom::route(Cell::new(ax, ay), Cell::new(bx, by), mode)))
    }

    #[wasm_bindgen(js_name = previewPill)]
    pub fn preview_pill(&self, cells_json: &str, outer: &str, inner: &str) -> Result<String, JsError> {
        let cells: Vec<Cell> = parse(cells_json)?;
        let st = transit_core::Station::new(
            vec![],
            transit_core::StationShape::Pill { cells, outer_color: outer.into(), inner_color: inner.into() },
        );
        Ok(render::station_svg(&st, self.inner.settings()))
    }

    #[wasm_bindgen(js_name = iconLibrary)]
    pub fn icon_library() -> String {
        let list: Vec<Value> = transit_core::icons::ICONS
            .iter()
            .map(|i| serde_json::json!({"name": i.name, "label": i.label, "svg": transit_core::icons::markup(i.name, "#ffffff", "#1d4f91")}))
            .collect();
        json(&list)
    }

    // ---------------------------------------------------------- history

    pub fn undo(&mut self) -> bool {
        self.inner.undo()
    }

    pub fn redo(&mut self) -> bool {
        self.inner.redo()
    }

    #[wasm_bindgen(js_name = canUndo)]
    pub fn can_undo(&self) -> bool {
        self.inner.can_undo()
    }

    #[wasm_bindgen(js_name = canRedo)]
    pub fn can_redo(&self) -> bool {
        self.inner.can_redo()
    }

    #[wasm_bindgen(js_name = setHistoryLimit)]
    pub fn set_history_limit(&mut self, limit: usize) {
        self.inner.set_history_limit(limit)
    }

    pub fn begin(&mut self) {
        self.inner.begin()
    }

    pub fn rebase(&mut self) {
        self.inner.rebase()
    }

    pub fn commit(&mut self) {
        self.inner.commit()
    }

    pub fn cancel(&mut self) {
        self.inner.cancel()
    }

    // -------------------------------------------------------- mutations

    #[wasm_bindgen(js_name = setSettings)]
    pub fn set_settings(&mut self, settings_json: &str) -> Result<(), JsError> {
        self.inner.set_settings(parse(settings_json)?).map_err(err)
    }

    #[wasm_bindgen(js_name = createLine)]
    pub fn create_line(&mut self, points_json: &str, style_json: &str) -> Result<String, JsError> {
        let pts: Vec<Cell> = parse(points_json)?;
        let style: LineStyle = parse(style_json)?;
        Ok(json(&self.inner.create_line(&pts, style).map_err(err)?))
    }

    #[wasm_bindgen(js_name = addSegment)]
    pub fn add_segment(&mut self, line_id: f64, points_json: &str) -> Result<String, JsError> {
        let pts: Vec<Cell> = parse(points_json)?;
        Ok(json(&self.inner.add_segment(line_id as u64, &pts).map_err(err)?))
    }

    pub fn erase(&mut self, cells_json: &str) -> Result<usize, JsError> {
        let cells: Vec<Cell> = parse(cells_json)?;
        self.inner.erase(&cells).map_err(err)
    }

    /// `(px, py)` is the exact pointer position in cells (picks a tick's corner).
    #[wasm_bindgen(js_name = addStation)]
    pub fn add_station(&mut self, kind: &str, x: i32, y: i32, px: f64, py: f64, opts_json: &str) -> Result<String, JsError> {
        let kind: StationKind = parse(&format!("\"{kind}\""))?;
        let opts: StationOptions = parse(opts_json)?;
        Ok(json(&self.inner.add_station_at(kind, Cell::new(x, y), Vec2::new(px, py), &opts).map_err(err)?))
    }

    /// Markup of the station `addStation` would create, or "" when there is no line nearby.
    #[wasm_bindgen(js_name = previewStation)]
    pub fn preview_station(&self, kind: &str, x: i32, y: i32, px: f64, py: f64, opts_json: &str) -> Result<String, JsError> {
        let kind: StationKind = parse(&format!("\"{kind}\""))?;
        let opts: StationOptions = parse(opts_json)?;
        Ok(self
            .inner
            .station_preview(kind, Cell::new(x, y), Vec2::new(px, py), &opts)
            .map(|st| render::station_svg(&st, self.inner.settings()))
            .unwrap_or_default())
    }

    #[wasm_bindgen(js_name = addPill)]
    pub fn add_pill(&mut self, cells_json: &str, opts_json: &str) -> Result<String, JsError> {
        let cells: Vec<Cell> = parse(cells_json)?;
        let opts: StationOptions = parse(opts_json)?;
        Ok(json(&self.inner.add_pill(&cells, &opts).map_err(err)?))
    }

    #[wasm_bindgen(js_name = setStationKind)]
    pub fn set_station_kind(&mut self, id: f64, kind: &str) -> Result<(), JsError> {
        let kind: StationKind = parse(&format!("\"{kind}\""))?;
        self.inner.set_station_kind(id as u64, kind).map_err(err)
    }

    #[wasm_bindgen(js_name = moveStationTo)]
    pub fn move_station_to(&mut self, id: f64, x: f64, y: f64) -> Result<String, JsError> {
        Ok(json(&self.inner.move_station_to(id as u64, Vec2::new(x, y)).map_err(err)?))
    }

    #[wasm_bindgen(js_name = addText)]
    pub fn add_text(&mut self, text_json: &str) -> Result<f64, JsError> {
        let t: TextBox = parse(text_json)?;
        self.inner.add_text(t).map(|v| v as f64).map_err(err)
    }

    #[wasm_bindgen(js_name = addIcon)]
    pub fn add_icon(&mut self, icon_json: &str) -> Result<f64, JsError> {
        let i: Icon = parse(icon_json)?;
        self.inner.add_icon(i).map(|v| v as f64).map_err(err)
    }

    #[wasm_bindgen(js_name = updateContent)]
    pub fn update_content(&mut self, id: f64, patch_json: &str) -> Result<(), JsError> {
        let patch: Value = parse(patch_json)?;
        self.inner.update_content(id as u64, &patch).map_err(err)
    }

    pub fn rename(&mut self, id: f64, name: &str) -> Result<(), JsError> {
        self.inner.rename(id as u64, name).map_err(err)
    }

    #[wasm_bindgen(js_name = setVisible)]
    pub fn set_visible(&mut self, id: f64, visible: bool) -> Result<(), JsError> {
        self.inner.set_visible(id as u64, visible).map_err(err)
    }

    #[wasm_bindgen(js_name = setLocked)]
    pub fn set_locked(&mut self, id: f64, locked: bool) -> Result<(), JsError> {
        self.inner.set_locked(id as u64, locked).map_err(err)
    }

    #[wasm_bindgen(js_name = setExpanded)]
    pub fn set_expanded(&mut self, id: f64, expanded: bool) -> Result<(), JsError> {
        self.inner.set_expanded(id as u64, expanded).map_err(err)
    }

    pub fn delete(&mut self, ids_json: &str) -> Result<usize, JsError> {
        self.inner.delete(&ids(ids_json)?).map_err(err)
    }

    pub fn group(&mut self, ids_json: &str) -> Result<f64, JsError> {
        self.inner.group(&ids(ids_json)?, None).map(|v| v as f64).map_err(err)
    }

    pub fn ungroup(&mut self, id: f64) -> Result<String, JsError> {
        Ok(json(&self.inner.ungroup(id as u64).map_err(err)?))
    }

    #[wasm_bindgen(js_name = newGroup)]
    pub fn new_group(&mut self) -> Result<f64, JsError> {
        self.inner.new_group(None).map(|v| v as f64).map_err(err)
    }

    /// `parent` < 0 means the root.
    #[wasm_bindgen(js_name = moveNode)]
    pub fn move_node(&mut self, id: f64, parent: f64, index: usize) -> Result<(), JsError> {
        let parent = if parent < 0.0 { None } else { Some(parent as u64) };
        self.inner.move_node(id as u64, parent, index).map_err(err)
    }

    pub fn restack(&mut self, ids_json: &str, delta: i32) -> Result<(), JsError> {
        self.inner.restack(&ids(ids_json)?, delta as i64).map_err(err)
    }

    pub fn duplicate(&mut self, ids_json: &str) -> Result<String, JsError> {
        Ok(json(&self.inner.duplicate(&ids(ids_json)?).map_err(err)?))
    }

    /// `free` moves stations off their line (Alt-drag) instead of re-snapping them.
    pub fn translate(&mut self, ids_json: &str, dx: f64, dy: f64, free: bool) -> Result<(), JsError> {
        self.inner.translate(&ids(ids_json)?, dx, dy, free).map_err(err)
    }

    pub fn rotate(&mut self, ids_json: &str, deg: f64, px: f64, py: f64) -> Result<f64, JsError> {
        self.inner.rotate(&ids(ids_json)?, deg, Vec2::new(px, py)).map_err(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_api_roundtrip_native() {
        let mut ed = TransitEditor::new();
        let created = ed.create_line("[{\"x\":0,\"y\":0},{\"x\":5,\"y\":0}]", "{}").ok().unwrap();
        assert!(created.contains("group_id"));
        assert!(ed.fragments().contains("<path"));
        assert!(ed.can_undo());
        assert!(ed.undo());
        assert_eq!(ed.fragments(), "[]");
        let saved = ed.save_transit();
        let mut other = TransitEditor::new();
        assert!(other.load_transit(&saved).is_ok());
        assert!(TransitEditor::icon_library().contains("wheelchair"));
    }
}
