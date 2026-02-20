use crate::{Dependencies, Error, Result, blender};
use clap::Parser;
use std::{
    ffi::OsStr,
    io::{Error as IOError, ErrorKind},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use ty_math::{TyRgbaColor, TyVector3};

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

    /// Uniform scale factor applied to every output point position (e.g. 0.01 to convert centimeters to meters).
    #[arg(value_name = "scale", long)]
    scale: Option<f64>,

    /// Paths to texture images. Each texture produces a color layer by sampling at the surface UV of each point.
    #[arg(value_name = "texture", long)]
    texture: Vec<PathBuf>,
}

struct SampledPoint {
    position: TyVector3,
    triangle_index: usize,
    bary_u: f64,
    bary_v: f64,
    bary_w: f64,
}

impl CreatePointCloud {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let CreatePointCloud {
            input_fbx,
            mesh_name,
            num_points,
            surface,
            max_iterations,
            scale,
            texture,
        } = self;

        let args: [&OsStr; 2] = [input_fbx.as_ref(), mesh_name.as_ref()];
        let stdout =
            dependencies.exec_temp_blender_script(&blender::EXTRACT_FACES_AND_VERTICES_PY, args)?;

        let json_start = stdout
            .iter()
            .position(|&b| b == b'{')
            .ok_or_else(|| parse_error("no '{' found in Blender output"))?;
        let json_end = stdout
            .iter()
            .rposition(|&b| b == b'}')
            .ok_or_else(|| parse_error("no '}' found in Blender output"))?;
        let json = &stdout[json_start..=json_end];

        let (vertices, triangles, uvs) = dependencies.parse_mesh_with_uvs_json(json)?;

        let sampled = if surface {
            sample_surface_points_with_barycentrics(&vertices, &triangles, num_points)?
        } else {
            let max_iter = max_iterations.unwrap_or(num_points * 1000);
            sample_volume_points_with_barycentrics(&vertices, &triangles, num_points, max_iter)?
        };

        let points: Vec<TyVector3> = if let Some(s) = scale {
            sampled.iter().map(|sp| sp.position * s).collect()
        } else {
            sampled.iter().map(|sp| sp.position).collect()
        };

        let color_layers: Vec<Vec<TyRgbaColor>> = texture
            .iter()
            .map(|texture_path| {
                let (pixels, img_w, img_h) = dependencies.load_image_rgba(texture_path)?;
                Ok(sampled
                    .iter()
                    .map(|s| sample_texture(&uvs, &pixels, img_w, img_h, s))
                    .collect())
            })
            .collect::<Result<_>>()?;

        let out = dependencies.serialize_points_and_colors_json(&points, &color_layers)?;
        dependencies.write_stdout(&out)?;

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

fn make_rng() -> u64 {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1);
    if seed == 0 { 1 } else { seed }
}

fn random_barycentric(rng: &mut u64) -> (f64, f64, f64) {
    let r1 = random_f64(rng);
    let r2 = random_f64(rng);
    let s = r1.sqrt();
    let u = 1.0 - s;
    let v = r2 * s;
    let w = 1.0 - u - v;
    (u, v, w)
}

