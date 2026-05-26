use std::collections::HashMap;

use super::body::Body;

/// A node in the body hierarchy tree.
/// Each node holds a `body_id` and references to children.
#[derive(Debug, Clone)]
pub struct HierarchyNode {
    pub body_id: u32,
    pub children: Vec<HierarchyNode>,
    pub depth: u32,
}

/// Builds a hierarchical display ordering from a flat list of bodies.
/// Returns nodes in depth-first order with their indentation depth.
#[derive(Debug, Clone)]
pub struct BodyHierarchy {
    roots: Vec<HierarchyNode>,
}

impl BodyHierarchy {
    /// Build hierarchy from a slice of bodies using `parent_id` relationships.
    ///
    /// 1. Root bodies have `parent_id == None`.
    /// 2. Children are recursively attached under their parent.
    /// 3. Children are sorted by `body_id` at each level.
    pub fn build(bodies: &[Body]) -> Self {
        // Group children by parent_id.
        let mut children_of: HashMap<u32, Vec<u32>> = HashMap::new();
        let mut root_ids: Vec<u32> = Vec::new();

        for body in bodies {
            match body.parent_id {
                Some(pid) => children_of.entry(pid).or_default().push(body.body_id),
                None => root_ids.push(body.body_id),
            }
        }

        root_ids.sort();

        // Sort each parent's children by body_id.
        for ids in children_of.values_mut() {
            ids.sort();
        }

        let roots = root_ids
            .iter()
            .map(|&id| Self::build_node(id, 0, &children_of))
            .collect();

        Self { roots }
    }

    /// Return `(body_id, depth)` pairs in display order (depth-first traversal).
    pub fn display_order(&self) -> Vec<(u32, u32)> {
        let mut result = Vec::new();
        for root in &self.roots {
            Self::collect_display_order(root, &mut result);
        }
        result
    }

    fn build_node(
        body_id: u32,
        depth: u32,
        children_of: &HashMap<u32, Vec<u32>>,
    ) -> HierarchyNode {
        let children = children_of
            .get(&body_id)
            .map(|ids| {
                ids.iter()
                    .map(|&child_id| Self::build_node(child_id, depth + 1, children_of))
                    .collect()
            })
            .unwrap_or_default();

        HierarchyNode {
            body_id,
            children,
            depth,
        }
    }

    fn collect_display_order(node: &HierarchyNode, result: &mut Vec<(u32, u32)>) {
        result.push((node.body_id, node.depth));
        for child in &node.children {
            Self::collect_display_order(child, result);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::body::Body;

    #[test]
    fn empty_bodies_produces_empty_hierarchy() {
        let h = BodyHierarchy::build(&[]);
        assert!(h.display_order().is_empty());
    }

    #[test]
    fn flat_bodies_are_all_roots() {
        let bodies = vec![
            Body::new(2, "B".into()),
            Body::new(1, "A".into()),
            Body::new(3, "C".into()),
        ];
        let order = BodyHierarchy::build(&bodies).display_order();
        assert_eq!(order, vec![(1, 0), (2, 0), (3, 0)]);
    }

    #[test]
    fn parent_child_depth_first() {
        let mut star = Body::new(0, "Star".into());
        star.parent_id = None;

        let mut planet = Body::new(1, "Planet".into());
        planet.parent_id = Some(0);

        let mut moon = Body::new(2, "Moon".into());
        moon.parent_id = Some(1);

        let bodies = vec![moon, star, planet]; // intentionally unordered
        let order = BodyHierarchy::build(&bodies).display_order();
        assert_eq!(order, vec![(0, 0), (1, 1), (2, 2)]);
    }
}
