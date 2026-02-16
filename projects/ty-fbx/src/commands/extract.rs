use clap::Parser;

#[derive(Clone, Debug, Parser)]
pub struct Extract {
    /// The input FBX file to extract from.
    #[arg(value_name = "input-fbx")]
    input_fbx: String,

    /// The name of the mesh's parent.
    #[arg(value_name = "parent-mesh-name")]
    parent_mesh_name: String,

    /// The output FBX file to write the extracted data to. If not provided,
    /// the input file will be overwritten.
    #[arg(value_name = "output-fbx")]
    output_fbx: Option<String>,

    /// The name of the output mesh to write. If not provided, the original
    /// mesh name will be used.
    #[arg(
        short = 'm',
        long = "mesh-name",
        value_name = "mesh-name",
        conflicts_with = "output_mesh_name_arg"
    )]
    output_mesh_name_flag: Option<String>,

    /// The name of the output mesh to write. If not provided, the original
    /// mesh name will be used.
    #[arg(value_name = "mesh-name", conflicts_with = "output_mesh_name_flag")]
    output_mesh_name_arg: Option<String>,
}

impl Extract {
    pub fn execute(self) {
        let Extract {
            input_fbx,
            parent_mesh_name,
            output_fbx,
            output_mesh_name_flag,
            output_mesh_name_arg,
        } = self;

        let output_mesh_name = output_mesh_name_flag
            .as_ref()
            .or(output_mesh_name_arg.as_ref())
            .unwrap_or(&parent_mesh_name);

        let output_fbx = output_fbx.as_ref().unwrap_or(&input_fbx);

        println!("input_fbx: {}", input_fbx);
        println!("parent_mesh_name: {}", parent_mesh_name);
        println!("output_fbx: {}", output_fbx);
        println!("output_mesh_name: {}", output_mesh_name);
    }
}
