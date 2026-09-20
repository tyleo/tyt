use crate::{
    Dependencies, Error, NoneOr, ObjectSelection, PositiveF64, Result, VoxelInput,
    cli_value_parser,
    commands::{
        MaterialTable, PrimitiveTable, Profile, ProfileSet, ProgramBuilder, ProgramFlag,
        ProgramFlags, apply_profile_files, apply_profile_materials, apply_profile_mesh_extras,
        apply_profile_primitives, check_expression, check_image_sources,
        declare_profile_primitives, flag_occurrences, parse_flag_index, parse_flag_value,
        parse_texture_shape, push_file_write, push_unique, push_uv_stream, resolve_gltf_container,
        select_one_object, written_file_name,
    },
};
use branded_id::U32Id;
use clap::{ArgAction, Parser};
use meshconv::{
    WriteFormat,
    gltf::{GltfContainer, GltfImageStorage, GltfWriteFormat, GltfWriteOptions},
    save,
};
use std::path::{Path, PathBuf};
use voxconv::load;
use voxcore::VoxMain;
use voxsmith::operations::mesh::{
    ArrayDomain, AttributeWrite, Computation, ComputedBinding, ExtraForm, ExtraSource, ExtraWrite,
    FileForm, FileWrite, MeshRecord, Method, PrimitiveRecord, SlotSource, SlotWrite, TextureShape,
    Transfer, WrittenValue, mesh,
};

