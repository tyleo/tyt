use crate::{Error, Result};
use gltf::{Node, buffer::Data, import_slice, mesh::Mode};

/// A 4x4 transform in column-major order (`matrix[column][row]`), matching how
/// glTF stores node matrices.
type Matrix = [[f64; 4]; 4];

/// The identity transform.
const IDENTITY: Matrix = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// Reads every triangle of a glTF or GLB byte slice into world space, with
/// glTF's Y-up axes converted to the Voxel Json Z-up convention so a voxelized
/// model stands upright. Each triangle is three `[x, y, z]` points.
///
/// Node and scene transforms are applied, so an authored scale bakes into the
/// points and two exports of one object at different scales voxelize alike.
/// Only `triangles`-mode primitives contribute; points, lines, and triangle
/// strips and fans are skipped. A `.gltf` that references external buffer files
/// cannot be resolved from bytes alone and errors.
pub(crate) fn gltf_triangles(bytes: &[u8]) -> Result<Vec<[[f64; 3]; 3]>> {
    let (document, buffers, _images) = import_slice(bytes)?;
    let scene = document
        .default_scene()
        .or_else(|| document.scenes().next())
        .ok_or_else(|| Error::invalid("glTF has no scene to voxelize"))?;

    let mut triangles = Vec::new();
    for node in scene.nodes() {
        collect_node(&node, &IDENTITY, &buffers, &mut triangles);
    }
    Ok(triangles)
}

/// Appends `node`'s triangles in world space, then recurses into its children.
/// `parent` is the accumulated world transform of `node`'s parent.
fn collect_node(node: &Node, parent: &Matrix, buffers: &[Data], out: &mut Vec<[[f64; 3]; 3]>) {
    let world = multiply(parent, &to_matrix(node.transform().matrix()));

    if let Some(mesh) = node.mesh() {
        for primitive in mesh.primitives() {
            if primitive.mode() != Mode::Triangles {
                continue;
            }
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()][..]));
            let Some(positions) = reader.read_positions() else {
                continue;
            };
            let positions: Vec<[f64; 3]> = positions
                .map(|p| world_z_up(&world, [p[0] as f64, p[1] as f64, p[2] as f64]))
                .collect();

            match reader.read_indices() {
                Some(indices) => {
                    let indices: Vec<u32> = indices.into_u32().collect();
                    for face in indices.chunks_exact(3) {
                        if let (Some(&a), Some(&b), Some(&c)) = (
                            positions.get(face[0] as usize),
                            positions.get(face[1] as usize),
                            positions.get(face[2] as usize),
                        ) {
                            out.push([a, b, c]);
                        }
                    }
                }
                None => {
                    for face in positions.chunks_exact(3) {
                        out.push([face[0], face[1], face[2]]);
                    }
                }
            }
        }
    }

    for child in node.children() {
        collect_node(&child, &world, buffers, out);
    }
}

/// Transforms `point` by `world` (glTF Y-up) and converts the result to the
/// Voxel Json Z-up axes, a +90 degree rotation about X that sends glTF +Y to
/// +Z, preserving the right-handedness both formats use.
fn world_z_up(world: &Matrix, point: [f64; 3]) -> [f64; 3] {
    let [x, y, z] = transform_point(world, point);
    [x, -z, y]
}

/// Applies a column-major transform to a point, treating it as `[x, y, z, 1]`.
fn transform_point(matrix: &Matrix, point: [f64; 3]) -> [f64; 3] {
    let [x, y, z] = point;
    [
        matrix[0][0] * x + matrix[1][0] * y + matrix[2][0] * z + matrix[3][0],
        matrix[0][1] * x + matrix[1][1] * y + matrix[2][1] * z + matrix[3][1],
        matrix[0][2] * x + matrix[1][2] * y + matrix[2][2] * z + matrix[3][2],
    ]
}

/// Widens a glTF `f32` matrix to `f64`.
fn to_matrix(matrix: [[f32; 4]; 4]) -> Matrix {
    matrix.map(|column| column.map(|value| value as f64))
}

/// The column-major product `a * b`.
fn multiply(a: &Matrix, b: &Matrix) -> Matrix {
    let mut out = [[0.0f64; 4]; 4];
    for (column, out_column) in out.iter_mut().enumerate() {
        for (row, out_cell) in out_column.iter_mut().enumerate() {
            *out_cell = (0..4).map(|k| a[k][row] * b[column][k]).sum();
        }
    }
    out
}