fn sample_surface_points_with_barycentrics(
    vertices: &[TyVector3],
    triangles: &[[usize; 3]],
    num_points: usize,
) -> Result<Vec<SampledPoint>> {
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

    let mut rng = make_rng();

    let mut sampled = Vec::with_capacity(num_points);
    for _ in 0..num_points {
        let target = random_f64(&mut rng) * total;
        let idx = cumulative
            .partition_point(|&a| a < target)
            .min(triangles.len() - 1);

        let [ai, bi, ci] = triangles[idx];
        let (a, b, c) = (vertices[ai], vertices[bi], vertices[ci]);
        let (u, v, w) = random_barycentric(&mut rng);

        sampled.push(SampledPoint {
            position: a * u + b * v + c * w,
            triangle_index: idx,
            bary_u: u,
            bary_v: v,
            bary_w: w,
        });
    }

    Ok(sampled)
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

fn sample_volume_points_with_barycentrics(
    vertices: &[TyVector3],
    triangles: &[[usize; 3]],
    num_points: usize,
    max_iterations: usize,
) -> Result<Vec<SampledPoint>> {
    if triangles.is_empty() {
        return Err(parse_error("mesh has no triangles"));
    }
    if vertices.is_empty() {
        return Err(parse_error("mesh has no vertices"));
    }

    let (min, max) = compute_aabb(vertices);
    let mut rng = make_rng();

    let mut sampled = Vec::with_capacity(num_points);
    let mut iterations = 0usize;
    while sampled.len() < num_points && iterations < max_iterations {
        iterations += 1;
        let candidate = TyVector3::new(
            min.x + random_f64(&mut rng) * (max.x - min.x),
            min.y + random_f64(&mut rng) * (max.y - min.y),
            min.z + random_f64(&mut rng) * (max.z - min.z),
        );
        if is_inside_mesh(&candidate, vertices, triangles) {
            let (closest, tri_idx, u, v, w) =
                closest_point_on_mesh(&candidate, vertices, triangles);
            let _ = closest;
            sampled.push(SampledPoint {
                position: candidate,
                triangle_index: tri_idx,
                bary_u: u,
                bary_v: v,
                bary_w: w,
            });
        }
    }

    if sampled.len() < num_points {
        return Err(parse_error(format!(
            "reached max iterations ({}) with only {}/{} points placed",
            max_iterations,
            sampled.len(),
            num_points,
        )));
    }

    Ok(sampled)
}

fn closest_point_on_mesh(
    point: &TyVector3,
    vertices: &[TyVector3],
    triangles: &[[usize; 3]],
) -> (TyVector3, usize, f64, f64, f64) {
    let mut best_dist_sq = f64::MAX;
    let mut best_pos = *point;
    let mut best_idx = 0;
    let mut best_u = 0.0;
    let mut best_v = 0.0;
    let mut best_w = 0.0;

    for (i, tri) in triangles.iter().enumerate() {
        let (closest, u, v, w) = closest_point_on_triangle(
            point,
            &vertices[tri[0]],
            &vertices[tri[1]],
            &vertices[tri[2]],
        );
        let diff = closest - *point;
        let dist_sq = diff.dot(&diff);
        if dist_sq < best_dist_sq {
            best_dist_sq = dist_sq;
            best_pos = closest;
            best_idx = i;
            best_u = u;
            best_v = v;
            best_w = w;
        }
    }

    (best_pos, best_idx, best_u, best_v, best_w)
}

fn closest_point_on_triangle(
    p: &TyVector3,
    a: &TyVector3,
    b: &TyVector3,
    c: &TyVector3,
) -> (TyVector3, f64, f64, f64) {
    let ab = *b - *a;
    let ac = *c - *a;
    let ap = *p - *a;

    let d1 = ab.dot(&ap);
    let d2 = ac.dot(&ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return (*a, 1.0, 0.0, 0.0);
    }

    let bp = *p - *b;
    let d3 = ab.dot(&bp);
    let d4 = ac.dot(&bp);
    if d3 >= 0.0 && d4 <= d3 {
        return (*b, 0.0, 1.0, 0.0);
    }

    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return (*a + ab * v, 1.0 - v, v, 0.0);
    }

    let cp = *p - *c;
    let d5 = ab.dot(&cp);
    let d6 = ac.dot(&cp);
    if d6 >= 0.0 && d5 <= d6 {
        return (*c, 0.0, 0.0, 1.0);
    }

    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return (*a + ac * w, 1.0 - w, 0.0, w);
    }

    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return (*b + (*c - *b) * w, 0.0, 1.0 - w, w);
    }

    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    let u = 1.0 - v - w;

    (*a + ab * v + ac * w, u, v, w)
}

fn sample_texture(
    uvs: &[[[f64; 2]; 3]],
    pixels: &[u8],
    img_w: u32,
    img_h: u32,
    sample: &SampledPoint,
) -> TyRgbaColor {
    let tri_uvs = &uvs[sample.triangle_index];
    let tex_u = tri_uvs[0][0] * sample.bary_u
        + tri_uvs[1][0] * sample.bary_v
        + tri_uvs[2][0] * sample.bary_w;
    let tex_v = tri_uvs[0][1] * sample.bary_u
        + tri_uvs[1][1] * sample.bary_v
        + tri_uvs[2][1] * sample.bary_w;

    let tex_u = tex_u.rem_euclid(1.0);
    let tex_v = 1.0 - tex_v.rem_euclid(1.0);

    let px = ((tex_u * img_w as f64) as u32).min(img_w - 1);
    let py = ((tex_v * img_h as f64) as u32).min(img_h - 1);

    let idx = ((py * img_w + px) * 4) as usize;
    let r = pixels[idx] as f32 / 255.0;
    let g = pixels[idx + 1] as f32 / 255.0;
    let b = pixels[idx + 2] as f32 / 255.0;
    let a = pixels[idx + 3] as f32 / 255.0;

    TyRgbaColor::new(r, g, b, a)
}