/// Triangulates one object's voxels into a glTF or GLB mesh, baking its
/// palette materials into values that ride along as textures, material
/// fields, and files beside the mesh.
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

    /// The meshing strategy, defaulting to `greedy`. Stable per-voxel topology
    /// needs `culled` or `naive`.
    #[arg(value_name = "method", long, value_parser = cli_value_parser::<Method>())]
    method: Option<Method>,

    /// The atlas canvas, counted in cells, defaulting to `pot`. Unused cells
    /// are transparent black the mesh never samples.
    ///
    /// 1. `fit`: the near-square packing.
    /// 2. `line`: a single row of cells.
    /// 3. `pot`: the smallest square power of two.
    /// 4. `square`: the smallest square.
    /// 5. `<n>`: an exact `n`x`n` canvas of cells. A canvas too small errors.
    #[arg(
        value_name = "texture-shape",
        long,
        value_parser = parse_texture_shape,
        verbatim_doc_comment
    )]
    texture_shape: Option<TextureShape>,

    /// The real-world edge length of one voxel in meters, defaulting to `1.0`
    /// and applied as a uniform scale to every vertex position.
    #[arg(value_name = "voxel-size", long)]
    voxel_size: Option<PositiveF64>,

    /// How many materials the mesh carries, numbered from `0`. Derived from
    /// use when omitted, as the highest mentioned index plus one, and a skipped
    /// index errors. When given, an index at or above it errors and an
    /// unmentioned index below it emits an empty material.
    #[arg(value_name = "count", long)]
    material_count: Option<u32>,

    /// Names a material, as the glTF `material.name`. Repeatable.
    #[arg(
        value_names = ["material-index", "name"],
        long,
        num_args = 2,
        action = ArgAction::Append,
    )]
    material_name: Vec<String>,

    /// Declares one stream of the indexed material's list, `corner`, `face`,
    /// `swatch`, or `voxel`. The flag order sets the `TEXCOORD` numbers. A
    /// domain listed twice errors. Each of the material's textures bakes at the
    /// lowest listed domain at or above its value's. Without the flag the list
    /// derives from the textures. Repeatable.
    #[arg(
        value_names = ["material-index", "domain"],
        long,
        num_args = 2,
        action = ArgAction::Append,
    )]
    material_uv: Vec<String>,

    /// Declares a primitive drawing with the indexed material, or `none`, and
    /// taking every face its select is true for. Primitives number from `0` in
    /// flag order. The selects partition the faces. Without the flag one
    /// primitive holds every face, drawing with material 0 when the mesh
    /// carries materials. Repeatable.
    #[arg(
        value_names = ["material-index", "src-expr"],
        long,
        num_args = 2,
        action = ArgAction::Append,
    )]
    primitive: Vec<String>,

    /// Names a primitive, at `vxl.name` in its `extras`. Repeatable.
    #[arg(
        value_names = ["primitive-index", "name"],
        long,
        num_args = 2,
        action = ArgAction::Append,
    )]
    primitive_name: Vec<String>,

    /// Computes each entry's index in the domain, `corner`, `face`, `swatch`,
    /// or `voxel`, and binds it to the name as a `u32` array over the domain.
    /// Repeatable.
    #[arg(
        value_names = ["domain", "dst-name"],
        long,
        num_args = 2,
        action = ArgAction::Append,
    )]
    compute_index: Vec<String>,

    /// Computes occlusion from the voxel geometry and binds it to the name as
    /// a per-corner `f32` in `[0, 1]`, with `0` fully occluded. Repeatable.
    #[arg(value_name = "dst-name", long, action = ArgAction::Append)]
    compute_occlusion: Vec<String>,

    /// Computes each voxel's grid position and binds it to the name as a
    /// per-voxel `u32` vec3. Repeatable.
    #[arg(value_name = "dst-name", long, action = ArgAction::Append)]
    compute_voxel_position: Vec<String>,

    /// Replaces `{file-stem}` in profile file templates. Defaults to the
    /// output mesh's stem, so an output of `turret.glb` fills
    /// `{file-stem}-mse.png` as `turret-mse.png`.
    #[arg(value_name = "file-stem", long)]
    file_stem: Option<String>,

    /// Applies a profile whole, expanding it into its flags with the
    /// `valuesFrom` values first. Wherever the flag sits, the profile's values
    /// join the program ahead of every `--value` and `--values-from` binding,
    /// so a hand binding can read or redefine a profile value. An explicit
    /// flag replaces the profile element it collides with.
    #[arg(value_name = "profile", long)]
    profile: Option<String>,

    #[command(flatten)]
    program_flags: ProgramFlags,

    #[command(flatten)]
    selection: ObjectSelection,

    /// Writes a value to a JSON file beside the mesh under the name with the
    /// transfer, `linear` or `srgb`. Each file holds one object, so repeating
    /// the flag on one file merges into it. Repeatable.
    #[arg(
        value_names = ["dst-file", "dst-name", "src-expr", "transfer"],
        long,
        num_args = 4,
        action = ArgAction::Append,
    )]
    write_file_json_value: Vec<String>,

    /// Writes an array value to an 8-bit PNG beside the mesh, one texel per
    /// entry, with the transfer, `linear` or `srgb`. The value's width sets the
    /// channels: vec1 grey, vec2 grey-alpha, vec3 RGB, vec4 RGBA. Repeatable.
    #[arg(
        value_names = ["dst-file", "src-expr", "transfer"],
        long,
        num_args = 3,
        action = ArgAction::Append,
    )]
    write_file_png_value: Vec<String>,

    /// Sets the indexed material's `extras.vxl.values` entry to a texture
    /// referencing a file `--write-file-png-value` writes. Repeatable.
    #[arg(
        value_names = ["material-index", "dst-name", "src-file"],
        long,
        num_args = 3,
        action = ArgAction::Append,
    )]
    write_material_extra_image_file: Vec<String>,

    /// Embeds an array value as an image the indexed material's
    /// `extras.vxl.values` entry references, with the transfer, `linear` or
    /// `srgb`. Repeatable.
    #[arg(
        value_names = ["material-index", "dst-name", "src-expr", "transfer"],
        long,
        num_args = 4,
        action = ArgAction::Append,
    )]
    write_material_extra_image_value: Vec<String>,

    /// Sets the indexed material's `extras.vxl.values` entry to a `uri`
    /// pointer at the file by relative path. Repeatable.
    #[arg(
        value_names = ["material-index", "dst-name", "src-file"],
        long,
        num_args = 3,
        action = ArgAction::Append,
    )]
    write_material_extra_json_file: Vec<String>,

    /// Writes a value's numbers into the indexed material's `extras.vxl.values`
    /// entry with the transfer, `linear` or `srgb`. An array writes as rows.
    /// Repeatable.
    #[arg(
        value_names = ["material-index", "dst-name", "src-expr", "transfer"],
        long,
        num_args = 4,
        action = ArgAction::Append,
    )]
    write_material_extra_json_value: Vec<String>,

    /// Sets a texture property of the indexed material, named as glTF's
    /// material schema leaf, to reference a file `--write-file-png-value`
    /// writes. Repeatable.
    #[arg(
        value_names = ["material-index", "dst-property", "src-file"],
        long,
        num_args = 3,
        action = ArgAction::Append,
    )]
    write_material_slot_file: Vec<String>,

    /// Sets a property of the indexed material, named as glTF's material
    /// schema leaf. A plain value becomes a field, and an array embeds as an
    /// image. Repeatable.
    #[arg(
        value_names = ["material-index", "dst-property", "src-expr"],
        long,
        num_args = 3,
        action = ArgAction::Append,
    )]
    write_material_slot_value: Vec<String>,

    /// Sets a mesh `extras.vxl.values` entry to a texture referencing a file
    /// `--write-file-png-value` writes. Repeatable.
    #[arg(
        value_names = ["dst-name", "src-file"],
        long,
        num_args = 2,
        action = ArgAction::Append,
    )]
    write_mesh_extra_image_file: Vec<String>,

    /// Embeds an array value as an image a mesh `extras.vxl.values` entry
    /// references, with the transfer, `linear` or `srgb`. Repeatable.
    #[arg(
        value_names = ["dst-name", "src-expr", "transfer"],
        long,
        num_args = 3,
        action = ArgAction::Append,
    )]
    write_mesh_extra_image_value: Vec<String>,

    /// Sets a mesh `extras.vxl.values` entry to a `uri` pointer at the file by
    /// relative path. Repeatable.
    #[arg(
        value_names = ["dst-name", "src-file"],
        long,
        num_args = 2,
        action = ArgAction::Append,
    )]
    write_mesh_extra_json_file: Vec<String>,

    /// Writes a value's numbers into a mesh `extras.vxl.values` entry with the
    /// transfer, `linear` or `srgb`. An array writes as rows. Repeatable.
    #[arg(
        value_names = ["dst-name", "src-expr", "transfer"],
        long,
        num_args = 3,
        action = ArgAction::Append,
    )]
    write_mesh_extra_json_value: Vec<String>,

    /// Writes a value to an attribute glTF defines, `COLOR_0`, on the indexed
    /// primitive. The defined vocabulary fixes the encoding. An underscore name
    /// errors. Repeatable.
    #[arg(
        value_names = ["primitive-index", "dst-attribute", "src-expr"],
        long,
        num_args = 3,
        action = ArgAction::Append,
    )]
    write_primitive_builtin_value: Vec<String>,

    /// Writes a value to a custom vertex attribute on the indexed primitive
    /// with the transfer, `linear` or `srgb`. The name carries the leading
    /// underscore glTF requires. A bare name errors. Repeatable.
    #[arg(
        value_names = ["primitive-index", "dst-name", "src-expr", "transfer"],
        long,
        num_args = 4,
        action = ArgAction::Append,
    )]
    write_primitive_custom_value: Vec<String>,

    /// Whether the indexed primitive writes `NORMAL` beside `POSITION`, `true`
    /// by default. Repeatable.
    #[arg(
        value_names = ["primitive-index", "false | true"],
        long,
        num_args = 2,
        action = ArgAction::Append,
    )]
    write_primitive_normal: Vec<String>,

    /// Writes one UV stream, `corner`, `face`, `swatch`, or `voxel`, on the
    /// indexed primitive. The flag order sets the `TEXCOORD` numbers. A domain
    /// listed twice errors. With the flag the primitive writes exactly the
    /// named streams, and without it its material's list. Repeatable.
    #[arg(
        value_names = ["primitive-index", "domain"],
        long,
        num_args = 2,
        action = ArgAction::Append,
    )]
    write_primitive_uv: Vec<String>,
}

