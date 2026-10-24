//! The editor: every document mutation goes through here so it can be recorded
//! on the undo/redo history.

use crate::error::{EditError, Result};
use crate::geom::{self, Cell, Dir8, Rect, Vec2};
use crate::model::*;
use crate::render;
use crate::tree;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

pub const DEFAULT_HISTORY_LIMIT: usize = 200;

#[derive(Debug, Clone)]
pub struct Editor {
    doc: Document,
    undo: Vec<Document>,
    redo: Vec<Document>,
    history_limit: usize,
    /// Snapshot taken when a transaction (e.g. a mouse drag) began.
    txn: Option<Document>,
}

/// Where a line passes through a cell.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct LineHit {
    pub layer_id: Id,
    pub line_id: Id,
    pub cell: Cell,
    pub dir: Dir8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct StationOptions {
    pub outer_color: String,
    pub inner_color: String,
    /// Tick color; `None` uses the line color.
    pub tick_color: Option<String>,
    pub tick_side: i8,
    pub tick_width: Option<f64>,
    pub tick_length: Option<f64>,
    /// Create a name label next to the station.
    pub label: bool,
    pub label_color: String,
}

impl Default for StationOptions {
    fn default() -> Self {
        StationOptions {
            outer_color: "#000000".into(),
            inner_color: "#ffffff".into(),
            tick_color: None,
            tick_side: 1,
            tick_width: None,
            tick_length: None,
            label: true,
            label_color: "#1a1a1a".into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct CreatedLine {
    pub group_id: Id,
    pub layer_id: Id,
    pub line_id: Id,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct CreatedStation {
    pub group_id: Id,
    pub station_id: Id,
    pub text_id: Option<Id>,
}

/// Light-weight view of the layer tree for the layers panel, top of the stack first.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TreeItem {
    pub id: Id,
    pub name: String,
    /// "group", "line", "station", "text" or "icon".
    pub kind: String,
    /// For groups: "generic", "line" or "station".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    pub visible: bool,
    pub locked: bool,
    pub expanded: bool,
    pub children: Vec<TreeItem>,
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

impl Editor {
    pub fn new() -> Self {
        Self::from_document(Document::default())
    }

    pub fn from_document(doc: Document) -> Self {
        Editor { doc, undo: Vec::new(), redo: Vec::new(), history_limit: DEFAULT_HISTORY_LIMIT, txn: None }
    }

    pub fn document(&self) -> &Document {
        &self.doc
    }

    pub fn settings(&self) -> &DocSettings {
        &self.doc.settings
    }

    /// Replace the whole document (open / new). Clears the history.
    pub fn replace_document(&mut self, doc: Document) {
        self.doc = doc;
        self.undo.clear();
        self.redo.clear();
        self.txn = None;
    }

    // ---------------------------------------------------------------- history

    pub fn set_history_limit(&mut self, limit: usize) {
        self.history_limit = limit.max(1);
        self.trim_history();
    }

    pub fn history_limit(&self) -> usize {
        self.history_limit
    }

    fn trim_history(&mut self) {
        if self.undo.len() > self.history_limit {
            let extra = self.undo.len() - self.history_limit;
            self.undo.drain(0..extra);
        }
    }

    fn record(&mut self, before: Document) {
        self.undo.push(before);
        self.redo.clear();
        self.trim_history();
    }

    /// Run a mutation atomically: on error nothing changes, on success the
    /// previous state is pushed to the undo stack (unless inside a transaction
    /// or nothing actually changed).
    fn apply<T>(&mut self, f: impl FnOnce(&mut Document) -> Result<T>) -> Result<T> {
        let before = self.doc.clone();
        match f(&mut self.doc) {
            Ok(v) => {
                if self.txn.is_none() && self.doc != before {
                    self.record(before);
                }
                Ok(v)
            }
            Err(e) => {
                self.doc = before;
                Err(e)
            }
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty() || self.txn.as_ref().is_some_and(|b| *b != self.doc)
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn undo(&mut self) -> bool {
        self.commit();
        match self.undo.pop() {
            Some(prev) => {
                let cur = std::mem::replace(&mut self.doc, prev);
                self.redo.push(cur);
                true
            }
            None => false,
        }
    }

    pub fn redo(&mut self) -> bool {
        self.commit();
        match self.redo.pop() {
            Some(next) => {
                let cur = std::mem::replace(&mut self.doc, next);
                self.undo.push(cur);
                self.trim_history();
                true
            }
            None => false,
        }
    }

    pub fn undo_depth(&self) -> usize {
        self.undo.len()
    }

    pub fn redo_depth(&self) -> usize {
        self.redo.len()
    }

    /// Start grouping several mutations into one history step.
    pub fn begin(&mut self) {
        if self.txn.is_none() {
            self.txn = Some(self.doc.clone());
        }
    }

    pub fn in_transaction(&self) -> bool {
        self.txn.is_some()
    }

    /// Reset the document to the state at the start of the transaction. Used by
    /// drags that recompute their effect from the original state on each move.
    pub fn rebase(&mut self) {
        if let Some(base) = &self.txn {
            self.doc = base.clone();
        }
    }

    /// Finish the transaction, recording one history step if anything changed.
    pub fn commit(&mut self) {
        if let Some(base) = self.txn.take() {
            if base != self.doc {
                self.record(base);
            }
        }
    }

    /// Abort the transaction and restore the original state.
    pub fn cancel(&mut self) {
        if let Some(base) = self.txn.take() {
            self.doc = base;
        }
    }

    // ------------------------------------------------------------- settings

    pub fn set_settings(&mut self, settings: DocSettings) -> Result<()> {
        self.apply(|d| {
            d.settings = settings.sanitized();
            Ok(())
        })
    }

    // -------------------------------------------------------------- queries

    pub fn node(&self, id: Id) -> Option<&Node> {
        tree::find(&self.doc.root, id)
    }

    pub fn layer(&self, id: Id) -> Option<&Layer> {
        tree::find_layer(&self.doc.root, id)
    }

    pub fn tree(&self) -> Vec<TreeItem> {
        fn item(n: &Node) -> TreeItem {
            match n {
                Node::Group(g) => TreeItem {
                    id: g.id,
                    name: g.name.clone(),
                    kind: "group".into(),
                    role: Some(
                        match g.kind {
                            GroupKind::Generic => "generic",
                            GroupKind::Line { .. } => "line",
                            GroupKind::Station => "station",
                        }
                        .into(),
                    ),
                    visible: g.visible,
                    locked: g.locked,
                    expanded: g.expanded,
                    children: g.children.iter().rev().map(item).collect(),
                },
                Node::Layer(l) => TreeItem {
                    id: l.id,
                    name: l.name.clone(),
                    kind: l.content.kind_name().into(),
                    role: None,
                    visible: l.visible,
                    locked: l.locked,
                    expanded: false,
                    children: Vec::new(),
                },
            }
        }
        self.doc.root.iter().rev().map(item).collect()
    }

    /// Bounds (pixels) of the given nodes' visible content.
    pub fn bounds(&self, ids: &[Id]) -> Option<Rect> {
        let leaves = tree::leaf_ids(&self.doc.root, ids);
        leaves
            .iter()
            .filter_map(|id| self.layer(*id))
            .filter_map(|l| render::content_bounds(&l.content, &self.doc.settings))
            .reduce(|a, b| a.union(b))
    }

    /// Position shown as X / Y for the given nodes, in cells: the anchor of a
    /// single object (text and icon position, station center, top-left cell
    /// center of a line), or the top-left-most anchor of several.
    pub fn position(&self, ids: &[Id]) -> Option<Vec2> {
        tree::leaf_ids(&self.doc.root, ids)
            .iter()
            .filter_map(|id| self.layer(*id))
            .filter_map(|l| content_anchor(&l.content))
            .reduce(|a, b| Vec2::new(a.x.min(b.x), a.y.min(b.y)))
    }

    /// Line segments that may be edited: visible and not locked.
    fn editable_segments(&self) -> Vec<&Layer> {
        let mut out = Vec::new();
        tree::walk_layers(&self.doc.root, &mut |l, vis, locked| {
            if vis && !locked && matches!(l.content, Content::Line(_)) {
                out.push(l);
            }
        });
        out
    }

    /// The topmost editable line passing through `cell`.
    pub fn line_hit_at(&self, cell: Cell) -> Option<LineHit> {
        self.editable_segments().into_iter().rev().find_map(|l| {
            let Content::Line(seg) = &l.content else { return None };
            let cells = geom::expand(&seg.points);
            let idx = cells.iter().position(|c| *c == cell)?;
            Some(LineHit { layer_id: l.id, line_id: seg.line_id, cell, dir: geom::direction_at(&cells, idx)? })
        })
    }

    /// The closest editable line cell to `pos` (cell units) within `max_dist`.
    pub fn nearest_line_cell(&self, pos: Vec2, max_dist: f64) -> Option<LineHit> {
        let mut best: Option<(f64, LineHit)> = None;
        for l in self.editable_segments().into_iter().rev() {
            let Content::Line(seg) = &l.content else { continue };
            let cells = geom::expand(&seg.points);
            for (i, c) in cells.iter().enumerate() {
                let d = c.center().sub(pos).len();
                if d <= max_dist && best.as_ref().is_none_or(|(bd, _)| d < *bd - 1e-9) {
                    if let Some(dir) = geom::direction_at(&cells, i) {
                        best = Some((d, LineHit { layer_id: l.id, line_id: seg.line_id, cell: *c, dir }));
                    }
                }
            }
        }
        best.map(|(_, h)| h)
    }

    // ---------------------------------------------------------------- lines

    pub fn create_line(&mut self, points: &[Cell], style: LineStyle) -> Result<CreatedLine> {
        self.apply(|d| {
            let pts = geom::normalize_path(points, d.settings.route_mode);
            if pts.len() < 2 {
                return Err(EditError::Invalid("a line needs at least two distinct cells".into()));
            }
            let line_id = d.alloc_id();
            let layer_id = d.alloc_id();
            let group_id = d.alloc_id();
            let count = count_groups(&d.root, |k| matches!(k, GroupKind::Line { .. })) + 1;
            let seg = Layer {
                id: layer_id,
                name: "Segment 1".into(),
                visible: true,
                locked: false,
                content: Content::Line(LineSegment { line_id, points: pts, style }),
            };
            d.root.push(Node::Group(Group {
                id: group_id,
                name: format!("Line {count}"),
                visible: true,
                locked: false,
                expanded: true,
                kind: GroupKind::Line { line_id },
                children: vec![Node::Layer(seg)],
            }));
            Ok(CreatedLine { group_id, layer_id, line_id })
        })
    }

    /// Add a disconnected segment to an existing line. It joins the line's
    /// group (below its stations) and takes the line's style.
    pub fn add_segment(&mut self, line_id: Id, points: &[Cell]) -> Result<CreatedLine> {
        self.apply(|d| {
            let pts = geom::normalize_path(points, d.settings.route_mode);
            if pts.len() < 2 {
                return Err(EditError::Invalid("a line needs at least two distinct cells".into()));
            }
            let group_id = line_group(&d.root, line_id).ok_or(EditError::NotFound(line_id))?;
            let style = line_style(&d.root, line_id).unwrap_or_default();
            let layer_id = d.alloc_id();
            let name = format!("Segment {}", count_segments(&d.root, line_id) + 1);
            let content = Content::Line(LineSegment { line_id, points: pts, style });
            let index = segments_end(&d.root, group_id);
            tree::insert(&mut d.root, Some(group_id), index, Node::Layer(Layer { id: layer_id, name, visible: true, locked: false, content }));
            Ok(CreatedLine { group_id, layer_id, line_id })
        })
    }

    /// Erase the given cells from every editable line, splitting segments into
    /// separate layers where they become discontinuous. Returns the number of
    /// segments touched.
    pub fn erase(&mut self, cells: &[Cell]) -> Result<usize> {
        let erase: HashSet<Cell> = cells.iter().copied().collect();
        let targets: Vec<(Id, Vec<Vec<Cell>>)> = self
            .editable_segments()
            .into_iter()
            .filter_map(|l| {
                let Content::Line(seg) = &l.content else { return None };
                let full = geom::expand(&seg.points);
                if !full.iter().any(|c| erase.contains(c)) {
                    return None;
                }
                let runs: Vec<Vec<Cell>> = full
                    .split(|c| erase.contains(c))
                    .filter(|r| r.len() >= 2)
                    .map(geom::compress)
                    .collect();
                Some((l.id, runs))
            })
            .collect();
        if targets.is_empty() {
            return Ok(0);
        }
        self.apply(|d| {
            for (id, runs) in &targets {
                let (node, loc) = tree::remove(&mut d.root, *id).ok_or(EditError::NotFound(*id))?;
                let Node::Layer(layer) = node else { continue };
                let Content::Line(seg) = &layer.content else { continue };
                let line_id = seg.line_id;
                let mut existing = count_segments(&d.root, line_id);
                for (i, run) in runs.iter().enumerate() {
                    let mut l = layer.clone();
                    if let Content::Line(s) = &mut l.content {
                        s.points = run.clone();
                    }
                    if i > 0 {
                        existing += 1;
                        l.id = d.alloc_id();
                        l.name = format!("Segment {}", existing + 1);
                    }
                    tree::insert(&mut d.root, loc.parent, loc.index + i, Node::Layer(l));
                }
            }
            tree::prune_empty_groups(&mut d.root);
            Ok(targets.len())
        })
    }

    // ------------------------------------------------------------- stations

    pub fn add_station(&mut self, kind: StationKind, cell: Cell, opts: &StationOptions) -> Result<CreatedStation> {
        self.add_station_at(kind, cell, cell.center(), opts)
    }

    /// Add a station on the line through (or near) `cell`. `pos` is the exact
    /// pointer position in cells; a tick goes into the cell corner nearest to it.
    pub fn add_station_at(&mut self, kind: StationKind, cell: Cell, pos: Vec2, opts: &StationOptions) -> Result<CreatedStation> {
        let (hit, shape) = self.station_shape_at(kind, cell, pos, opts)?;
        self.apply(|d| {
            let s = d.settings.clone();
            let normal = {
                let n = hit.dir.unit().perp();
                if kind == StationKind::Tick && opts.tick_side < 0 {
                    n.scale(-1.0)
                } else {
                    n
                }
            };
            let extra = if kind == StationKind::Tick { s.tick_length + s.line_width / 2.0 } else { 0.0 };
            let label_pos = shape.anchor().add(normal.scale(s.label_offset + extra));
            insert_station(d, hit.layer_id, vec![hit.line_id], shape, opts, label_pos, normal)
        })
    }

    /// The station `add_station_at` would create, without creating it (used
    /// for the station tool's hover preview).
    pub fn station_preview(&self, kind: StationKind, cell: Cell, pos: Vec2, opts: &StationOptions) -> Option<Station> {
        let (hit, shape) = self.station_shape_at(kind, cell, pos, opts).ok()?;
        Some(Station::new(vec![hit.line_id], shape))
    }

    fn station_shape_at(&self, kind: StationKind, cell: Cell, pos: Vec2, opts: &StationOptions) -> Result<(LineHit, StationShape)> {
        let hit = self
            .line_hit_at(cell)
            .or_else(|| self.nearest_line_cell(cell.center(), 1.5))
            .ok_or_else(|| EditError::Invalid("stations must be placed on a line".into()))?;
        let line_color = match self.layer(hit.layer_id).map(|l| &l.content) {
            Some(Content::Line(seg)) => seg.style.color.clone(),
            _ => "#000000".into(),
        };
        let s = &self.doc.settings;
        let shape = match kind {
            StationKind::Dot => StationShape::Dot {
                cell: hit.cell,
                outer_color: opts.outer_color.clone(),
                inner_color: opts.inner_color.clone(),
            },
            StationKind::Pill => StationShape::Pill {
                cells: vec![hit.cell],
                outer_color: opts.outer_color.clone(),
                inner_color: opts.inner_color.clone(),
            },
            StationKind::Tick => StationShape::Tick {
                cell: hit.cell,
                dir: hit.dir,
                side: if opts.tick_side < 0 { -1 } else { 1 },
                along: nearest_corner(hit.cell, hit.dir, pos),
                width: opts.tick_width.unwrap_or(s.tick_width),
                length: opts.tick_length.unwrap_or(s.tick_length),
                color: opts.tick_color.clone().unwrap_or(line_color),
            },
        };
        Ok((hit, shape))
    }

    /// Create a pill station covering a blob of cells. At least one cell must
    /// lie on an editable line; the station is coupled to every line it touches.
    pub fn add_pill(&mut self, cells: &[Cell], opts: &StationOptions) -> Result<CreatedStation> {
        let mut uniq: Vec<Cell> = Vec::new();
        for c in cells {
            if !uniq.contains(c) {
                uniq.push(*c);
            }
        }
        let mut line_ids: Vec<Id> = Vec::new();
        // The station goes into the group of the topmost touched line so it is
        // painted above every line it covers.
        let mut top_layer = None;
        for l in self.editable_segments() {
            let Content::Line(seg) = &l.content else { continue };
            let cells = geom::expand(&seg.points);
            if uniq.iter().any(|c| cells.contains(c)) {
                if !line_ids.contains(&seg.line_id) {
                    line_ids.push(seg.line_id);
                }
                top_layer = Some(l.id);
            }
        }
        let Some(anchor_layer) = top_layer else {
            return Err(EditError::Invalid("a pill station must touch at least one line".into()));
        };
        self.apply(|d| {
            let s = d.settings.clone();
            let top = uniq.iter().map(|c| c.y).min().unwrap_or(0);
            let shape = StationShape::Pill {
                cells: uniq.clone(),
                outer_color: opts.outer_color.clone(),
                inner_color: opts.inner_color.clone(),
            };
            let anchor = shape.anchor();
            let label_pos = Vec2::new(anchor.x, top as f64 + 0.5 - s.label_offset);
            insert_station(d, anchor_layer, line_ids, shape, opts, label_pos, Vec2::new(0.0, -1.0))
        })
    }

    /// Replace the cells of a pill station, e.g. to grow it into a neighboring
    /// cell or drop one. The blob must stay connected (8-way) and touch at
    /// least one line; the station is coupled to every line it covers.
    pub fn set_pill_cells(&mut self, id: Id, cells: &[Cell]) -> Result<()> {
        let mut uniq: Vec<Cell> = Vec::new();
        for c in cells {
            if !uniq.contains(c) {
                uniq.push(*c);
            }
        }
        if uniq.is_empty() {
            return Err(EditError::Invalid("a pill station needs at least one cell".into()));
        }
        if !geom::is_connected(&uniq) {
            return Err(EditError::Invalid("a pill station must stay in one piece".into()));
        }
        let mut touched: Vec<Id> = Vec::new();
        for l in self.editable_segments() {
            let Content::Line(seg) = &l.content else { continue };
            if !touched.contains(&seg.line_id) && geom::expand(&seg.points).iter().any(|c| uniq.contains(c)) {
                touched.push(seg.line_id);
            }
        }
        if touched.is_empty() {
            return Err(EditError::Invalid("a pill station must touch at least one line".into()));
        }
        self.apply(|d| {
            let layer = tree::find_layer_mut(&mut d.root, id).ok_or(EditError::NotFound(id))?;
            let Content::Station(st) = &mut layer.content else {
                return Err(EditError::Invalid("not a station".into()));
            };
            let StationShape::Pill { cells, .. } = &mut st.shape else {
                return Err(EditError::Invalid("only pill stations can change their cells".into()));
            };
            *cells = uniq;
            // Keep the existing coupling order, then add newly covered lines.
            st.line_ids.retain(|l| touched.contains(l));
            for l in touched {
                if !st.line_ids.contains(&l) {
                    st.line_ids.push(l);
                }
            }
            Ok(())
        })
    }

    /// Change a station's shape while keeping its position and colors.
    pub fn set_station_kind(&mut self, id: Id, kind: StationKind) -> Result<()> {
        let hit_dir = |doc: &Document, line_ids: &[Id], cell: Cell| snap_to_lines(doc, line_ids, cell.center()).map(|h| h.1);
        self.apply(|d| {
            let s = d.settings.clone();
            let snapshot = d.clone();
            let layer = tree::find_layer_mut(&mut d.root, id).ok_or(EditError::NotFound(id))?;
            let Content::Station(st) = &mut layer.content else {
                return Err(EditError::Invalid("not a station".into()));
            };
            let (outer, inner, color) = match &st.shape {
                StationShape::Dot { outer_color, inner_color, .. } | StationShape::Pill { outer_color, inner_color, .. } => {
                    (outer_color.clone(), inner_color.clone(), outer_color.clone())
                }
                StationShape::Tick { color, .. } => ("#000000".into(), "#ffffff".into(), color.clone()),
            };
            let cells = st.shape.cells();
            let cell = cells.first().copied().unwrap_or(Cell::new(0, 0));
            st.shape = match kind {
                StationKind::Dot => StationShape::Dot { cell, outer_color: outer, inner_color: inner },
                StationKind::Pill => StationShape::Pill { cells, outer_color: outer, inner_color: inner },
                StationKind::Tick => StationShape::Tick {
                    cell,
                    dir: hit_dir(&snapshot, &st.line_ids, cell).unwrap_or(Dir8::E),
                    side: 1,
                    along: -1,
                    width: s.tick_width,
                    length: s.tick_length,
                    color,
                },
            };
            Ok(())
        })
    }

    // ----------------------------------------------------------- text, icons

    pub fn add_text(&mut self, mut text: TextBox) -> Result<Id> {
        text.rotation = snap_rotation(text.rotation);
        text.pos = text.pos.snapped();
        self.apply(|d| {
            let id = d.alloc_id();
            let name = first_line_name(&text.text, "Text");
            d.root.push(Node::Layer(Layer { id, name, visible: true, locked: false, content: Content::Text(text) }));
            Ok(id)
        })
    }

    pub fn add_icon(&mut self, mut icon: Icon) -> Result<Id> {
        icon.rotation = snap_rotation(icon.rotation);
        icon.pos = icon.pos.snapped();
        self.apply(|d| {
            let id = d.alloc_id();
            let name = crate::icons::find(&icon.icon).map(|i| i.label).unwrap_or("Icon").to_string();
            d.root.push(Node::Layer(Layer { id, name, visible: true, locked: false, content: Content::Icon(icon) }));
            Ok(id)
        })
    }

    // ------------------------------------------------------ generic editing

    /// Merge a JSON patch into a layer's content. Style changes to a line
    /// segment are applied to every segment of the same line.
    pub fn update_content(&mut self, id: Id, patch: &Value) -> Result<()> {
        let obj = patch.as_object().ok_or_else(|| EditError::Invalid("patch must be an object".into()))?;
        if obj.contains_key("kind") || obj.contains_key("shape") || obj.contains_key("line_id") {
            return Err(EditError::Invalid("patch may not change the layer kind".into()));
        }
        self.apply(|d| {
            let layer = tree::find_layer(&d.root, id).ok_or(EditError::NotFound(id))?;
            let mut v = serde_json::to_value(&layer.content).map_err(|e| EditError::Invalid(e.to_string()))?;
            merge(&mut v, patch);
            let mut content: Content = serde_json::from_value(v).map_err(|e| EditError::Invalid(e.to_string()))?;
            match &mut content {
                Content::Text(t) => {
                    t.rotation = snap_rotation(t.rotation);
                    t.pos = t.pos.snapped();
                }
                Content::Icon(i) => {
                    i.rotation = snap_rotation(i.rotation);
                    i.pos = i.pos.snapped();
                }
                _ => {}
            }
            let rename = match (&layer.content, &content) {
                (Content::Text(old), Content::Text(new)) if layer.name == first_line_name(&old.text, "Text") => {
                    Some(first_line_name(&new.text, "Text"))
                }
                _ => None,
            };
            let line_style = match &content {
                Content::Line(seg) if obj.contains_key("style") => Some((seg.line_id, seg.style.clone())),
                _ => None,
            };
            if let Content::Line(seg) = &content {
                if seg.points.len() < 2 || geom::normalize_path(&seg.points, d.settings.route_mode) != seg.points {
                    return Err(EditError::Invalid("line points must be aligned to 8 directions".into()));
                }
            }
            let layer = tree::find_layer_mut(&mut d.root, id).unwrap();
            layer.content = content;
            if let Some(name) = rename {
                layer.name = name;
            }
            if let Some((line_id, style)) = line_style {
                tree::walk_layers_mut(&mut d.root, &mut |l| {
                    if let Content::Line(s) = &mut l.content {
                        if s.line_id == line_id {
                            s.style = style.clone();
                        }
                    }
                });
            }
            Ok(())
        })
    }

    pub fn rename(&mut self, id: Id, name: &str) -> Result<()> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(EditError::Invalid("name cannot be empty".into()));
        }
        self.apply(|d| {
            tree::find_mut(&mut d.root, id).ok_or(EditError::NotFound(id))?.set_name(name);
            Ok(())
        })
    }

    pub fn set_visible(&mut self, id: Id, visible: bool) -> Result<()> {
        self.apply(|d| {
            tree::find_mut(&mut d.root, id).ok_or(EditError::NotFound(id))?.set_visible(visible);
            Ok(())
        })
    }

    pub fn set_locked(&mut self, id: Id, locked: bool) -> Result<()> {
        self.apply(|d| {
            tree::find_mut(&mut d.root, id).ok_or(EditError::NotFound(id))?.set_locked(locked);
            Ok(())
        })
    }

    /// Expanding or collapsing a group is pure UI state: it is saved but not
    /// recorded in the history.
    pub fn set_expanded(&mut self, id: Id, expanded: bool) -> Result<()> {
        match tree::find_group_mut(&mut self.doc.root, id) {
            Some(g) => {
                g.expanded = expanded;
                for snap in self.undo.iter_mut().chain(self.redo.iter_mut()) {
                    if let Some(g) = tree::find_group_mut(&mut snap.root, id) {
                        g.expanded = expanded;
                    }
                }
                Ok(())
            }
            None => Err(EditError::NotFound(id)),
        }
    }

    pub fn delete(&mut self, ids: &[Id]) -> Result<usize> {
        self.apply(|d| {
            let ids = tree::top_level_only(&d.root, ids);
            let mut n = 0;
            for id in ids {
                if tree::is_locked(&d.root, id) {
                    continue;
                }
                if tree::remove(&mut d.root, id).is_some() {
                    n += 1;
                }
            }
            tree::prune_empty_groups(&mut d.root);
            Ok(n)
        })
    }

    /// Put the given nodes into a new group placed where the topmost of them was.
    pub fn group(&mut self, ids: &[Id], name: Option<&str>) -> Result<Id> {
        self.apply(|d| {
            let order = tree::paint_order(&d.root);
            let mut ids = tree::top_level_only(&d.root, ids);
            if ids.is_empty() {
                return Err(EditError::Invalid("nothing to group".into()));
            }
            ids.sort_by_key(|id| order.iter().position(|o| o == id));
            let top = *ids.last().unwrap();
            let target = tree::locate(&d.root, top).unwrap();
            let gid = d.alloc_id();
            // Insert an empty group right above the topmost node, then move nodes in.
            let count = count_groups(&d.root, |k| *k == GroupKind::Generic) + 1;
            tree::insert(
                &mut d.root,
                target.parent,
                target.index + 1,
                Node::Group(Group {
                    id: gid,
                    name: name.map(str::to_string).unwrap_or_else(|| format!("Group {count}")),
                    visible: true,
                    locked: false,
                    expanded: true,
                    kind: GroupKind::Generic,
                    children: Vec::new(),
                }),
            );
            for id in ids {
                let (node, _) = tree::remove(&mut d.root, id).ok_or(EditError::NotFound(id))?;
                let g = tree::find_group_mut(&mut d.root, gid).unwrap();
                g.children.push(node);
            }
            tree::prune_empty_groups(&mut d.root);
            Ok(gid)
        })
    }

    /// Replace a group by its children. Returns the ids of the released children.
    pub fn ungroup(&mut self, id: Id) -> Result<Vec<Id>> {
        self.apply(|d| {
            let (node, loc) = tree::remove(&mut d.root, id).ok_or(EditError::NotFound(id))?;
            let Node::Group(g) = node else {
                return Err(EditError::Invalid("only groups can be ungrouped".into()));
            };
            let ids = g.children.iter().map(|c| c.id()).collect();
            for (i, child) in g.children.into_iter().enumerate() {
                tree::insert(&mut d.root, loc.parent, loc.index + i, child);
            }
            Ok(ids)
        })
    }

    pub fn new_group(&mut self, name: Option<&str>) -> Result<Id> {
        self.apply(|d| {
            let id = d.alloc_id();
            let count = count_groups(&d.root, |k| *k == GroupKind::Generic) + 1;
            d.root.push(Node::Group(Group {
                id,
                name: name.map(str::to_string).unwrap_or_else(|| format!("Group {count}")),
                visible: true,
                locked: false,
                expanded: true,
                kind: GroupKind::Generic,
                children: Vec::new(),
            }));
            Ok(id)
        })
    }

    /// Move a node to `parent` (None = root) at `index` in bottom-to-top order.
    pub fn move_node(&mut self, id: Id, parent: Option<Id>, index: usize) -> Result<()> {
        self.apply(|d| {
            if let Some(p) = parent {
                if p == id || tree::is_descendant(&d.root, p, id) {
                    return Err(EditError::Invalid("cannot move a group into itself".into()));
                }
                if !matches!(tree::find(&d.root, p), Some(Node::Group(_))) {
                    return Err(EditError::Invalid("target is not a group".into()));
                }
            }
            let (mut node, loc) = tree::remove(&mut d.root, id).ok_or(EditError::NotFound(id))?;
            let index = if loc.parent == parent && loc.index < index { index - 1 } else { index };
            // Dropping segments into another line's group makes them part of that line.
            let target_line = parent.and_then(|p| match tree::find(&d.root, p) {
                Some(Node::Group(Group { kind: GroupKind::Line { line_id }, .. })) => Some(*line_id),
                _ => None,
            });
            match (target_line, node) {
                (Some(target), Node::Group(Group { kind: GroupKind::Line { line_id: source }, children, .. })) if source != target => {
                    // A whole line dropped onto another: merge it, keeping its stations.
                    let style = line_style(&d.root, target).unwrap_or_default();
                    let mut children = children;
                    join_line(&mut children, source, target, &style);
                    tree::walk_layers_mut(&mut d.root, &mut |l| relink_station(l, source, target));
                    for (i, child) in children.into_iter().enumerate() {
                        tree::insert(&mut d.root, parent, index + i, child);
                    }
                }
                (target, n) => {
                    node = n;
                    if let Some(target) = target {
                        let style = line_style(&d.root, target).unwrap_or_default();
                        let mut list = vec![node];
                        let sources: HashSet<Id> = line_ids_in(&list).into_iter().filter(|l| *l != target).collect();
                        for source in sources {
                            join_line(&mut list, source, target, &style);
                        }
                        node = list.pop().unwrap();
                    }
                    tree::insert(&mut d.root, parent, index, node);
                }
            }
            tree::prune_empty_groups(&mut d.root);
            Ok(())
        })
    }

    /// Move nodes up (positive) or down (negative) among their siblings.
    pub fn restack(&mut self, ids: &[Id], delta: i64) -> Result<()> {
        self.apply(|d| {
            let order = tree::paint_order(&d.root);
            let mut ids = tree::top_level_only(&d.root, ids);
            ids.sort_by_key(|id| order.iter().position(|o| o == id));
            if delta > 0 {
                ids.reverse();
            }
            for id in ids {
                let loc = tree::locate(&d.root, id).ok_or(EditError::NotFound(id))?;
                let list = tree::children_mut(&mut d.root, loc.parent).unwrap();
                let max = list.len() as i64 - 1;
                let target = (loc.index as i64 + delta).clamp(0, max) as usize;
                let node = list.remove(loc.index);
                list.insert(target, node);
            }
            Ok(())
        })
    }

    /// Copy nodes (with fresh ids) right above the originals, offset by one cell.
    pub fn duplicate(&mut self, ids: &[Id]) -> Result<Vec<Id>> {
        self.apply(|d| {
            let ids = tree::top_level_only(&d.root, ids);
            let mut created = Vec::new();
            for id in ids {
                let node = tree::find(&d.root, id).ok_or(EditError::NotFound(id))?.clone();
                let loc = tree::locate(&d.root, id).unwrap();
                let mut map = HashMap::new();
                let mut line_map = HashMap::new();
                let mut copy = vec![reassign_ids(d, node, &mut map, &mut line_map)];
                fix_references(&mut copy, &map, &line_map);
                let copy = copy.pop().unwrap();
                let new_id = copy.id();
                tree::insert(&mut d.root, loc.parent, loc.index + 1, copy);
                let leaves = tree::leaf_ids(&d.root, &[new_id]);
                translate_layers(d, &leaves, 1.0, 1.0, false);
                created.push(new_id);
            }
            Ok(created)
        })
    }

    // -------------------------------------------------------- transforms

    /// Move nodes by a delta in cell units. Lines move in whole cells, text and
    /// icons in tenths of a cell. Stations normally move in whole cells and
    /// re-snap onto their line; with `free` they leave the line and move in
    /// tenths (their coupling to the line is kept).
    pub fn translate(&mut self, ids: &[Id], dx: f64, dy: f64, free: bool) -> Result<()> {
        self.apply(|d| {
            let leaves = editable_leaves(d, ids);
            translate_layers(d, &leaves, dx, dy, free);
            Ok(())
        })
    }

    /// Rotate nodes by `deg` around `pivot` (cell units). Everything rotates in
    /// 45° steps; selections with grid-bound content (lines, stations) pivot
    /// around a cell center so they land back on the grid. Returns the angle
    /// actually applied.
    pub fn rotate(&mut self, ids: &[Id], deg: f64, pivot: Vec2) -> Result<f64> {
        self.apply(|d| {
            let leaves = editable_leaves(d, ids);
            let grid_bound = leaves.iter().any(|id| {
                matches!(tree::find_layer(&d.root, *id).map(|l| &l.content), Some(Content::Line(_) | Content::Station(_)))
            });
            let steps = (deg / 45.0).round() as i32;
            let angle = steps as f64 * 45.0;
            let pivot = if grid_bound { Cell::containing(pivot).center() } else { pivot };
            if steps == 0 {
                return Ok(0.0);
            }
            let route = d.settings.route_mode;
            let rot_cell = |c: Cell| Cell::containing(c.center().rotate_around(pivot, angle));
            let old_anchors = station_anchors(d);
            for id in &leaves {
                let Some(layer) = tree::find_layer_mut(&mut d.root, *id) else { continue };
                match &mut layer.content {
                    Content::Line(seg) => {
                        let pts: Vec<Cell> = seg.points.iter().map(|c| rot_cell(*c)).collect();
                        let norm = geom::normalize_path(&pts, route);
                        if norm.len() >= 2 {
                            seg.points = norm;
                        }
                    }
                    Content::Station(st) => {
                        st.offset = st.offset.rotate_around(Vec2::default(), angle).snapped();
                        match &mut st.shape {
                            StationShape::Dot { cell, .. } => *cell = rot_cell(*cell),
                            StationShape::Tick { cell, dir, .. } => {
                                *cell = rot_cell(*cell);
                                *dir = dir.rotated(steps);
                            }
                            StationShape::Pill { cells, .. } => {
                                let mut out: Vec<Cell> = Vec::new();
                                for c in cells.iter() {
                                    let r = rot_cell(*c);
                                    if !out.contains(&r) {
                                        out.push(r);
                                    }
                                }
                                *cells = out;
                            }
                        }
                    }
                    Content::Text(t) => {
                        t.pos = t.pos.rotate_around(pivot, angle).snapped();
                        t.rotation = snap_rotation(t.rotation + angle);
                    }
                    Content::Icon(i) => {
                        i.pos = i.pos.rotate_around(pivot, angle).snapped();
                        i.rotation = snap_rotation(i.rotation + angle);
                    }
                }
            }
            snap_stations(d, &leaves);
            move_orphan_labels(d, &leaves, &old_anchors);
            Ok(angle)
        })
    }

    /// Move a dot or tick station so it sits on the closest cell of its line.
    pub fn move_station_to(&mut self, id: Id, pos: Vec2) -> Result<Cell> {
        self.apply(|d| {
            let snapshot = d.clone();
            let old = station_anchors(d);
            let layer = tree::find_layer_mut(&mut d.root, id).ok_or(EditError::NotFound(id))?;
            let Content::Station(st) = &mut layer.content else {
                return Err(EditError::Invalid("not a station".into()));
            };
            let (cell, dir) = snap_to_lines(&snapshot, &st.line_ids, pos)
                .ok_or_else(|| EditError::Invalid("station is not coupled to an existing line".into()))?;
            st.offset = Vec2::default();
            if let StationShape::Pill { cells, .. } = &st.shape {
                let a = Cell::containing(st.shape.anchor());
                let moved: Vec<Cell> = cells.iter().map(|c| c.offset(cell.x - a.x, cell.y - a.y)).collect();
                if let StationShape::Pill { cells, .. } = &mut st.shape {
                    *cells = moved;
                }
            }
            match &mut st.shape {
                StationShape::Dot { cell: c, .. } => *c = cell,
                StationShape::Tick { cell: c, dir: dd, along, width, .. } => {
                    let (sc, sd, sa) = snap_tick(&snapshot, &st.line_ids, pos, *width).unwrap_or((cell, dir, -1));
                    *c = sc;
                    *dd = sd;
                    *along = sa;
                }
                StationShape::Pill { .. } => {}
            }
            move_orphan_labels(d, &[id], &old);
            Ok(cell)
        })
    }
}

// ------------------------------------------------------------------ helpers

fn content_anchor(c: &Content) -> Option<Vec2> {
    match c {
        Content::Line(seg) => Rect::from_points(seg.points.iter().map(|p| p.center())).map(|r| Vec2::new(r.x, r.y)),
        Content::Station(st) => Some(st.anchor()),
        Content::Text(t) => Some(t.pos),
        Content::Icon(i) => Some(i.pos),
    }
}

fn normalize_angle(a: f64) -> f64 {
    let mut a = a % 360.0;
    if a > 180.0 {
        a -= 360.0;
    }
    if a <= -180.0 {
        a += 360.0;
    }
    a
}

/// Rotations are always whole multiples of 45°, normalized to (-180, 180].
pub fn snap_rotation(a: f64) -> f64 {
    if !a.is_finite() {
        return 0.0;
    }
    normalize_angle((a / 45.0).round() * 45.0)
}

fn first_line_name(text: &str, fallback: &str) -> String {
    let first = text.lines().next().unwrap_or("").trim();
    if first.is_empty() {
        fallback.to_string()
    } else {
        first.chars().take(40).collect()
    }
}

fn count_groups(nodes: &[Node], pred: impl Fn(&GroupKind) -> bool) -> usize {
    let mut n = 0;
    tree::walk_nodes(nodes, &mut |node| {
        if let Node::Group(g) = node {
            if pred(&g.kind) {
                n += 1;
            }
        }
    });
    n
}

fn count_segments(nodes: &[Node], line_id: Id) -> usize {
    let mut n = 0;
    tree::walk_layers(nodes, &mut |l, _, _| {
        if matches!(&l.content, Content::Line(s) if s.line_id == line_id) {
            n += 1;
        }
    });
    n
}

/// The group holding the given line.
fn line_group(nodes: &[Node], line_id: Id) -> Option<Id> {
    let mut out = None;
    tree::walk_nodes(nodes, &mut |n| {
        if let Node::Group(g) = n {
            if out.is_none() && g.kind == (GroupKind::Line { line_id }) {
                out = Some(g.id);
            }
        }
    });
    out
}

fn line_style(nodes: &[Node], line_id: Id) -> Option<LineStyle> {
    let mut out = None;
    tree::walk_layers(nodes, &mut |l, _, _| {
        if let Content::Line(s) = &l.content {
            if out.is_none() && s.line_id == line_id {
                out = Some(s.style.clone());
            }
        }
    });
    out
}

fn line_ids_in(nodes: &[Node]) -> Vec<Id> {
    let mut out = Vec::new();
    tree::walk_layers(nodes, &mut |l, _, _| {
        if let Content::Line(s) = &l.content {
            if !out.contains(&s.line_id) {
                out.push(s.line_id);
            }
        }
    });
    out
}

/// Index right after the last segment in a line group, so new segments stay below its stations.
fn segments_end(nodes: &[Node], group_id: Id) -> usize {
    tree::children(nodes, Some(group_id))
        .map(|c| c.iter().rposition(|n| matches!(n, Node::Layer(Layer { content: Content::Line(_), .. }))).map_or(0, |i| i + 1))
        .unwrap_or(0)
}

/// Turn the segments of line `from` (within `nodes`) into segments of line `to`.
fn join_line(nodes: &mut [Node], from: Id, to: Id, style: &LineStyle) {
    tree::walk_layers_mut(nodes, &mut |l| {
        if let Content::Line(s) = &mut l.content {
            if s.line_id == from {
                s.line_id = to;
                s.style = style.clone();
            }
        }
        relink_station(l, from, to);
    });
}

fn relink_station(l: &mut Layer, from: Id, to: Id) {
    if let Content::Station(st) = &mut l.content {
        if st.line_ids.contains(&to) {
            st.line_ids.retain(|id| *id != from);
        } else {
            for id in st.line_ids.iter_mut().filter(|id| **id == from) {
                *id = to;
            }
        }
    }
}

fn count_stations(nodes: &[Node]) -> usize {
    let mut n = 0;
    tree::walk_layers(nodes, &mut |l, _, _| {
        if matches!(l.content, Content::Station(_)) {
            n += 1;
        }
    });
    n
}

/// Deep-merge `patch` into `target` (objects merge, everything else replaces).
pub fn merge(target: &mut Value, patch: &Value) {
    match (target, patch) {
        (Value::Object(t), Value::Object(p)) => {
            for (k, v) in p {
                match t.get_mut(k) {
                    Some(existing) if existing.is_object() && v.is_object() => merge(existing, v),
                    _ => {
                        t.insert(k.clone(), v.clone());
                    }
                }
            }
        }
        (t, p) => *t = p.clone(),
    }
}

fn insert_station(
    d: &mut Document,
    line_layer: Id,
    line_ids: Vec<Id>,
    shape: StationShape,
    opts: &StationOptions,
    label_pos: Vec2,
    normal: Vec2,
) -> Result<CreatedStation> {
    let s = d.settings.clone();
    let number = count_stations(&d.root) + 1;
    let name = format!("Station {number}");
    let station_id = d.alloc_id();
    let mut children = vec![Node::Layer(Layer {
        id: station_id,
        name: name.clone(),
        visible: true,
        locked: false,
        content: Content::Station(Station::new(line_ids, shape)),
    })];
    let mut text_id = None;
    if opts.label {
        let id = d.alloc_id();
        let align = if normal.x.abs() > normal.y.abs() + 1e-9 {
            if normal.x > 0.0 {
                TextAlign::Start
            } else {
                TextAlign::End
            }
        } else {
            TextAlign::Middle
        };
        children.push(Node::Layer(Layer {
            id,
            name: name.clone(),
            visible: true,
            locked: false,
            content: Content::Text(TextBox {
                pos: label_pos.snapped(),
                text: name.clone(),
                size: s.label_font_size,
                color: opts.label_color.clone(),
                font_family: s.label_font_family.clone(),
                align,
                station_id: Some(station_id),
                ..TextBox::default()
            }),
        }));
        text_id = Some(id);
    }
    let group_id = d.alloc_id();
    let group = Node::Group(Group {
        id: group_id,
        name,
        visible: true,
        locked: false,
        expanded: false,
        kind: GroupKind::Station,
        children,
    });
    // Stations live inside the line's group, above its segments.
    let parent = tree::locate(&d.root, line_layer).and_then(|l| l.parent);
    let len = tree::children(&d.root, parent).map(|c| c.len()).unwrap_or(0);
    tree::insert(&mut d.root, parent, len, group);
    Ok(CreatedStation { group_id, station_id, text_id })
}

/// Closest cell (and line direction there) on any segment of the given lines.
fn snap_to_lines(doc: &Document, line_ids: &[Id], pos: Vec2) -> Option<(Cell, Dir8)> {
    let mut best: Option<(f64, Cell, Dir8)> = None;
    tree::walk_layers(&doc.root, &mut |l, _, _| {
        let Content::Line(seg) = &l.content else { return };
        if !line_ids.contains(&seg.line_id) {
            return;
        }
        let cells = geom::expand(&seg.points);
        for (i, c) in cells.iter().enumerate() {
            let dist = c.center().sub(pos).len();
            if best.as_ref().is_none_or(|b| dist < b.0 - 1e-9) {
                if let Some(dir) = geom::direction_at(&cells, i) {
                    best = Some((dist, *c, dir));
                }
            }
        }
    });
    best.map(|(_, c, d)| (c, d))
}

/// Which corner of `cell` (along the line direction) is closest to `pos`.
fn nearest_corner(cell: Cell, dir: Dir8, pos: Vec2) -> i8 {
    if dir.is_diagonal() {
        return 0;
    }
    if pos.sub(cell.center()).dot(dir.unit()) > 1e-9 {
        1
    } else {
        -1
    }
}

/// Closest tick slot (cell, line direction, corner) on the given lines to a
/// tick center at `pos`. Every straight cell offers two corner slots.
fn snap_tick(doc: &Document, line_ids: &[Id], pos: Vec2, width: f64) -> Option<(Cell, Dir8, i8)> {
    let mut best: Option<(f64, Cell, Dir8, i8)> = None;
    tree::walk_layers(&doc.root, &mut |l, _, _| {
        let Content::Line(seg) = &l.content else { return };
        if !line_ids.contains(&seg.line_id) {
            return;
        }
        let cells = geom::expand(&seg.points);
        for (i, c) in cells.iter().enumerate() {
            let Some(dir) = geom::direction_at(&cells, i) else { continue };
            let corners: &[i8] = if dir.is_diagonal() { &[0] } else { &[-1, 1] };
            for &along in corners {
                let dist = geom::tick_center(*c, dir, along, width).sub(pos).len();
                if best.as_ref().is_none_or(|b| dist < b.0 - 1e-9) {
                    best = Some((dist, *c, dir, along));
                }
            }
        }
    });
    best.map(|(_, c, d, a)| (c, d, a))
}

fn editable_leaves(d: &Document, ids: &[Id]) -> Vec<Id> {
    tree::leaf_ids(&d.root, ids).into_iter().filter(|id| !tree::is_locked(&d.root, *id)).collect()
}

fn station_anchors(d: &Document) -> HashMap<Id, Vec2> {
    let mut out = HashMap::new();
    tree::walk_layers(&d.root, &mut |l, _, _| {
        if let Content::Station(st) = &l.content {
            out.insert(l.id, st.anchor());
        }
    });
    out
}

/// Labels follow their station when the station moved but the label itself
/// was not part of the operation.
fn move_orphan_labels(d: &mut Document, moved: &[Id], old: &HashMap<Id, Vec2>) {
    let new = station_anchors(d);
    tree::walk_layers_mut(&mut d.root, &mut |l| {
        if moved.contains(&l.id) {
            return;
        }
        if let Content::Text(t) = &mut l.content {
            if let Some(sid) = t.station_id {
                if !moved.contains(&sid) {
                    return;
                }
                if let (Some(a), Some(b)) = (old.get(&sid), new.get(&sid)) {
                    t.pos = t.pos.add(b.sub(*a)).snapped();
                }
            }
        }
    });
}

/// Snap moved dot/tick stations back onto their lines.
fn snap_stations(d: &mut Document, leaves: &[Id]) {
    let snapshot = d.clone();
    tree::walk_layers_mut(&mut d.root, &mut |l| {
        if !leaves.contains(&l.id) {
            return;
        }
        let Content::Station(st) = &mut l.content else { return };
        let line_ids = st.line_ids.clone();
        match &mut st.shape {
            StationShape::Dot { cell, .. } => {
                if let Some((c, _)) = snap_to_lines(&snapshot, &line_ids, cell.center()) {
                    *cell = c;
                }
            }
            StationShape::Tick { cell, dir, .. } => {
                if let Some((c, dd)) = snap_to_lines(&snapshot, &line_ids, cell.center()) {
                    *cell = c;
                    // Keep the tick on the same side when the line runs the other way.
                    if dd != *dir && dd != dir.opposite() {
                        *dir = dd;
                    }
                }
            }
            StationShape::Pill { .. } => {}
        }
    });
}

fn translate_layers(d: &mut Document, leaves: &[Id], dx: f64, dy: f64, free: bool) {
    let (ix, iy) = (dx.round() as i32, dy.round() as i32);
    let delta = Vec2::new(dx, dy);
    let old = station_anchors(d);
    // Stations whose lines move along keep their place relative to them.
    let mut moved_lines = HashSet::new();
    for id in leaves {
        if let Some(Content::Line(seg)) = tree::find_layer(&d.root, *id).map(|l| &l.content) {
            moved_lines.insert(seg.line_id);
        }
    }
    let mut snap = Vec::new();
    let mut ticks = Vec::new();
    for id in leaves {
        let Some(layer) = tree::find_layer_mut(&mut d.root, *id) else { continue };
        match &mut layer.content {
            Content::Line(seg) => {
                for p in seg.points.iter_mut() {
                    *p = p.offset(ix, iy);
                }
            }
            Content::Station(st) => {
                let with_line = !st.line_ids.is_empty() && st.line_ids.iter().all(|l| moved_lines.contains(l));
                if !free && !with_line && st.shape.kind() == StationKind::Tick {
                    // Ticks hop between cell corners, so they follow the exact delta.
                    st.offset = Vec2::default();
                    ticks.push((*id, st.shape.anchor().add(delta)));
                    continue;
                }
                let (cx, cy) = if free && !with_line { (0, 0) } else { (ix, iy) };
                match &mut st.shape {
                    StationShape::Dot { cell, .. } | StationShape::Tick { cell, .. } => *cell = cell.offset(cx, cy),
                    StationShape::Pill { cells, .. } => {
                        for c in cells.iter_mut() {
                            *c = c.offset(cx, cy);
                        }
                    }
                }
                if free {
                    st.offset = st.offset.add(delta).sub(Vec2::new(cx as f64, cy as f64)).snapped();
                } else {
                    if !with_line {
                        st.offset = Vec2::default();
                    }
                    snap.push(*id);
                }
            }
            Content::Text(t) => t.pos = t.pos.add(delta).snapped(),
            Content::Icon(i) => i.pos = i.pos.add(delta).snapped(),
        }
    }
    snap_stations(d, &snap);
    let snapshot = d.clone();
    for (id, target) in ticks {
        let Some(Content::Station(st)) = tree::find_layer_mut(&mut d.root, id).map(|l| &mut l.content) else { continue };
        let line_ids = st.line_ids.clone();
        if let StationShape::Tick { cell, dir, along, width, .. } = &mut st.shape {
            if let Some((c, dd, a)) = snap_tick(&snapshot, &line_ids, target, *width) {
                *cell = c;
                // Keep the tick on the same side when the line runs the other way.
                if dd == dir.opposite() {
                    *along = -a;
                } else {
                    *dir = dd;
                    *along = a;
                }
            }
        }
    }
    move_orphan_labels(d, leaves, &old);
}

fn reassign_ids(d: &mut Document, node: Node, map: &mut HashMap<Id, Id>, line_map: &mut HashMap<Id, Id>) -> Node {
    match node {
        Node::Layer(mut l) => {
            let nid = d.alloc_id();
            map.insert(l.id, nid);
            l.id = nid;
            if let Content::Line(seg) = &mut l.content {
                let next = line_map.entry(seg.line_id).or_insert_with(|| {
                    let id = d.next_id;
                    d.next_id += 1;
                    id
                });
                seg.line_id = *next;
            }
            Node::Layer(l)
        }
        Node::Group(mut g) => {
            let nid = d.alloc_id();
            map.insert(g.id, nid);
            g.id = nid;
            if let GroupKind::Line { line_id } = &mut g.kind {
                let next = line_map.entry(*line_id).or_insert_with(|| {
                    let id = d.next_id;
                    d.next_id += 1;
                    id
                });
                *line_id = *next;
            }
            g.children = g.children.into_iter().map(|c| reassign_ids(d, c, map, line_map)).collect();
            Node::Group(g)
        }
    }
}

fn fix_references(nodes: &mut [Node], map: &HashMap<Id, Id>, line_map: &HashMap<Id, Id>) {
    tree::walk_layers_mut(nodes, &mut |l| match &mut l.content {
        Content::Station(st) => {
            for id in st.line_ids.iter_mut() {
                if let Some(n) = line_map.get(id) {
                    *id = *n;
                }
            }
        }
        Content::Text(t) => {
            if let Some(sid) = t.station_id {
                if let Some(n) = map.get(&sid) {
                    t.station_id = Some(*n);
                }
            }
        }
        _ => {}
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn merge_is_deep() {
        let mut v = json!({"a": 1, "b": {"c": 2, "d": 3}});
        merge(&mut v, &json!({"b": {"c": 5}, "e": [1]}));
        assert_eq!(v, json!({"a": 1, "b": {"c": 5, "d": 3}, "e": [1]}));
    }

    #[test]
    fn normalize_angle_range() {
        assert_eq!(normalize_angle(190.0), -170.0);
        assert_eq!(normalize_angle(-180.0), 180.0);
        assert_eq!(normalize_angle(720.0), 0.0);
    }

    #[test]
    fn snap_rotation_to_45() {
        assert_eq!(snap_rotation(30.0), 45.0);
        assert_eq!(snap_rotation(-200.0), 180.0);
        assert_eq!(snap_rotation(22.0), 0.0);
        assert_eq!(snap_rotation(f64::NAN), 0.0);
    }

    #[test]
    fn first_line_name_truncates() {
        assert_eq!(first_line_name("  Hello\nWorld", "x"), "Hello");
        assert_eq!(first_line_name("   ", "x"), "x");
        assert_eq!(first_line_name(&"a".repeat(80), "x").len(), 40);
    }
}
