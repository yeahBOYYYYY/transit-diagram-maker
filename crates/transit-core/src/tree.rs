//! Helpers for navigating and editing the layer tree.

use crate::model::{Group, GroupKind, Id, Layer, Node};

/// Location of a node: the id of its parent group (`None` = root) and its index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    pub parent: Option<Id>,
    pub index: usize,
}

pub fn find(nodes: &[Node], id: Id) -> Option<&Node> {
    for n in nodes {
        if n.id() == id {
            return Some(n);
        }
        if let Node::Group(g) = n {
            if let Some(f) = find(&g.children, id) {
                return Some(f);
            }
        }
    }
    None
}

pub fn find_mut(nodes: &mut [Node], id: Id) -> Option<&mut Node> {
    for n in nodes.iter_mut() {
        if n.id() == id {
            return Some(n);
        }
        if let Node::Group(g) = n {
            if let Some(f) = find_mut(&mut g.children, id) {
                return Some(f);
            }
        }
    }
    None
}

pub fn find_layer(nodes: &[Node], id: Id) -> Option<&Layer> {
    match find(nodes, id) {
        Some(Node::Layer(l)) => Some(l),
        _ => None,
    }
}

pub fn find_layer_mut(nodes: &mut [Node], id: Id) -> Option<&mut Layer> {
    match find_mut(nodes, id) {
        Some(Node::Layer(l)) => Some(l),
        _ => None,
    }
}

pub fn find_group_mut(nodes: &mut [Node], id: Id) -> Option<&mut Group> {
    match find_mut(nodes, id) {
        Some(Node::Group(g)) => Some(g),
        _ => None,
    }
}

pub fn locate(nodes: &[Node], id: Id) -> Option<Location> {
    fn go(nodes: &[Node], parent: Option<Id>, id: Id) -> Option<Location> {
        for (i, n) in nodes.iter().enumerate() {
            if n.id() == id {
                return Some(Location { parent, index: i });
            }
            if let Node::Group(g) = n {
                if let Some(l) = go(&g.children, Some(g.id), id) {
                    return Some(l);
                }
            }
        }
        None
    }
    go(nodes, None, id)
}

/// Ids of all ancestors of `id`, nearest first.
pub fn ancestors(nodes: &[Node], id: Id) -> Vec<Id> {
    let mut out = Vec::new();
    let mut cur = id;
    while let Some(Location { parent: Some(p), .. }) = locate(nodes, cur) {
        out.push(p);
        cur = p;
    }
    out
}

pub fn is_descendant(nodes: &[Node], id: Id, ancestor: Id) -> bool {
    ancestors(nodes, id).contains(&ancestor)
}

pub fn children_mut(nodes: &mut Vec<Node>, parent: Option<Id>) -> Option<&mut Vec<Node>> {
    match parent {
        None => Some(nodes),
        Some(pid) => find_group_mut(nodes, pid).map(|g| &mut g.children),
    }
}

pub fn children(nodes: &[Node], parent: Option<Id>) -> Option<&[Node]> {
    match parent {
        None => Some(nodes),
        Some(pid) => match find(nodes, pid) {
            Some(Node::Group(g)) => Some(&g.children),
            _ => None,
        },
    }
}

pub fn remove(nodes: &mut Vec<Node>, id: Id) -> Option<(Node, Location)> {
    let loc = locate(nodes, id)?;
    let list = children_mut(nodes, loc.parent)?;
    Some((list.remove(loc.index), loc))
}

pub fn insert(nodes: &mut Vec<Node>, parent: Option<Id>, index: usize, node: Node) -> bool {
    match children_mut(nodes, parent) {
        Some(list) => {
            let i = index.min(list.len());
            list.insert(i, node);
            true
        }
        None => false,
    }
}

/// Visit every leaf layer in paint order (bottom to top), passing the effective
/// visibility and lock state inherited from ancestor groups.
pub fn walk_layers<'a>(nodes: &'a [Node], f: &mut impl FnMut(&'a Layer, bool, bool)) {
    fn go<'a>(nodes: &'a [Node], vis: bool, lock: bool, f: &mut impl FnMut(&'a Layer, bool, bool)) {
        for n in nodes {
            match n {
                Node::Layer(l) => f(l, vis && l.visible, lock || l.locked),
                Node::Group(g) => go(&g.children, vis && g.visible, lock || g.locked, f),
            }
        }
    }
    go(nodes, true, false, f)
}

pub fn walk_layers_mut(nodes: &mut [Node], f: &mut impl FnMut(&mut Layer)) {
    for n in nodes.iter_mut() {
        match n {
            Node::Layer(l) => f(l),
            Node::Group(g) => walk_layers_mut(&mut g.children, f),
        }
    }
}

/// Visit every node (groups and layers) depth-first.
pub fn walk_nodes<'a>(nodes: &'a [Node], f: &mut impl FnMut(&'a Node)) {
    for n in nodes {
        f(n);
        if let Node::Group(g) = n {
            walk_nodes(&g.children, f);
        }
    }
}

