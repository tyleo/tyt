use crate::{Dependencies, Error, Result, blender};
use clap::Parser;
use std::{
    ffi::OsStr,
    io::{Error as IOError, ErrorKind},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use ty_math::TyVector3;

/// Creates a cloud of random points inside a mesh volume (or on the surface with `--surface`) within an FBX file.
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

    /// Sample points on the mesh surface instead of inside the volume.
    #[arg(value_name = "surface", long)]
    surface: bool,

    /// Maximum iterations for volume rejection sampling (default: num_points * 1000).
    #[arg(value_name = "max-iterations", long)]
    max_iterations: Option<usize>,
}

impl CreatePointCloud {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let CreatePointCloud {
            input_fbx,
            mesh_name,
            num_points,
            surface,
            max_iterations,
        } = self;

        let args: [&OsStr; 2] = [input_fbx.as_ref(), mesh_name.as_ref()];
        let stdout =
            dependencies.exec_temp_blender_script(&blender::EXTRACT_FACES_AND_VERTICES_PY, args)?;

        // Blender may emit non-JSON lines to stdout (e.g. "FBX version: 7400",
        // "Blender quit"). Extract the JSON object spanning the first '{' to the
        // last '}'.
        let json_start = stdout
            .iter()
            .position(|&b| b == b'{')
            .ok_or_else(|| parse_error("no '{' found in Blender output"))?;
        let json_end = stdout
            .iter()
            .rposition(|&b| b == b'}')
            .ok_or_else(|| parse_error("no '}' found in Blender output"))?;
        let json = &stdout[json_start..=json_end];

        let (vertices, triangles) = dependencies.parse_mesh_json(json)?;
        let points = if surface {
            sample_surface_points(&vertices, &triangles, num_points)?
        } else {
            let max_iter = max_iterations.unwrap_or(num_points * 1000);
            sample_volume_points(&vertices, &triangles, num_points, max_iter)?
        };

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

fn sample_surface_points(
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

fn compute_aabb(vertices: &[TyVector3]) -> (TyVector3, TyVector3) {
    let mut min = vertices[0];
    let mut max = vertices[0];
    for v in &vertices[1..] {
        if v.x < min.x {
            min.x = v.x;
        }
        if v.y < min.y {
            min.y = v.y;
        }
        if v.z < min.z {
            min.z = v.z;
        }
        if v.x > max.x {
            max.x = v.x;
        }
        if v.y > max.y {
            max.y = v.y;
        }
        if v.z > max.z {
            max.z = v.z;
        }
    }
    (min, max)
}

fn ray_triangle_intersects(
    origin: &TyVector3,
    dir: &TyVector3,
    v0: &TyVector3,
    v1: &TyVector3,
    v2: &TyVector3,
) -> bool {
    let epsilon = 1e-10;
    let edge1 = *v1 - *v0;
    let edge2 = *v2 - *v0;
    let h = dir.cross(&edge2);
    let a = edge1.dot(&h);
    if a > -epsilon && a < epsilon {
        return false;
    }
    let f = 1.0 / a;
    let s = *origin - *v0;
    let u = f * s.dot(&h);
    if !(0.0..=1.0).contains(&u) {
        return false;
    }
    let q = s.cross(&edge1);
    let v = f * dir.dot(&q);
    if v < 0.0 || u + v > 1.0 {
        return false;
    }
    let t = f * edge2.dot(&q);
    t > epsilon
}

fn is_inside_mesh(point: &TyVector3, vertices: &[TyVector3], triangles: &[[usize; 3]]) -> bool {
    let dir = TyVector3::new(1.0, 0.0, 0.0);
    let mut count = 0u32;
    for tri in triangles {
        if ray_triangle_intersects(
            point,
            &dir,
            &vertices[tri[0]],
            &vertices[tri[1]],
            &vertices[tri[2]],
        ) {
            count += 1;
        }
    }
    count % 2 == 1
}

fn sample_volume_points(
    vertices: &[TyVector3],
    triangles: &[[usize; 3]],
    num_points: usize,
    max_iterations: usize,
) -> Result<Vec<TyVector3>> {
    if triangles.is_empty() {
        return Err(parse_error("mesh has no triangles"));
    }
    if vertices.is_empty() {
        return Err(parse_error("mesh has no vertices"));
    }

    let (min, max) = compute_aabb(vertices);

    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1);
    let mut rng = if seed == 0 { 1 } else { seed };

    let mut points = Vec::with_capacity(num_points);
    let mut iterations = 0usize;
    while points.len() < num_points && iterations < max_iterations {
        iterations += 1;
        let candidate = TyVector3::new(
            min.x + random_f64(&mut rng) * (max.x - min.x),
            min.y + random_f64(&mut rng) * (max.y - min.y),
            min.z + random_f64(&mut rng) * (max.z - min.z),
        );
        if is_inside_mesh(&candidate, vertices, triangles) {
            points.push(candidate);
        }
    }

    if points.len() < num_points {
        return Err(parse_error(format!(
            "reached max iterations ({}) with only {}/{} points placed",
            max_iterations,
            points.len(),
            num_points,
        )));
    }

    Ok(points)
}