impl Mesh {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let (container, output) = self.resolve_output();

        let record = self.record(&output)?;

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

    /// The output container and path, from the flags or the input.
    fn resolve_output(&self) -> (GltfContainer, PathBuf) {
        let container = resolve_gltf_container(self.to, self.output.as_deref());

        let output = self
            .input
            .output_path(self.output.clone(), container.extension());

        (container, output)
    }

    /// The stem `{file-stem}` fills with: `--file-stem`, else `output`'s.
    fn file_stem(&self, output: &Path) -> String {
        match &self.file_stem {
            Some(stem) => stem.clone(),

            None => output
                .file_stem()
                .expect("the output path carries a file name")
                .to_string_lossy()
                .into_owned(),
        }
    }

    /// Whether any flag reads a profile.
    fn uses_profiles(&self) -> bool {
        self.profile.is_some() || self.program_flags.uses_profiles()
    }

    /// The record the flags and the profile lower into, checked for what they
    /// alone decide: the material and primitive counts, an element written
    /// twice, the attribute naming rules, every expression parsing, and every
    /// image reference pointing at a written PNG. A flag's element stands at
    /// its destination, and the profile fills the rest.
    fn record(&self, output: &Path) -> Result<MeshRecord> {
        let file_stem = self.file_stem(output);

        let profiles = self.uses_profiles().then(ProfileSet::built_in);

        let profile = match &self.profile {
            Some(name) => {
                let profile = profiles
                    .as_ref()
                    .expect("--profile loads the profiles")
                    .get("--profile", name)?;

                Some((format!("the profile `{name}`"), profile))
            }

            None => None,
        };

        let mut materials = match (self.material_count, &profile) {
            (Some(count), _) => MaterialTable::declared(count, format!("--material-count {count}")),

            (None, Some((origin, profile))) => {
                MaterialTable::declared(profile.materials.len() as u32, origin.clone())
            }

            (None, None) => MaterialTable::derived(),
        };

        self.lower_materials(&mut materials)?;

        let mut declared = self.lower_declared_primitives(&mut materials)?;

        // The profile's primitive list is one element, so any --primitive
        // replaces it whole.
        let profile_primitives_stand = declared.is_empty() && profile.is_some();

        if let Some((origin, profile)) = &profile {
            if profile_primitives_stand {
                declared = declare_profile_primitives(&mut materials, profile, origin)?;
            }

            apply_profile_materials(&mut materials, profile, origin, &file_stem)?;
        }

        let materials = materials.finish()?;

        let mut primitives = PrimitiveTable::new(declared, materials.len() as u32);

        self.lower_primitives(&mut primitives)?;

        if let Some((origin, profile)) = &profile
            && profile_primitives_stand
        {
            apply_profile_primitives(&mut primitives, profile, origin)?;
        }

        let mut files = self.files()?;

        let mut mesh_extras = self.mesh_extras()?;

        if let Some((origin, profile)) = &profile {
            apply_profile_files(&mut files, profile, origin, &file_stem)?;
            apply_profile_mesh_extras(&mut mesh_extras, profile, origin, &file_stem)?;
        }

        let mut builder = ProgramBuilder::new(profiles.as_ref(), self.computed_bindings()?);

        if let Some(name) = &self.profile {
            builder.land_profile("--profile", name)?;
        }

        for flag in &self.program_flags.entries {
            match flag {
                ProgramFlag::Value(text) => builder.push_value(text)?,
                ProgramFlag::ValuesFrom(name) => builder.land_profile("--values-from", name)?,
            }
        }

        let (program, computed_bindings) = builder.finish();

        let profile_voxel_size = match &profile {
            Some((origin, profile)) => profile_voxel_size(origin, profile)?,
            None => None,
        };

        let record = MeshRecord {
            method: self
                .method
                .or(profile
                    .as_ref()
                    .and_then(|(_, profile)| profile.method.map(|method| method.0)))
                .unwrap_or(Method::Greedy),
            texture_shape: self
                .texture_shape
                .or(profile
                    .as_ref()
                    .and_then(|(_, profile)| profile.texture_shape.map(|shape| shape.0)))
                .unwrap_or(TextureShape::Pot),
            voxel_size: self
                .voxel_size
                .map(|size| size.0)
                .or(profile_voxel_size)
                .unwrap_or(1.0),
            computed_bindings,
            program,
            materials,
            primitives: primitives.finish(),
            files,
            mesh_extras,
        };

        check_image_sources(&record)?;

        Ok(record)
    }