/// All leaf layer ids below (and including) the given node ids, in paint order, deduplicated.
pub fn leaf_ids(nodes: &[Node], ids: &[Id]) -> Vec<Id> {
    let mut out = Vec::new();
    walk_layers(nodes, &mut |l, _, _| {
        if ids.contains(&l.id) || ancestors(nodes, l.id).iter().any(|a| ids.contains(a)) {
            out.push(l.id);
        }
    });
    out
}

/// Effective lock state of a node (its own or any ancestor's).
pub fn is_locked(nodes: &[Node], id: Id) -> bool {
    find(nodes, id).map(|n| n.locked()).unwrap_or(false)
        || ancestors(nodes, id).iter().any(|a| find(nodes, *a).map(|n| n.locked()).unwrap_or(false))
}

/// Drop ids whose ancestor is also in the set, so operations on groups don't apply twice.
pub fn top_level_only(nodes: &[Node], ids: &[Id]) -> Vec<Id> {
    ids.iter()
        .copied()
        .filter(|id| find(nodes, *id).is_some())
        .filter(|id| !ancestors(nodes, *id).iter().any(|a| ids.contains(a)))
        .fold(Vec::new(), |mut acc, id| {
            if !acc.contains(&id) {
                acc.push(id);
            }
            acc
        })
}

/// Remove empty line/station groups that only existed to hold automatically
/// created content. User-made (generic) groups are kept even when empty.
pub fn prune_empty_groups(nodes: &mut Vec<Node>) {
    nodes.retain_mut(|n| match n {
        Node::Group(g) => {
            prune_empty_groups(&mut g.children);
            !g.children.is_empty() || g.kind == GroupKind::Generic
        }
        Node::Layer(_) => true,
    });
}

/// Paint-order position of every node, used to sort selections.
pub fn paint_order(nodes: &[Node]) -> Vec<Id> {
    let mut out = Vec::new();
    walk_nodes(nodes, &mut |n| out.push(n.id()));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    fn text_layer(id: Id) -> Node {
        Node::Layer(Layer {
            id,
            name: format!("L{id}"),
            visible: true,
            locked: false,
            content: Content::Text(TextBox::default()),
        })
    }

    fn group(id: Id, children: Vec<Node>) -> Node {
        Node::Group(Group {
            id,
            name: format!("G{id}"),
            visible: true,
            locked: false,
            expanded: true,
            kind: GroupKind::Generic,
            children,
        })
    }

    fn sample() -> Vec<Node> {
        vec![text_layer(1), group(2, vec![text_layer(3), group(4, vec![text_layer(5)])]), text_layer(6)]
    }

    #[test]
    fn find_and_locate_nested() {
        let t = sample();
        assert_eq!(find(&t, 5).unwrap().id(), 5);
        assert_eq!(locate(&t, 5), Some(Location { parent: Some(4), index: 0 }));
        assert_eq!(locate(&t, 6), Some(Location { parent: None, index: 2 }));
        assert!(find(&t, 99).is_none());
    }

    #[test]
    fn ancestors_nearest_first() {
        let t = sample();
        assert_eq!(ancestors(&t, 5), vec![4, 2]);
        assert!(is_descendant(&t, 5, 2));
        assert!(!is_descendant(&t, 1, 2));
    }

    #[test]
    fn remove_and_insert() {
        let mut t = sample();
        let (n, loc) = remove(&mut t, 3).unwrap();
        assert_eq!(loc, Location { parent: Some(2), index: 0 });
        assert!(insert(&mut t, None, 0, n));
        assert_eq!(t[0].id(), 3);
        assert!(!insert(&mut t, Some(1), 0, text_layer(9)), "cannot insert into a layer");
    }

    #[test]
    fn walk_inherits_visibility_and_lock() {
        let mut t = sample();
        if let Some(Node::Group(g)) = find_mut(&mut t, 2) {
            g.visible = false;
            g.locked = true;
        }
        let mut seen = Vec::new();
        walk_layers(&t, &mut |l, v, k| seen.push((l.id, v, k)));
        assert_eq!(seen, vec![(1, true, false), (3, false, true), (5, false, true), (6, true, false)]);
        assert!(is_locked(&t, 5));
        assert!(!is_locked(&t, 6));
    }

    #[test]
    fn leaf_ids_expand_groups() {
        let t = sample();
        assert_eq!(leaf_ids(&t, &[2]), vec![3, 5]);
        assert_eq!(leaf_ids(&t, &[6, 1]), vec![1, 6]);
    }

    #[test]
    fn top_level_only_drops_nested() {
        let t = sample();
        assert_eq!(top_level_only(&t, &[5, 2, 6, 6, 42]), vec![2, 6]);
    }

    #[test]
    fn prune_removes_empty_auto_groups_recursively() {
        let mut station = group(2, vec![]);
        if let Node::Group(g) = &mut station {
            g.kind = GroupKind::Station;
        }
        let mut line = group(1, vec![station]);
        if let Node::Group(g) = &mut line {
            g.kind = GroupKind::Line { line_id: 7 };
        }
        let mut t = vec![line, group(4, vec![]), text_layer(3)];
        prune_empty_groups(&mut t);
        let ids: Vec<Id> = t.iter().map(|n| n.id()).collect();
        assert_eq!(ids, vec![4, 3], "generic empty group is kept");
    }
}
