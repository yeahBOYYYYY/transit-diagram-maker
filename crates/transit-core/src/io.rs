//! `.transit` file format: a versioned JSON envelope around the document.

use crate::error::{EditError, Result};
use crate::geom;
use crate::model::*;
use crate::tree;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const FORMAT: &str = "transit-diagram";
pub const VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
struct Envelope {
    format: String,
    version: u32,
    document: Document,
}

pub fn to_transit(doc: &Document) -> String {
    let env = Envelope { format: FORMAT.into(), version: VERSION, document: doc.clone() };
    serde_json::to_string_pretty(&env).expect("document serialization cannot fail")
}

pub fn from_transit(text: &str) -> Result<Document> {
    let env: Envelope = serde_json::from_str(text).map_err(|e| EditError::Parse(e.to_string()))?;
    if env.format != FORMAT {
        return Err(EditError::Parse(format!("unexpected format '{}'", env.format)));
    }
    if env.version > VERSION {
        return Err(EditError::Parse(format!("file version {} is newer than supported ({VERSION})", env.version)));
    }
    validate(env.document)
}

/// Repair or reject anything that would break editing: duplicate ids, stale
/// id counters, malformed lines and out-of-range settings.
pub fn validate(mut doc: Document) -> Result<Document> {
    let mut seen = HashSet::new();
    let mut max_id = 0;
    let mut dup = None;
    tree::walk_nodes(&doc.root, &mut |n| {
        max_id = max_id.max(n.id());
        if !seen.insert(n.id()) {
            dup = Some(n.id());
        }
    });
    tree::walk_layers(&doc.root, &mut |l, _, _| {
        if let Content::Line(seg) = &l.content {
            max_id = max_id.max(seg.line_id);
        }
    });
    if let Some(id) = dup {
        return Err(EditError::Parse(format!("duplicate id {id}")));
    }
    doc.next_id = doc.next_id.max(max_id + 1);
    doc.settings = doc.settings.sanitized();
    let route = doc.settings.route_mode;
    tree::walk_layers_mut(&mut doc.root, &mut |l| {
        if let Content::Line(seg) = &mut l.content {
            seg.points = geom::normalize_path(&seg.points, route);
        }
    });
    remove_degenerate(&mut doc.root);
    Ok(doc)
}

fn remove_degenerate(nodes: &mut Vec<Node>) {
    nodes.retain_mut(|n| match n {
        Node::Group(g) => {
            remove_degenerate(&mut g.children);
            true
        }
        Node::Layer(l) => !matches!(&l.content, Content::Line(s) if s.points.len() < 2)
            && !matches!(&l.content, Content::Station(st) if st.shape.cells().is_empty()),
    });
}