    /// Fills `materials` from every flag that takes a material index.
    fn lower_materials(&self, materials: &mut MaterialTable) -> Result<()> {
        for [index, name] in flag_occurrences::<2>(&self.material_name) {
            let flag = "--material-name";
            let index = parse_flag_index(flag, index)?;
            let material = materials.material(flag, index)?;

            if material.name.is_some() {
                return Err(Error::usage(format!("{flag} names material {index} twice")));
            }

            material.name = Some(name.clone());
        }

        for [index, domain] in flag_occurrences::<2>(&self.material_uv) {
            let flag = "--material-uv";
            let index = parse_flag_index(flag, index)?;
            let domain: ArrayDomain = parse_flag_value(flag, domain)?;
            let streams = materials
                .material(flag, index)?
                .uv_streams
                .get_or_insert_with(Vec::new);

            push_uv_stream(streams, domain, || format!("{flag} lists material {index}"))?;
        }

        for [index, property, file] in flag_occurrences::<3>(&self.write_material_slot_file) {
            let flag = "--write-material-slot-file";
            let index = parse_flag_index(flag, index)?;
            let slot = SlotWrite {
                property: property.clone(),
                source: SlotSource::File(file.clone()),
            };

            push_slot(&mut materials.material(flag, index)?.slots, slot, index)?;
        }

        for [index, property, expression] in flag_occurrences::<3>(&self.write_material_slot_value)
        {
            let flag = "--write-material-slot-value";
            let index = parse_flag_index(flag, index)?;

            check_expression(flag, expression)?;

            let slot = SlotWrite {
                property: property.clone(),
                source: SlotSource::Value(expression.clone()),
            };

            push_slot(&mut materials.material(flag, index)?.slots, slot, index)?;
        }

        for [index, name, file] in flag_occurrences::<3>(&self.write_material_extra_image_file) {
            let flag = "--write-material-extra-image-file";
            let index = parse_flag_index(flag, index)?;
            let extra = extra_file(name, ExtraForm::Image, file);

            push_material_extra(&mut materials.material(flag, index)?.extras, extra, index)?;
        }

        for [index, name, expression, transfer] in
            flag_occurrences::<4>(&self.write_material_extra_image_value)
        {
            let flag = "--write-material-extra-image-value";
            let index = parse_flag_index(flag, index)?;
            let extra = extra_value(flag, name, ExtraForm::Image, expression, transfer)?;

            push_material_extra(&mut materials.material(flag, index)?.extras, extra, index)?;
        }

        for [index, name, file] in flag_occurrences::<3>(&self.write_material_extra_json_file) {
            let flag = "--write-material-extra-json-file";
            let index = parse_flag_index(flag, index)?;
            let extra = extra_file(name, ExtraForm::Json, file);

            push_material_extra(&mut materials.material(flag, index)?.extras, extra, index)?;
        }

        for [index, name, expression, transfer] in
            flag_occurrences::<4>(&self.write_material_extra_json_value)
        {
            let flag = "--write-material-extra-json-value";
            let index = parse_flag_index(flag, index)?;
            let extra = extra_value(flag, name, ExtraForm::Json, expression, transfer)?;

            push_material_extra(&mut materials.material(flag, index)?.extras, extra, index)?;
        }

        Ok(())
    }

    /// The `--primitive` declarations in flag order, each drawing with a
    /// material it mentions in `materials` or with none.
    fn lower_declared_primitives(
        &self,
        materials: &mut MaterialTable,
    ) -> Result<Vec<PrimitiveRecord>> {
        flag_occurrences::<2>(&self.primitive)
            .map(|[material, select]| {
                let flag = "--primitive";
                let material_id = match material.parse::<NoneOr<u32>>() {
                    Ok(NoneOr::None) => None,

                    Ok(NoneOr::Value(index)) => {
                        materials.material(flag, index)?;
                        Some(U32Id::from_u32(index))
                    }

                    Err(_) => {
                        return Err(Error::usage(format!(
                            "{flag} takes a material index counted from 0 or `none`, not \
                             `{material}`"
                        )));
                    }
                };

                check_expression(flag, select)?;

                Ok(PrimitiveRecord {
                    material_id,
                    select: select.clone(),
                    name: None,
                    normal: true,
                    uv_streams: None,
                    attributes: Vec::new(),
                })
            })
            .collect()
    }

    /// Fills `primitives` from every flag that takes a primitive index.
    fn lower_primitives(&self, primitives: &mut PrimitiveTable) -> Result<()> {
        for [index, name] in flag_occurrences::<2>(&self.primitive_name) {
            let flag = "--primitive-name";
            let index = parse_flag_index(flag, index)?;
            let primitive = primitives.primitive(flag, index)?;

            if primitive.name.is_some() {
                return Err(Error::usage(format!(
                    "{flag} names primitive {index} twice"
                )));
            }

            primitive.name = Some(name.clone());
        }

        for [index, attribute, expression] in
            flag_occurrences::<3>(&self.write_primitive_builtin_value)
        {
            let flag = "--write-primitive-builtin-value";
            let index = parse_flag_index(flag, index)?;

            if attribute.starts_with('_') {
                return Err(Error::usage(format!(
                    "{flag} takes an attribute glTF defines, and `{attribute}` is custom; write \
                     it with --write-primitive-custom-value"
                )));
            }

            check_expression(flag, expression)?;

            let write = AttributeWrite::Builtin {
                attribute: attribute.clone(),
                expression: expression.clone(),
            };

            push_attribute(
                &mut primitives.primitive(flag, index)?.attributes,
                write,
                index,
            )?;
        }

        for [index, name, expression, transfer] in
            flag_occurrences::<4>(&self.write_primitive_custom_value)
        {
            let flag = "--write-primitive-custom-value";
            let index = parse_flag_index(flag, index)?;

            if !name.starts_with('_') {
                return Err(Error::usage(format!(
                    "{flag} takes a custom attribute name with glTF's leading underscore, as \
                     `_{name}`"
                )));
            }

            let write = AttributeWrite::Custom {
                name: name.clone(),
                value: written_value(flag, expression, transfer)?,
            };

            push_attribute(
                &mut primitives.primitive(flag, index)?.attributes,
                write,
                index,
            )?;
        }

        for [index, normal] in flag_occurrences::<2>(&self.write_primitive_normal) {
            let flag = "--write-primitive-normal";
            let index = parse_flag_index(flag, index)?;

            let normal = match normal.as_str() {
                "false" => false,

                "true" => true,

                other => {
                    return Err(Error::usage(format!(
                        "{flag} takes `false` or `true`, not `{other}`"
                    )));
                }
            };

            primitives.set_normal(flag, index, normal)?;
        }

        for [index, domain] in flag_occurrences::<2>(&self.write_primitive_uv) {
            let flag = "--write-primitive-uv";
            let index = parse_flag_index(flag, index)?;
            let domain: ArrayDomain = parse_flag_value(flag, domain)?;
            let streams = primitives
                .primitive(flag, index)?
                .uv_streams
                .get_or_insert_with(Vec::new);

            push_uv_stream(streams, domain, || {
                format!("{flag} lists primitive {index}")
            })?;
        }

        Ok(())
    }

