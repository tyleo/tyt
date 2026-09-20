use crate::{
    Dependencies, ObjectSelection, Result, VoxelInput, cli_value_parser,
    commands::{resolve_gltf_container, select_one_object},
};
use branded_id::IdVec;
use clap::Parser;
use meshconv::{
    WriteFormat,
    gltf::{GltfContainer, GltfImageStorage, GltfWriteFormat, GltfWriteOptions},
    save,
};
use std::path::PathBuf;
use voxconv::load;
use voxcore::VoxMain;
use voxsmith::operations::mesh::{MeshRecord, Method, PrimitiveRecord, TextureShape, mesh};

/// Triangulates one object's voxels into a glTF or GLB mesh.
#[derive(Clone, Debug, Parser)]
#[command(name = "mesh")]
pub struct Mesh {
    #[command(flatten)]
    input: VoxelInput,

    /// The output mesh. Defaults to the input path with the mesh extension.
    #[arg(value_name = "output")]
    output: Option<PathBuf>,

    /// Target mesh container, glTF text (`.gltf`) or binary (`.glb`). Inferred
    /// from the output extension when omitted, defaulting to `.glb`.
    #[arg(value_name = "to", long, value_parser = cli_value_parser::<GltfContainer>())]
    to: Option<GltfContainer>,

    #[command(flatten)]
    selection: ObjectSelection,
}

impl Mesh {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let container = resolve_gltf_container(self.to, self.output.as_deref());

        let output = self
            .input
            .output_path(self.output.clone(), container.extension());

        let record = self.record();

        let from = self.input.resolve_format()?;

        let main: VoxMain = load(&dependencies, from, &self.input.path)?;

        let object = select_one_object(&main, &self.selection)?;

        let document = mesh(&main, object, &record)?;

        let format = WriteFormat::Gltf(GltfWriteFormat {
            container,
            options: GltfWriteOptions {
                images: GltfImageStorage::Embedded,
            },
        });

        Ok(save(&dependencies, &format, document, &output)?)
    }

    /// The record the run meshes under: the implicit whole-mesh primitive
    /// with no material, and every other element at its default.
    fn record(&self) -> MeshRecord {
        MeshRecord {
            method: Method::Greedy,
            texture_shape: TextureShape::Pot,
            voxel_size: 1.0,
            computed_bindings: Vec::new(),
            program: String::new(),
            materials: IdVec::default(),
            primitives: IdVec::from_vec(vec![PrimitiveRecord {
                material_id: None,
                select: "true".to_owned(),
                name: None,
                normal: true,
                uv_streams: None,
                attributes: Vec::new(),
            }]),
            files: Vec::new(),
            mesh_extras: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Mesh;
    use clap::Parser;
    use meshconv::gltf::GltfContainer;
    use voxsmith::operations::mesh::Method;

    /// The command parsed from `args` after the input.
    fn parse(args: &[&str]) -> Mesh {
        let mut argv = vec!["mesh", "model.voxj"];
        argv.extend_from_slice(args);
        Mesh::try_parse_from(argv).unwrap()
    }

    #[test]
    fn the_output_and_container_default_from_the_input() {
        let mesh = parse(&[]);
        assert_eq!(mesh.output, None);
        assert_eq!(mesh.to, None);

        assert_eq!(parse(&["--to", "gltf"]).to, Some(GltfContainer::Gltf));
        assert!(Mesh::try_parse_from(["mesh", "model.voxj", "--to", "obj"]).is_err());
    }

    #[test]
    fn the_record_holds_one_whole_mesh_primitive_with_no_material() {
        let record = parse(&[]).record();

        assert_eq!(record.method, Method::Greedy);
        assert_eq!(record.voxel_size, 1.0);
        assert!(record.materials.is_empty());
        assert_eq!(record.primitives.len(), 1);

        let [primitive] = record.primitives.as_slice() else {
            panic!("the table holds one primitive");
        };
        assert_eq!(primitive.material_id, None);
        assert_eq!(primitive.select, "true");
        assert!(primitive.normal);
    }
}
