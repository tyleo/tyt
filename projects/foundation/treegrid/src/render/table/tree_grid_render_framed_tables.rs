use crate::{
    BTreeGridNode, TreeGrid, TreeGridCells, TreeGridTableLabelMode, TreeGridTableShapeKind,
    render::{Cell, TableFrame},
};
use branded_id::U32Id;

/// The table walk the table layouts share: the blocks a shape yields,
/// each heading and table drawn by a frame.
pub trait TreeGridRenderFramedTables {
    /// Renders the headings and tables of `shape` through `frame`, the
    /// nested headings spending the ancestor path per `label`. Blocks
    /// join with a blank line; an empty grid renders as an empty string.
    fn render_framed_tables<F: TableFrame>(
        &self,
        shape: TreeGridTableShapeKind,
        label: TreeGridTableLabelMode,
        frame: &F,
    ) -> String;
}

impl<C: TreeGridCells> TreeGridRenderFramedTables for TreeGrid<C> {
    fn render_framed_tables<F: TableFrame>(
        &self,
        shape: TreeGridTableShapeKind,
        label: TreeGridTableLabelMode,
        frame: &F,
    ) -> String {
        let blocks = match shape {
            TreeGridTableShapeKind::Flat => flat_blocks(self, frame),
            TreeGridTableShapeKind::Nested => nested_blocks(self, label, frame),
            TreeGridTableShapeKind::Records => records_blocks(self, frame),
        };
        if blocks.is_empty() {
            String::new()
        } else {
            format!("{}\n", blocks.join("\n\n"))
        }
    }
}

/// The nested blocks: headings and group tables in the grouping
/// walk's order.
fn nested_blocks<C: TreeGridCells, F: TableFrame>(
    grid: &TreeGrid<C>,
    label: TreeGridTableLabelMode,
    frame: &F,
) -> Vec<String> {
    let mut blocks: Vec<String> = Vec::new();
    // Groups arrive depth-first with parents before children, so a
    // depth-indexed stack of cumulative paths recovers each
    // branch's dot-joined path without parent links in the arena.
    // The stacked segments stay bare; the annotation joins only
    // the branch's heading.
    let mut paths: Vec<String> = Vec::new();
    for group in grid.groups() {
        if let Some(branch) = group.branch {
            let node = grid.node(branch);
            let leaf = node.label.render();
            paths.truncate(group.depth);
            let path = match paths.last() {
                Some(parent) => format!("{parent}.{leaf}"),
                None => leaf.clone(),
            };
            let text = match label {
                TreeGridTableLabelMode::Concat => path.clone(),
                TreeGridTableLabelMode::Header => leaf,
            };
            let text = match &node.annotation {
                Some(annotation) => format!("{text} {annotation}"),
                None => text,
            };
            blocks.push(frame.heading(group.depth, &text));
            paths.push(path);
        }
        if !group.members.is_empty() {
            let labels: Vec<String> = group
                .members
                .iter()
                .map(|&id| grid.node(id).annotated_label())
                .collect();
            blocks.push(table_block(grid, &labels, &group.members, frame));
        }
    }
    blocks
}

/// The flat block: one table over every data node, when any.
fn flat_blocks<C: TreeGridCells, F: TableFrame>(grid: &TreeGrid<C>, frame: &F) -> Vec<String> {
    let (labels, ids): (Vec<String>, Vec<U32Id<BTreeGridNode>>) =
        grid.data_paths().into_iter().unzip();
    if ids.is_empty() {
        return Vec::new();
    }
    vec![table_block(grid, &labels, &ids, frame)]
}

/// The records blocks: a heading-less table of the data-bearing
/// roots, then one heading and table per value-less root that
/// leads to data.
fn records_blocks<C: TreeGridCells, F: TableFrame>(grid: &TreeGrid<C>, frame: &F) -> Vec<String> {
    let mut blocks: Vec<String> = Vec::new();
    let leading: Vec<U32Id<BTreeGridNode>> = grid
        .roots()
        .iter()
        .copied()
        .filter(|&root| !grid.node(root).values.is_empty())
        .collect();
    if !leading.is_empty() {
        blocks.push(records_table(grid, &leading, frame));
    }
    for &root in grid.roots() {
        let node = grid.node(root);
        if !node.values.is_empty() || !grid.leads_to_data(root) {
            continue;
        }
        blocks.push(frame.heading(0, &node.annotated_label()));
        let rows: Vec<U32Id<BTreeGridNode>> = node
            .children()
            .iter()
            .copied()
            .filter(|&child| bears_data(grid, child))
            .collect();
        blocks.push(records_table(grid, &rows, frame));
    }
    blocks
}