    /// The `--compute-*` bindings, each name computed once.
    fn computed_bindings(&self) -> Result<Vec<ComputedBinding>> {
        let mut bindings = Vec::new();

        let push = |bindings: &mut Vec<ComputedBinding>, name: &str, computation| {
            let binding = ComputedBinding {
                name: name.to_owned(),
                computation,
            };

            push_unique(
                bindings,
                binding,
                |binding| binding.name.clone(),
                |binding| format!("`{}` is computed twice", binding.name),
            )
        };

        for [domain, name] in flag_occurrences::<2>(&self.compute_index) {
            let domain: ArrayDomain = parse_flag_value("--compute-index", domain)?;

            push(&mut bindings, name, Computation::Index(domain))?;
        }

        for name in &self.compute_occlusion {
            push(&mut bindings, name, Computation::Occlusion)?;
        }

        for name in &self.compute_voxel_position {
            push(&mut bindings, name, Computation::VoxelPosition)?;
        }

        Ok(bindings)
    }

    /// The `--write-file-*` writes, each file a bare name beside the mesh.
    fn files(&self) -> Result<Vec<FileWrite>> {
        let mut files = Vec::new();

        for [file, name, expression, transfer] in flag_occurrences::<4>(&self.write_file_json_value)
        {
            let flag = "--write-file-json-value";
            let write = FileWrite {
                file: written_file_name(flag, file)?,
                value: written_value(flag, expression, transfer)?,
                form: FileForm::Json { name: name.clone() },
            };

            push_file_write(&mut files, write, flag)?;
        }

        for [file, expression, transfer] in flag_occurrences::<3>(&self.write_file_png_value) {
            let flag = "--write-file-png-value";
            let write = FileWrite {
                file: written_file_name(flag, file)?,
                value: written_value(flag, expression, transfer)?,
                form: FileForm::Png,
            };

            push_file_write(&mut files, write, flag)?;
        }

        Ok(files)
    }

    /// The `--write-mesh-extra-*` entries, each name written once.
    fn mesh_extras(&self) -> Result<Vec<ExtraWrite>> {
        let mut extras = Vec::new();

        let push = |extras: &mut Vec<ExtraWrite>, extra: ExtraWrite| {
            push_unique(
                extras,
                extra,
                |extra| extra.name.clone(),
                |extra| format!("the mesh extra `{}` is written twice", extra.name),
            )
        };

        for [name, file] in flag_occurrences::<2>(&self.write_mesh_extra_image_file) {
            push(&mut extras, extra_file(name, ExtraForm::Image, file))?;
        }

        for [name, expression, transfer] in
            flag_occurrences::<3>(&self.write_mesh_extra_image_value)
        {
            let flag = "--write-mesh-extra-image-value";

            push(
                &mut extras,
                extra_value(flag, name, ExtraForm::Image, expression, transfer)?,
            )?;
        }

        for [name, file] in flag_occurrences::<2>(&self.write_mesh_extra_json_file) {
            push(&mut extras, extra_file(name, ExtraForm::Json, file))?;
        }

        for [name, expression, transfer] in flag_occurrences::<3>(&self.write_mesh_extra_json_value)
        {
            let flag = "--write-mesh-extra-json-value";

            push(
                &mut extras,
                extra_value(flag, name, ExtraForm::Json, expression, transfer)?,
            )?;
        }

        Ok(extras)
    }
}

/// The voxel size `profile`, which `origin` applies, sets, checked positive.
fn profile_voxel_size(origin: &str, profile: &Profile) -> Result<Option<f64>> {
    let Some(size) = profile.voxel_size else {
        return Ok(None);
    };

    if size <= 0.0 || !size.is_finite() {
        return Err(Error::usage(format!(
            "{origin}'s voxelSize is {size}, and a voxel size must be positive"
        )));
    }

    Ok(Some(size))
}

/// A written value of `flag`, its expression checked and its transfer
/// parsed.
fn written_value(flag: &str, expression: &str, transfer: &str) -> Result<WrittenValue> {
    check_expression(flag, expression)?;

    Ok(WrittenValue {
        expression: expression.to_owned(),
        transfer: parse_flag_value::<Transfer>(flag, transfer)?,
    })
}

/// An extras entry referencing `file`.
fn extra_file(name: &str, form: ExtraForm, file: &str) -> ExtraWrite {
    ExtraWrite {
        name: name.to_owned(),
        form,
        source: ExtraSource::File(file.to_owned()),
    }
}

/// An extras entry of `flag` holding a written value.
fn extra_value(
    flag: &str,
    name: &str,
    form: ExtraForm,
    expression: &str,
    transfer: &str,
) -> Result<ExtraWrite> {
    Ok(ExtraWrite {
        name: name.to_owned(),
        form,
        source: ExtraSource::Value(written_value(flag, expression, transfer)?),
    })
}

/// Pushes `slot` onto material `index`'s slots. A property written twice
/// errors.
fn push_slot(slots: &mut Vec<SlotWrite>, slot: SlotWrite, index: u32) -> Result<()> {
    push_unique(
        slots,
        slot,
        |slot| slot.property.clone(),
        |slot| {
            format!(
                "material {index}'s slot `{}` is written twice",
                slot.property
            )
        },
    )
}

/// Pushes `extra` onto material `index`'s extras. A name written twice in
/// any two forms errors.
fn push_material_extra(extras: &mut Vec<ExtraWrite>, extra: ExtraWrite, index: u32) -> Result<()> {
    push_unique(
        extras,
        extra,
        |extra| extra.name.clone(),
        |extra| format!("material {index}'s extra `{}` is written twice", extra.name),
    )
}

