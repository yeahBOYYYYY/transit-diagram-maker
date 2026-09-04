use crate::coord::Aabb;
use crate::graph::NodeId;

/// Represents an entry in the spatial BVH for collision detection
#[derive(Debug, Clone)]
pub struct SpatialItem {
    pub id: NodeId,
    pub aabb: Aabb,
}

/// A 2D Bounding Volume Hierarchy (BVH) node for fast spatial collision queries
#[derive(Debug, Clone)]
pub enum BvhNode {
    Leaf {
        items: Vec<SpatialItem>,
        bounds: Aabb,
    },
    Internal {
        left: Box<BvhNode>,
        right: Box<BvhNode>,
        bounds: Aabb,
    },
}

impl BvhNode {
    pub fn bounds(&self) -> Aabb {
        match self {
            BvhNode::Leaf { bounds, .. } => *bounds,
            BvhNode::Internal { bounds, .. } => *bounds,
        }
    }

    /// Recursively build BVH tree from items
    pub fn build(mut items: Vec<SpatialItem>, max_leaf_size: usize) -> Option<BvhNode> {
        if items.is_empty() {
            return None;
        }

        let mut overall_bounds = items[0].aabb;
        for item in &items[1..] {
            overall_bounds = overall_bounds.union(&item.aabb);
        }

        if items.len() <= max_leaf_size {
            return Some(BvhNode::Leaf {
                items,
                bounds: overall_bounds,
            });
        }

        // Split along the axis of greatest extent
        let width = overall_bounds.width();
        let height = overall_bounds.height();

        if width >= height {
            items.sort_by(|a, b| {
                a.aabb
                    .center()
                    .x
                    .partial_cmp(&b.aabb.center().x)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        } else {
            items.sort_by(|a, b| {
                a.aabb
                    .center()
                    .y
                    .partial_cmp(&b.aabb.center().y)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        let mid = items.len() / 2;
        let right_items = items.split_off(mid);

        let left_child = Self::build(items, max_leaf_size)?;
        let right_child = Self::build(right_items, max_leaf_size)?;

        Some(BvhNode::Internal {
            bounds: overall_bounds,
            left: Box::new(left_child),
            right: Box::new(right_child),
        })
    }

    /// Query intersecting pairs within this subtree
    pub fn find_self_intersections(&self, collisions: &mut Vec<NodeId>) {
        match self {
            BvhNode::Leaf { items, .. } => {
                let n = items.len();
                for i in 0..n {
                    for j in (i + 1)..n {
                        if items[i].aabb.intersects(&items[j].aabb) {
                            collisions.push(items[i].id);
                            collisions.push(items[j].id);
                        }
                    }
                }
            }
            BvhNode::Internal { left, right, .. } => {
                left.find_self_intersections(collisions);
                right.find_self_intersections(collisions);
                Self::find_cross_intersections(left, right, collisions);
            }
        }
    }

    fn find_cross_intersections(a: &BvhNode, b: &BvhNode, collisions: &mut Vec<NodeId>) {
        if !a.bounds().intersects(&b.bounds()) {
            return;
        }

        match (a, b) {
            (BvhNode::Leaf { items: items_a, .. }, BvhNode::Leaf { items: items_b, .. }) => {
                for item_a in items_a {
                    for item_b in items_b {
                        if item_a.id != item_b.id && item_a.aabb.intersects(&item_b.aabb) {
                            collisions.push(item_a.id);
                            collisions.push(item_b.id);
                        }
                    }
                }
            }
            (BvhNode::Internal { left, right, .. }, leaf @ BvhNode::Leaf { .. }) => {
                Self::find_cross_intersections(left, leaf, collisions);
                Self::find_cross_intersections(right, leaf, collisions);
            }
            (leaf @ BvhNode::Leaf { .. }, BvhNode::Internal { left, right, .. }) => {
                Self::find_cross_intersections(leaf, left, collisions);
                Self::find_cross_intersections(leaf, right, collisions);
            }
            (
                BvhNode::Internal {
                    left: left_a,
                    right: right_a,
                    ..
                },
                BvhNode::Internal {
                    left: left_b,
                    right: right_b,
                    ..
                },
            ) => {
                Self::find_cross_intersections(left_a, left_b, collisions);
                Self::find_cross_intersections(left_a, right_b, collisions);
                Self::find_cross_intersections(right_a, left_b, collisions);
                Self::find_cross_intersections(right_a, right_b, collisions);
            }
        }
    }
}

/// Detects all colliding text node IDs using spatial partitioning
pub fn detect_text_collisions(text_items: Vec<SpatialItem>) -> Vec<NodeId> {
    if text_items.len() < 2 {
        return Vec::new();
    }

    if let Some(root) = BvhNode::build(text_items, 4) {
        let mut colliding_ids = Vec::new();
        root.find_self_intersections(&mut colliding_ids);
        colliding_ids.sort_unstable();
        colliding_ids.dedup();
        colliding_ids
    } else {
        Vec::new()
    }
}
