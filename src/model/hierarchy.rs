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
    /// 3. Children are sorted by `sort_key` (naming convention order).
    pub fn build(bodies: &[Body]) -> Self {
        // Index bodies by id for sort_key lookup.
        let body_map: HashMap<u32, &Body> = bodies.iter().map(|b| (b.body_id, b)).collect();

        // Group children by parent_id.
        let mut children_of: HashMap<u32, Vec<u32>> = HashMap::new();
        let mut root_ids: Vec<u32> = Vec::new();

        for body in bodies {
            let has_parent_in_list = match body.parent_id {
                Some(pid) => body_map.contains_key(&pid),
                None => false,
            };
            if has_parent_in_list {
                children_of.entry(body.parent_id.unwrap()).or_default().push(body.body_id);
            } else {
                root_ids.push(body.body_id);
            }
        }

        root_ids.sort_by(|a, b| {
            let ak = body_map.get(a).map(|b| b.sort_key.as_str()).unwrap_or("");
            let bk = body_map.get(b).map(|b| b.sort_key.as_str()).unwrap_or("");
            ak.cmp(bk)
        });

        // Sort each parent's children by sort_key.
        for ids in children_of.values_mut() {
            ids.sort_by(|a, b| {
                let ak = body_map.get(a).map(|b| b.sort_key.as_str()).unwrap_or("");
                let bk = body_map.get(b).map(|b| b.sort_key.as_str()).unwrap_or("");
                ak.cmp(bk)
            });
        }

        let roots = root_ids
            .iter()
            .map(|&id| Self::build_node(id, 0, &children_of))
            .collect();

        Self { roots }
    }

    /// Return `(body_id, depth, is_last_sibling)` triples in display order (depth-first traversal).
    pub fn display_order(&self) -> Vec<(u32, u32, bool)> {
        let mut result = Vec::new();
        let root_count = self.roots.len();
        for (i, root) in self.roots.iter().enumerate() {
            Self::collect_display_order(root, &mut result, i == root_count - 1);
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

    fn collect_display_order(node: &HierarchyNode, result: &mut Vec<(u32, u32, bool)>, is_last: bool) {
        result.push((node.body_id, node.depth, is_last));
        let child_count = node.children.len();
        for (i, child) in node.children.iter().enumerate() {
            Self::collect_display_order(child, result, i == child_count - 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::body::Body;
    use crate::model::naming::parse_body_name;

    /// Create a body with sort_key derived from the naming parser.
    fn body_with_name(id: u32, short_name: &str) -> Body {
        let mut b = Body::new(id, short_name.into());
        b.short_name = short_name.into();
        b.sort_key = parse_body_name(short_name).sort_key;
        b
    }

    #[test]
    fn empty_bodies_produces_empty_hierarchy() {
        let h = BodyHierarchy::build(&[]);
        assert!(h.display_order().is_empty());
    }

    #[test]
    fn flat_bodies_are_all_roots() {
        let bodies = vec![
            body_with_name(2, "B"),
            body_with_name(1, "A"),
            body_with_name(3, "C"),
        ];
        let order = BodyHierarchy::build(&bodies).display_order();
        assert_eq!(order, vec![(1, 0, false), (2, 0, false), (3, 0, true)]);
    }

    #[test]
    fn parent_child_depth_first() {
        let mut star = body_with_name(0, "");
        star.parent_id = None;

        let mut planet = body_with_name(1, "1");
        planet.parent_id = Some(0);

        let mut moon = body_with_name(2, "1 a");
        moon.parent_id = Some(1);

        let bodies = vec![moon, star, planet]; // intentionally unordered
        let order = BodyHierarchy::build(&bodies).display_order();
        assert_eq!(order, vec![(0, 0, true), (1, 1, true), (2, 2, true)]);
    }

    #[test]
    fn missing_parent_becomes_temporary_root() {
        let mut star = body_with_name(0, "");
        star.parent_id = None;

        // Planet 1 is missing from the list!
        let mut moon = body_with_name(2, "1 a");
        moon.parent_id = Some(1); // Points to missing planet 1

        let bodies = vec![moon, star];
        let order = BodyHierarchy::build(&bodies).display_order();
        
        // Star is a root (depth 0). Moon has missing parent so it also becomes a temporary root (depth 0).
        assert_eq!(order, vec![(0, 0, false), (2, 0, true)]);
    }
}