/// Pushes `write` onto primitive `index`'s attributes. An attribute written
/// twice errors.
fn push_attribute(
    attributes: &mut Vec<AttributeWrite>,
    write: AttributeWrite,
    index: u32,
) -> Result<()> {
    push_unique(
        attributes,
        write,
        |write| write.name().to_owned(),
        |write| {
            format!(
                "primitive {index}'s attribute `{}` is written twice",
                write.name()
            )
        },
    )
}

#[cfg(test)]
mod tests {
    use super::Mesh;
    use crate::Result;
    use branded_id::U32Id;
    use clap::Parser;
    use meshconv::gltf::GltfContainer;
    use voxsmith::operations::mesh::{
        ArrayDomain, AttributeWrite, Computation, ExtraForm, ExtraSource, FileForm, MeshRecord,
        Method, SlotSource, TextureShape, Transfer, WrittenValue,
    };

    /// The command parsed from `args` after the input.
    fn parse(args: &[&str]) -> Mesh {
        let mut argv = vec!["mesh", "model.voxj"];
        argv.extend_from_slice(args);
        Mesh::try_parse_from(argv).unwrap()
    }

    /// The record `args` lower into, or the error they raise.
    fn try_record(args: &[&str]) -> Result<MeshRecord> {
        let mesh = parse(args);
        let (_, output) = mesh.resolve_output();
        mesh.record(&output)
    }

    /// The record `args` lower into.
    fn record(args: &[&str]) -> MeshRecord {
        try_record(args).unwrap()
    }

    /// Whether `args` lower into a record or error.
    fn lowers(args: &[&str]) -> bool {
        try_record(args).is_ok()
    }

    /// The error `args` raise.
    fn error_of(args: &[&str]) -> String {
        try_record(args).unwrap_err().to_string()
    }

    /// The names the program binds, in order.
    fn bound_names(record: &MeshRecord) -> Vec<&str> {
        record
            .program
            .lines()
            .map(|line| line.split(" = ").next().unwrap())
            .collect()
    }

    const DEFAULTS: [&str; 6] = [
        "baseColorFactor",
        "occlusionStrength",
        "roughnessFactor",
        "metallicFactor",
        "emissiveFactor",
        "emissiveStrength",
    ];

    #[test]
    fn the_output_and_container_default_from_the_input() {
        let mesh = parse(&[]);
        assert_eq!(mesh.output, None);
        assert_eq!(mesh.to, None);

        assert_eq!(parse(&["--to", "gltf"]).to, Some(GltfContainer::Gltf));
        assert!(Mesh::try_parse_from(["mesh", "model.voxj", "--to", "obj"]).is_err());
    }

    #[test]
    fn the_bare_record_holds_one_whole_mesh_primitive_with_no_material() {
        let record = record(&[]);

        assert_eq!(record.method, Method::Greedy);
        assert_eq!(record.texture_shape, TextureShape::Pot);
        assert_eq!(record.voxel_size, 1.0);
        assert!(record.materials.is_empty());
        assert!(record.program.is_empty());

        let [primitive] = record.primitives.as_slice() else {
            panic!("the table holds one primitive");
        };
        assert_eq!(primitive.material_id, None);
        assert_eq!(primitive.select, "true");
        assert!(primitive.normal);
    }

    #[test]
    fn the_run_flags_lower_into_the_record() {
        let record = record(&[
            "--method",
            "naive",
            "--texture-shape",
            "64",
            "--voxel-size",
            "0.5",
            "--value",
            "a = 1",
            "--value",
            "b = a",
        ]);

        assert_eq!(record.method, Method::Naive);
        assert_eq!(record.texture_shape, TextureShape::Exact(64));
        assert_eq!(record.voxel_size, 0.5);
        assert_eq!(record.program, "a = 1;\nb = a;");
    }

    #[test]
    fn materials_derive_from_use_and_the_implicit_primitive_draws_material_0() {
        let record = record(&[
            "--material-name",
            "1",
            "glass",
            "--material-uv",
            "1",
            "face",
            "--material-uv",
            "1",
            "swatch",
            "--write-material-slot-value",
            "0",
            "baseColorFactor",
            "baseColorFactor",
        ]);

        let [steel, glass] = record.materials.as_slice() else {
            panic!("two materials");
        };
        assert_eq!(steel.name, None);
        assert_eq!(
            steel.slots[0].source,
            SlotSource::Value("baseColorFactor".to_owned())
        );
        assert_eq!(glass.name.as_deref(), Some("glass"));
        assert_eq!(
            glass.uv_streams,
            Some(vec![ArrayDomain::Face, ArrayDomain::Swatch])
        );

        assert_eq!(
            record.primitives.as_slice()[0].material_id,
            Some(U32Id::from_u32(0))
        );
    }

    #[test]
    fn a_skipped_material_index_errors_unless_the_count_is_declared() {
        assert!(!lowers(&["--material-name", "1", "glass"]));
        assert!(lowers(&[
            "--material-count",
            "2",
            "--material-name",
            "1",
            "glass"
        ]));
        assert!(!lowers(&[
            "--material-count",
            "1",
            "--material-name",
            "1",
            "glass"
        ]));
    }

    #[test]
    fn declared_primitives_replace_the_implicit_one_and_mention_their_materials() {
        let record = record(&[
            "--material-name",
            "0",
            "steel",
            "--primitive",
            "1",
            "isGlass",
            "--primitive",
            "none",
            "!isGlass",
            "--primitive-name",
            "1",
            "rest",
            "--write-primitive-normal",
            "0",
            "false",
            "--write-primitive-uv",
            "0",
            "swatch",
            "--write-primitive-builtin-value",
            "1",
            "COLOR_0",
            "baseColorFactor",
            "--write-primitive-custom-value",
            "1",
            "_HEAT",
            "heat",
            "linear",
        ]);

        assert_eq!(record.materials.len(), 2);

        let [glass, rest] = record.primitives.as_slice() else {
            panic!("two primitives");
        };
        assert_eq!(glass.material_id, Some(U32Id::from_u32(1)));
        assert_eq!(glass.select, "isGlass");
        assert!(!glass.normal);
        assert_eq!(glass.uv_streams, Some(vec![ArrayDomain::Swatch]));

        assert_eq!(rest.material_id, None);
        assert_eq!(rest.name.as_deref(), Some("rest"));
        assert!(rest.normal);
        assert_eq!(rest.uv_streams, None);
        assert_eq!(
            rest.attributes,
            [
                AttributeWrite::Builtin {
                    attribute: "COLOR_0".to_owned(),
                    expression: "baseColorFactor".to_owned(),
                },
                AttributeWrite::Custom {
                    name: "_HEAT".to_owned(),
                    value: WrittenValue {
                        expression: "heat".to_owned(),
                        transfer: Transfer::Linear,
                    },
                },
            ]
        );
    }