/// One records table over `rows`: a `label` column, a `value`
/// column when any row bears values, then the union of the rows'
/// relative descendant data paths in first-encounter order. A row
/// without data at a column leaves the cell blank.
fn records_table<C: TreeGridCells, F: TableFrame>(
    grid: &TreeGrid<C>,
    rows: &[U32Id<BTreeGridNode>],
    frame: &F,
) -> String {
    let row_entries: Vec<Vec<(String, Vec<U32Id<BTreeGridNode>>)>> = rows
        .iter()
        .map(|&row| relative_entries(grid, row))
        .collect();
    let mut columns: Vec<String> = Vec::new();
    for entries in &row_entries {
        for (path, _) in entries {
            if !columns.contains(path) {
                columns.push(path.clone());
            }
        }
    }
    let has_value = rows.iter().any(|&row| !grid.node(row).values.is_empty());

    let mut headers = vec![Cell::text("label")];
    if has_value {
        headers.push(Cell::text("value"));
    }
    headers.extend(columns.iter().map(|path| frame.cell(Cell::text(path))));
    let table_rows: Vec<Vec<Cell>> = rows
        .iter()
        .zip(&row_entries)
        .map(|(&row, entries)| {
            let mut cells = vec![frame.cell(Cell::text(grid.node(row).annotated_label()))];
            if has_value {
                cells.push(joined_cell(grid, &[row], frame));
            }
            for path in &columns {
                let ids = match entries.iter().find(|(entry, _)| entry == path) {
                    Some((_, ids)) => ids.as_slice(),
                    None => &[],
                };
                cells.push(joined_cell(grid, ids, frame));
            }
            cells
        })
        .collect();
    frame.table(&headers, &table_rows)
}

/// The relative descendant data paths under `row` in pre-order,
/// same-path entries merged in encounter order.
fn relative_entries<C: TreeGridCells>(
    grid: &TreeGrid<C>,
    row: U32Id<BTreeGridNode>,
) -> Vec<(String, Vec<U32Id<BTreeGridNode>>)> {
    let mut paths: Vec<(String, U32Id<BTreeGridNode>)> = Vec::new();
    for &child in grid.node(row).children() {
        grid.collect_data_paths(child, "", &mut paths);
    }
    let mut entries: Vec<(String, Vec<U32Id<BTreeGridNode>>)> = Vec::new();
    for (path, id) in paths {
        match entries.iter_mut().find(|(entry, _)| *entry == path) {
            Some((_, ids)) => ids.push(id),
            None => entries.push((path, vec![id])),
        }
    }
    entries
}

/// One table cell joining every value of `ids` with the separator
/// rule; no values leave the cell blank.
fn joined_cell<C: TreeGridCells, F: TableFrame>(
    grid: &TreeGrid<C>,
    ids: &[U32Id<BTreeGridNode>],
    frame: &F,
) -> Cell {
    let mut cells: Vec<Cell> = Vec::new();
    for &id in ids {
        let node = grid.node(id);
        for value in &node.values {
            cells.push(Cell::render(grid.cells(), node.format, value));
        }
    }
    if cells.is_empty() {
        return Cell::text("");
    }
    let separator = Cell::separator(&cells);
    let width =
        cells.iter().map(|cell| cell.width).sum::<usize>() + separator.len() * (cells.len() - 1);
    let rendered = cells
        .iter()
        .map(|cell| cell.rendered.as_str())
        .collect::<Vec<&str>>()
        .join(separator);
    frame.cell(Cell {
        rendered,
        width,
        bare_visual: separator.is_empty(),
    })
}

/// Whether the node or any descendant bears values.
fn bears_data<C: TreeGridCells>(grid: &TreeGrid<C>, id: U32Id<BTreeGridNode>) -> bool {
    !grid.node(id).values.is_empty() || grid.leads_to_data(id)
}

/// One table over `ids`: a `#` index column, then one column per
/// node headed by its label, one row per value index.
fn table_block<C: TreeGridCells, F: TableFrame>(
    grid: &TreeGrid<C>,
    labels: &[String],
    ids: &[U32Id<BTreeGridNode>],
    frame: &F,
) -> String {
    let columns: Vec<Vec<Cell>> = ids
        .iter()
        .map(|&id| {
            let node = grid.node(id);
            node.values
                .iter()
                .map(|value| frame.cell(Cell::render(grid.cells(), node.format, value)))
                .collect()
        })
        .collect();
    let row_count = columns.iter().map(Vec::len).max().unwrap_or(0);

    let mut headers = vec![Cell::text("#")];
    headers.extend(labels.iter().map(|label| frame.cell(Cell::text(label))));
    let rows: Vec<Vec<Cell>> = (0..row_count)
        .map(|row| {
            let mut cells = vec![Cell::text(row.to_string())];
            cells.extend(
                columns
                    .iter()
                    .map(|column| column.get(row).cloned().unwrap_or_else(|| Cell::text(""))),
            );
            cells
        })
        .collect();
    frame.table(&headers, &rows)
}
