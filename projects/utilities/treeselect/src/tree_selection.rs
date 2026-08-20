/// A resolved tree selection: per-node `selected` and `visible` flags plus the
/// match-root indices, derived from match flags and parent links.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreeSelection {
    /// True where the node matched.
    selected: Vec<bool>,

    /// True where the node matched or leads to a match.
    visible: Vec<bool>,

    /// Matched nodes whose parent is unmatched, in index order: the entry
    /// point of each selected subtree.
    match_roots: Vec<usize>,
}

impl TreeSelection {
    /// Resolves `matched` flags over the tree described by `parents`, a
    /// per-node parent index (`None` at a root). Each matched node's ancestor
    /// chain becomes visible. A matched node whose parent is unmatched is a
    /// match root, so a deselected middle node starts fresh roots below it.
    /// The inputs line up by node; their lengths must match.
    pub fn from_matches(matched: Vec<bool>, parents: &[Option<usize>]) -> Self {
        assert_eq!(
            matched.len(),
            parents.len(),
            "matched and parents must line up"
        );

        // Each walk stops at the first already-visible ancestor, whose chain
        // is complete or pending its own walk, so marking stays linear.
        let mut visible = matched.clone();
        for index in 0..matched.len() {
            if !matched[index] {
                continue;
            }

            let mut ancestor = parents[index];
            while let Some(node) = ancestor {
                if visible[node] {
                    break;
                }
                visible[node] = true;
                ancestor = parents[node];
            }
        }

        let match_roots = matched
            .iter()
            .enumerate()
            .filter(|&(_, &is_matched)| is_matched)
            .filter(|&(index, _)| match parents[index] {
                Some(parent) => !matched[parent],
                None => true,
            })
            .map(|(index, _)| index)
            .collect();

        Self {
            selected: matched,
            visible,
            match_roots,
        }
    }

    /// True where the node matched.
    pub fn selected(&self) -> &[bool] {
        &self.selected
    }

    /// True where the node matched or leads to a match.
    pub fn visible(&self) -> &[bool] {
        &self.visible
    }

    /// Matched nodes whose parent is unmatched, in index order.
    pub fn match_roots(&self) -> &[usize] {
        &self.match_roots
    }
}

#[cfg(test)]
mod tests {
    use crate::TreeSelection;

    /// The test forest: a three-node chain (0 -> 1 -> 2) and a lone root (3).
    fn parents() -> Vec<Option<usize>> {
        vec![None, Some(0), Some(1), None]
    }

    #[test]
    fn a_matched_root_subtree_yields_one_match_root() {
        let selection = TreeSelection::from_matches(vec![true, true, true, false], &parents());

        assert_eq!(selection.selected(), &[true, true, true, false]);
        assert_eq!(selection.visible(), &[true, true, true, false]);
        assert_eq!(selection.match_roots(), &[0]);
    }

    #[test]
    fn a_nested_match_marks_its_ancestor_chain_visible() {
        let selection = TreeSelection::from_matches(vec![false, false, true, false], &parents());

        assert_eq!(selection.selected(), &[false, false, true, false]);
        assert_eq!(selection.visible(), &[true, true, true, false]);
        assert_eq!(selection.match_roots(), &[2]);
    }

    #[test]
    fn disjoint_matches_yield_a_match_root_each() {
        let selection = TreeSelection::from_matches(vec![false, true, true, true], &parents());

        assert_eq!(selection.visible(), &[true, true, true, true]);
        assert_eq!(selection.match_roots(), &[1, 3]);
    }

    #[test]
    fn a_deselected_middle_node_starts_a_fresh_match_root_below_it() {
        // 2 is a match root by the unmatched-parent rule despite its matched
        // grandparent; a no-matched-ancestor rule would fold it under 0.
        let selection = TreeSelection::from_matches(vec![true, false, true, false], &parents());

        assert_eq!(selection.visible(), &[true, true, true, false]);
        assert_eq!(selection.match_roots(), &[0, 2]);
    }
}