    #[test]
    fn a_primitive_index_at_the_count_errors() {
        assert!(!lowers(&["--primitive-name", "1", "x"]));
        assert!(!lowers(&[
            "--primitive",
            "none",
            "true",
            "--write-primitive-normal",
            "1",
            "false",
        ]));
    }

    #[test]
    fn an_element_written_twice_errors() {
        assert!(!lowers(&[
            "--material-name",
            "0",
            "a",
            "--material-name",
            "0",
            "b"
        ]));
        assert!(!lowers(&[
            "--material-uv",
            "0",
            "face",
            "--material-uv",
            "0",
            "face"
        ]));
        assert!(!lowers(&[
            "--write-material-slot-value",
            "0",
            "ior",
            "1.5",
            "--write-material-slot-file",
            "0",
            "ior",
            "a.png",
        ]));
        assert!(!lowers(&[
            "--write-material-extra-json-value",
            "0",
            "wear",
            "w",
            "linear",
            "--write-material-extra-json-file",
            "0",
            "wear",
            "wear.json",
        ]));
        assert!(!lowers(&[
            "--write-mesh-extra-json-value",
            "wear",
            "w",
            "linear",
            "--write-mesh-extra-json-file",
            "wear",
            "wear.json",
        ]));
        assert!(!lowers(&[
            "--primitive-name",
            "0",
            "a",
            "--primitive-name",
            "0",
            "b"
        ]));
        assert!(!lowers(&[
            "--write-primitive-normal",
            "0",
            "true",
            "--write-primitive-normal",
            "0",
            "false",
        ]));
        assert!(!lowers(&[
            "--write-primitive-uv",
            "0",
            "face",
            "--write-primitive-uv",
            "0",
            "face",
        ]));
        assert!(!lowers(&[
            "--write-primitive-custom-value",
            "0",
            "_A",
            "a",
            "linear",
            "--write-primitive-custom-value",
            "0",
            "_A",
            "b",
            "srgb",
        ]));
        assert!(!lowers(&[
            "--compute-occlusion",
            "ao",
            "--compute-index",
            "face",
            "ao"
        ]));
    }

    #[test]
    fn the_attribute_flags_enforce_the_underscore_rule() {
        assert!(!lowers(&[
            "--write-primitive-builtin-value",
            "0",
            "_COLOR",
            "c"
        ]));
        assert!(!lowers(&[
            "--write-primitive-custom-value",
            "0",
            "HEAT",
            "h",
            "linear",
        ]));
    }

    #[test]
    fn computed_bindings_lower_in_flag_order() {
        let record = record(&[
            "--compute-index",
            "voxel",
            "voxelIndex",
            "--compute-occlusion",
            "ao",
            "--compute-voxel-position",
            "at",
        ]);

        let computations: Vec<_> = record
            .computed_bindings
            .iter()
            .map(|binding| (binding.name.as_str(), binding.computation))
            .collect();

        assert_eq!(
            computations,
            [
                ("voxelIndex", Computation::Index(ArrayDomain::Voxel)),
                ("ao", Computation::Occlusion),
                ("at", Computation::VoxelPosition),
            ]
        );
    }

    #[test]
    fn files_and_extras_lower_with_their_forms() {
        let record = record(&[
            "--write-file-png-value",
            "albedo.png",
            "baseColorFactor",
            "srgb",
            "--write-file-json-value",
            "values.json",
            "ior",
            "ior",
            "linear",
            "--write-material-slot-file",
            "0",
            "baseColorTexture",
            "albedo.png",
            "--write-material-extra-image-file",
            "0",
            "skin",
            "albedo.png",
            "--write-mesh-extra-image-value",
            "heat",
            "heat",
            "linear",
            "--write-mesh-extra-json-file",
            "wear",
            "sidecars/wear.json",
        ]);

        assert_eq!(
            record.files[0].form,
            FileForm::Json {
                name: "ior".to_owned()
            }
        );
        assert_eq!(record.files[1].form, FileForm::Png);
        assert_eq!(record.files[1].value.transfer, Transfer::Srgb);

        let material = &record.materials.as_slice()[0];
        assert_eq!(
            material.slots[0].source,
            SlotSource::File("albedo.png".to_owned())
        );
        assert_eq!(material.extras[0].form, ExtraForm::Image);

        assert_eq!(record.mesh_extras[0].form, ExtraForm::Image);
        assert!(matches!(
            record.mesh_extras[0].source,
            ExtraSource::Value(_)
        ));
        assert_eq!(
            record.mesh_extras[1].source,
            ExtraSource::File("sidecars/wear.json".to_owned())
        );
    }

    #[test]
    fn an_image_reference_points_at_a_written_png() {
        assert!(!lowers(&[
            "--write-material-slot-file",
            "0",
            "baseColorTexture",
            "albedo.png",
        ]));
        assert!(!lowers(&[
            "--write-mesh-extra-image-file",
            "skin",
            "albedo.png"
        ]));
        assert!(lowers(&[
            "--write-file-png-value",
            "albedo.png",
            "c",
            "srgb",
            "--write-mesh-extra-image-file",
            "skin",
            "albedo.png",
        ]));
    }

