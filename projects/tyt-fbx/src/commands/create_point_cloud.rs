use crate::{Dependencies, Error, Result, blender};
use clap::Parser;
use std::{
    ffi::OsStr,
    fmt::Write as _,
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

        let json_str = std::str::from_utf8(&stdout).map_err(|e| parse_error(e.to_string()))?;

        let (vertices, triangles) = parse_mesh_data(json_str)?;
        let points = sample_points(&vertices, &triangles, num_points)?;

        let mut output = String::from("[");
        for (i, point) in points.iter().enumerate() {
            if i > 0 {
                output.push(',');
            }
            write!(
                output,
                "{{\"X\":{},\"Y\":{},\"Z\":{}}}",
                point.x, point.y, point.z
            )
            .unwrap();
        }
        output.push_str("]\n");

        dependencies.write_stdout(output.as_bytes())?;
        Ok(())
    }
}

fn parse_error(msg: impl Into<String>) -> Error {
    Error::IO(IOError::new(ErrorKind::InvalidData, msg.into()))
}

struct JsonParser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> JsonParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            bytes: input.as_bytes(),
            pos: 0,
        }
    }

    fn skip_ws(&mut self) {
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn expect(&mut self, ch: u8) -> Result<()> {
        self.skip_ws();
        if self.peek() == Some(ch) {
            self.pos += 1;
            Ok(())
        } else {
            Err(parse_error(format!("expected '{}'", ch as char)))
        }
    }

    fn parse_string(&mut self) -> Result<&'a str> {
        self.expect(b'"')?;
        let start = self.pos;
        while self.pos < self.bytes.len() && self.bytes[self.pos] != b'"' {
            self.pos += 1;
        }
        if self.pos >= self.bytes.len() {
            return Err(parse_error("unterminated string"));
        }
        let s = std::str::from_utf8(&self.bytes[start..self.pos])
            .map_err(|e| parse_error(e.to_string()))?;
        self.pos += 1;
        Ok(s)
    }

    fn parse_f64(&mut self) -> Result<f64> {
        self.skip_ws();
        let start = self.pos;
        while self.pos < self.bytes.len()
            && matches!(
                self.bytes[self.pos],
                b'0'..=b'9' | b'.' | b'-' | b'+' | b'e' | b'E'
            )
        {
            self.pos += 1;
        }
        let s = std::str::from_utf8(&self.bytes[start..self.pos])
            .map_err(|e| parse_error(e.to_string()))?;
        s.parse::<f64>().map_err(|e| parse_error(e.to_string()))
    }

    fn parse_usize(&mut self) -> Result<usize> {
        self.skip_ws();
        let start = self.pos;
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
            self.pos += 1;
        }
        let s = std::str::from_utf8(&self.bytes[start..self.pos])
            .map_err(|e| parse_error(e.to_string()))?;
        s.parse::<usize>().map_err(|e| parse_error(e.to_string()))
    }

    fn maybe_comma(&mut self) -> bool {
        self.skip_ws();
        if self.peek() == Some(b',') {
            self.pos += 1;
            true
        } else {
            false
        }
    }
}

fn parse_vertex(p: &mut JsonParser<'_>) -> Result<TyVector3> {
    p.expect(b'{')?;
    let mut x = 0.0;
    let mut y = 0.0;
    let mut z = 0.0;
    for i in 0..3 {
        if i > 0 {
            p.expect(b',')?;
        }
        let key = p.parse_string()?;
        p.expect(b':')?;
        let val = p.parse_f64()?;
        match key {
            "X" => x = val,
            "Y" => y = val,
            "Z" => z = val,
            _ => return Err(parse_error(format!("unexpected vertex key: {key}"))),
        }
    }
    p.expect(b'}')?;
    Ok(TyVector3::new(x, y, z))
}

fn parse_triangle(p: &mut JsonParser<'_>) -> Result<[usize; 3]> {
    p.expect(b'[')?;
    let a = p.parse_usize()?;
    p.expect(b',')?;
    let b = p.parse_usize()?;
    p.expect(b',')?;
    let c = p.parse_usize()?;
    p.expect(b']')?;
    Ok([a, b, c])
}

fn parse_mesh_data(input: &str) -> Result<(Vec<TyVector3>, Vec<[usize; 3]>)> {
    let mut p = JsonParser::new(input);
    let mut vertices = Vec::new();
    let mut triangles = Vec::new();

    p.expect(b'{')?;

    for i in 0..2 {
        if i > 0 {
            p.expect(b',')?;
        }
        let key = p.parse_string()?;
        p.expect(b':')?;
        match key {
            "vertices" => {
                p.expect(b'[')?;
                loop {
                    p.skip_ws();
                    if p.peek() == Some(b']') {
                        break;
                    }
                    vertices.push(parse_vertex(&mut p)?);
                    if !p.maybe_comma() {
                        break;
                    }
                }
                p.expect(b']')?;
            }
            "triangles" => {
                p.expect(b'[')?;
                loop {
                    p.skip_ws();
                    if p.peek() == Some(b']') {
                        break;
                    }
                    triangles.push(parse_triangle(&mut p)?);
                    if !p.maybe_comma() {
                        break;
                    }
                }
                p.expect(b']')?;
            }
            _ => return Err(parse_error(format!("unexpected key: {key}"))),
        }
    }

    p.expect(b'}')?;

    Ok((vertices, triangles))
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
