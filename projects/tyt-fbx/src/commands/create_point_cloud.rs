use crate::{Dependencies, Error, Result, blender};
use clap::Parser;
use std::{
    ffi::OsStr,
    io::{Error as IOError, ErrorKind},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use ty_math::TyVector3;

/// Creates a cloud of random points on a mesh surface within an FBX file.
#[derive(Clone, Debug, Parser)]
#[command(name = "create-point-cloud")]
pub struct CreatePointCloud {
    /// The input FBX file.
    #[arg(value_name = "input-fbx")]
    input_fbx: PathBuf,

    /// The name of the mesh to sample points from.
    #[arg(value_name = "mesh-name")]
    mesh_name: String,

    /// The number of random points to generate.
    #[arg(value_name = "num-points")]
    num_points: usize,
}

impl CreatePointCloud {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let CreatePointCloud {
            input_fbx,
            mesh_name,
            num_points,
        } = self;

        let args: [&OsStr; 2] = [input_fbx.as_ref(), mesh_name.as_ref()];
        let stdout =
            dependencies.exec_temp_blender_script(&blender::EXTRACT_FACES_AND_VERTICES_PY, args)?;

        let (vertices, triangles) = dependencies.parse_mesh_json(&stdout)?;
        let points = sample_points(&vertices, &triangles, num_points)?;

        let json = dependencies.serialize_points_json(&points)?;
        dependencies.write_stdout(&json)?;
        Ok(())
    }
}

fn parse_error(msg: impl Into<String>) -> Error {
    Error::IO(IOError::new(ErrorKind::InvalidData, msg.into()))
}

fn triangle_area(a: TyVector3, b: TyVector3, c: TyVector3) -> f64 {
    0.5 * (b - a).cross(&(c - a)).magnitude()
}

fn xorshift64(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

fn random_f64(state: &mut u64) -> f64 {
    (xorshift64(state) >> 11) as f64 / ((1u64 << 53) as f64)
}

fn sample_points(
    vertices: &[TyVector3],
    triangles: &[[usize; 3]],
    num_points: usize,
) -> Result<Vec<TyVector3>> {
    if triangles.is_empty() {
        return Err(parse_error("mesh has no triangles"));
    }

    let mut cumulative = Vec::with_capacity(triangles.len());
    let mut total = 0.0;
    for tri in triangles {
        let area = triangle_area(vertices[tri[0]], vertices[tri[1]], vertices[tri[2]]);
        total += area;
        cumulative.push(total);
    }

    if total <= 0.0 {
        return Err(parse_error("mesh has zero surface area"));
    }

    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1);
    let mut rng = if seed == 0 { 1 } else { seed };

    let mut points = Vec::with_capacity(num_points);
    for _ in 0..num_points {
        let target = random_f64(&mut rng) * total;
        let idx = cumulative
            .partition_point(|&a| a < target)
            .min(triangles.len() - 1);

        let [ai, bi, ci] = triangles[idx];
        let a = vertices[ai];
        let b = vertices[bi];
        let c = vertices[ci];

        let r1 = random_f64(&mut rng);
        let r2 = random_f64(&mut rng);
        let s = r1.sqrt();
        let u = 1.0 - s;
        let v = r2 * s;
        let w = 1.0 - u - v;

        points.push(a * u + b * v + c * w);
    }

    Ok(points)
}