    #[test]
    fn a_written_file_is_a_bare_name() {
        assert!(!lowers(&[
            "--write-file-png-value",
            "maps/albedo.png",
            "c",
            "srgb"
        ]));
        assert!(!lowers(&[
            "--write-file-json-value",
            "",
            "ior",
            "ior",
            "linear"
        ]));
    }

    #[test]
    fn a_bad_token_names_its_flag() {
        let error = error_of(&["--material-uv", "x", "face"]);
        assert!(error.contains("--material-uv"), "{error}");

        let error = error_of(&["--write-file-png-value", "a.png", "c", "gamma"]);
        assert!(error.contains("--write-file-png-value"), "{error}");
    }

    #[test]
    fn a_broken_binding_or_expression_errors_at_its_flag() {
        assert!(error_of(&["--value", "a ="]).contains("--value"));
        assert!(error_of(&["--value", " "]).contains("--value"));
        assert!(error_of(&["--primitive", "none", "1 +"]).contains("--primitive"));
        assert!(
            error_of(&["--write-material-slot-value", "0", "ior", "1 +"])
                .contains("--write-material-slot-value")
        );
    }

    #[test]
    fn a_profile_expands_with_its_values_ahead_of_the_flags() {
        let record = record(&["--value", "x = orm", "--profile", "orm"]);

        let mut expected = DEFAULTS.to_vec();
        expected.extend(["orm", "x"]);
        assert_eq!(bound_names(&record), expected);

        let [material] = record.materials.as_slice() else {
            panic!("one material");
        };
        let slots: Vec<_> = material
            .slots
            .iter()
            .map(|slot| (slot.property.as_str(), &slot.source))
            .collect();
        let orm = SlotSource::Value("orm".to_owned());
        assert_eq!(
            slots,
            [
                ("metallicRoughnessTexture", &orm),
                ("occlusionTexture", &orm)
            ]
        );

        assert_eq!(
            record.primitives.as_slice()[0].material_id,
            Some(U32Id::from_u32(0))
        );
    }

    #[test]
    fn values_from_appends_at_its_position_and_leaves_the_writers_behind() {
        let record = record(&[
            "--value",
            "a = 1",
            "--values-from",
            "emissive",
            "--value",
            "b = a",
            "--values-from",
            "orm",
        ]);

        let mut expected = vec!["a"];
        expected.extend(DEFAULTS);
        expected.extend(["maxStrength", "emissive", "white", "b", "orm"]);
        assert_eq!(bound_names(&record), expected);

        assert!(record.materials.is_empty());
        assert_eq!(record.primitives.as_slice()[0].material_id, None);
    }

    #[test]
    fn pbr_imports_its_three_maps_and_writes_six_slots() {
        let record = record(&["--profile", "pbr"]);

        let mut expected = DEFAULTS.to_vec();
        expected.extend(["albedo", "orm", "maxStrength", "emissive", "white"]);
        assert_eq!(bound_names(&record), expected);

        assert_eq!(record.materials.as_slice()[0].slots.len(), 6);
    }

    #[test]
    fn a_flag_replaces_the_profile_element_at_its_destination() {
        let record = record(&[
            "--profile",
            "orm",
            "--method",
            "culled",
            "--material-name",
            "0",
            "body",
            "--write-material-slot-value",
            "0",
            "occlusionTexture",
            "ao",
        ]);

        assert_eq!(record.method, Method::Culled);

        let [material] = record.materials.as_slice() else {
            panic!("one material");
        };
        assert_eq!(material.name.as_deref(), Some("body"));

        let slots: Vec<_> = material
            .slots
            .iter()
            .map(|slot| (slot.property.as_str(), &slot.source))
            .collect();
        assert_eq!(
            slots,
            [
                ("occlusionTexture", &SlotSource::Value("ao".to_owned())),
                (
                    "metallicRoughnessTexture",
                    &SlotSource::Value("orm".to_owned())
                ),
            ]
        );
    }

    #[test]
    fn two_flags_still_collide_under_a_profile() {
        assert!(!lowers(&[
            "--profile",
            "orm",
            "--write-material-slot-value",
            "0",
            "occlusionTexture",
            "a",
            "--write-material-slot-file",
            "0",
            "occlusionTexture",
            "a.png",
        ]));
    }

    #[test]
    fn a_profile_declares_the_material_count() {
        let error = error_of(&["--profile", "orm", "--material-name", "1", "glow"]);
        assert!(
            error.contains("the profile `orm` declares material 0 alone"),
            "{error}"
        );

        assert!(!lowers(&[
            "--profile",
            "defaults",
            "--write-material-slot-value",
            "0",
            "ior",
            "1.5",
        ]));
        assert!(lowers(&[
            "--profile",
            "defaults",
            "--material-count",
            "1",
            "--write-material-slot-value",
            "0",
            "ior",
            "1.5",
        ]));

        let error = error_of(&["--profile", "orm", "--material-count", "0"]);
        assert!(
            error.contains("--material-count 0 declares no materials"),
            "{error}"
        );
    }

    #[test]
    fn the_file_stem_defaults_to_the_output_stem() {
        let mesh = parse(&[]);
        let (_, output) = mesh.resolve_output();
        assert_eq!(mesh.file_stem(&output), "model");

        let mesh = parse(&["out/turret.gltf"]);
        let (_, output) = mesh.resolve_output();
        assert_eq!(mesh.file_stem(&output), "turret");

        let mesh = parse(&["--file-stem", "lamp"]);
        let (_, output) = mesh.resolve_output();
        assert_eq!(mesh.file_stem(&output), "lamp");
    }

    #[test]
    fn an_undefined_profile_errors_at_its_flag() {
        assert!(error_of(&["--profile", "metal"]).contains("--profile"));
        assert!(error_of(&["--values-from", "metal"]).contains("--values-from"));
    }
}
