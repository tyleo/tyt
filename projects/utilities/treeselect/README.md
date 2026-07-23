# treeselect

Tree selection: resolve per-node match flags over parent links into selected
and visible flags plus match roots.

```rust
use treeselect::TreeSelection;

// The tree: house (0) -> door (1) -> knob (2), and a lone shed (3).
let parents = [None, Some(0), Some(1), None];

// Whether each node matched, from any predicate: here, just `door`.
let matched = vec![false, true, false, false];

let selection = TreeSelection::from_matches(matched, &parents);

assert_eq!(selection.selected(), &[false, true, false, false]);
assert_eq!(selection.visible(), &[true, true, false, false]);
assert_eq!(selection.match_roots(), &[1]);
```

Each accessor answers one question:

1. `selected()`: did the node match? The input flags, unchanged.
2. `visible()`: should the node print? Matching `door` marks `house` visible
   too, since the chain above a match stays on screen.
3. `match_roots()`: where does a selected subtree start? At a matched node
   whose parent is unmatched, so a deselected middle node starts fresh roots
   below it.

Subtree inclusion stays with the matcher: to select a whole branch, flag its
descendants as matched too. The crate has no dependencies.

## With pathspec and treegrid

treeselect is the middle of a select-then-render pipeline. The command that
owns the tree composes all three crates:

1. Match each node's logical path, for example with `pathspec`'s
   gitignore-style globs:

   ```rust
   let globs = GitIgnoreRegex::from_spans_ignore_inert(&["door/**"])?;
   let matched: Vec<bool> = paths
       .iter()
       .map(|path| is_file_path_match(&globs, path) == Some(true))
       .collect();
   ```

2. Resolve the selection:

   ```rust
   let selection = TreeSelection::from_matches(matched, &parents);
   ```

3. Populate the renderer, for example a `treegrid` forest, keeping the nodes
   whose `visible` flag is set and anchoring a collapsed listing at the
   `match_roots`:

   ```rust
   let mut grid = TreeGrid::new();
   for node in tree_nodes_in_order() {
       if selection.visible()[node.index] {
           add_to_grid(&mut grid, node);
       }
   }
   let output = grid.render_hierarchy(&TreeGridHierarchyOptions::default());
   ```
