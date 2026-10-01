use crate::{
    CliValue, Dependencies, Error, NoneOr, ObjectSelection, PositiveF64, ProfileSet, Result,
    VoxelInput, cli_value_parser,
    commands::{
        ExtraEntry, MaterialTable, MeshProfile, MeshRun, PrimitiveTable, ProgramBuilder,
        ProgramFlag, ProgramFlags, SlotEntry, load_mesh_profile_set, parse_texture_shape,
    },
    flag_occurrences, parse_flag_index, parse_flag_value,
};
use branded_id::U32Id;
use clap::{ArgAction, Parser};
use meshconv::{
    WriteFormat,
    gltf::{GltfContainer, GltfImageStorage, GltfWriteFormat, GltfWriteOptions},
    save,
};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    result::Result as StdResult,
};
use vox_value_language::parse_expression;
use voxconv::load;
use voxcore::{BVoxObject, VoxExt, VoxMain};
use voxsmith::{
    dependencies::DependenciesImpl as VoxsmithDependenciesImpl,
    operations::object::{
        ArrayDomain, AttributeWrite, Computation, ComputedBinding, ExtraForm, ExtraSource,
        ExtraWrite, FileForm, FileWrite, MeshRecord, MeshTarget, Method, PrimitiveRecord,
        SlotSource, SlotWrite, TextureShape, Transfer, WrittenValue, mesh,
    },
};

/// Triangulates the selected objects' voxels into a glTF or GLB mesh, one
/// mesh object per voxel object placed by the hierarchy reaching it, baking
/// the palette materials into values that ride along as textures, material
/// fields, and files beside the mesh.
#[derive(Clone, Debug, Parser)]
#[command(name = "mesh")]
pub struct ObjectMesh {
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
    /// and applied as a uniform scale to every vertex position and hierarchy
    /// node position.
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
    /// a per-corner `f64` in `[0, 1]`, with `0` fully occluded. Repeatable.
    #[arg(value_name = "dst-name", long, action = ArgAction::Append)]
    compute_occlusion: Vec<String>,

    /// Computes each voxel's grid position and binds it to the name as a
    /// per-voxel `u32` vec3. Repeatable.
    #[arg(value_name = "dst-name", long, action = ArgAction::Append)]
    compute_voxel_position: Vec<String>,

    /// Replaces `{file-stem}` in profile file templates. Defaults to the
    /// output mesh's stem, so an output of `turret.glb` fills
    /// `{file-stem}-mse.png` as `turret-mse.png`. A run over several objects,
    /// or under `--split-files`, joins each object's name to it with a hyphen:
    /// the same output fills the `barrel` object's as `turret-barrel-mse.png`.
    #[arg(value_name = "file-stem", long)]
    file_stem: Option<String>,

    /// Applies a profile whole, expanding it into its flags with the
    /// `valuesFrom` values first. Wherever the flag sits, the profile's values
    /// join the program ahead of every `--value` and `--values-from` binding,
    /// so a hand binding can read or redefine a profile value. An explicit
    /// flag replaces the profile element it collides with. Repeatable: the
    /// profiles stack in line order, their lists merging by position. An
    /// element two of them set errors. The profiles are the built-ins under
    /// every `.vxlconfig`'s `mesh.profiles`, the user's `~/.vxlconfig` first
    /// and then each directory from the git root down to the working
    /// directory, a name reading from the last file supplying it.
    #[arg(value_name = "profile", long, action = ArgAction::Append)]
    profile: Vec<String>,

    #[command(flatten)]
    program_flags: ProgramFlags,

    #[command(flatten)]
    selection: ObjectSelection,

    /// Writes one mesh per selected object instead of one holding them all.
    /// Each is named by the output's stem, a hyphen, and the object's name
    /// under the output's extension, holding the hierarchy reaching that
    /// object alone.
    #[arg(value_name = "split-files", long = "split-files")]
    split_files: bool,

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

impl ObjectMesh {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let (container, output) = self.resolve_output();

        let profiles = self
            .uses_profiles()
            .then(|| load_mesh_profile_set(&dependencies))
            .transpose()?;

        let input = &self.input;

        let from = input.resolve_format()?;

        let main: VoxMain = load(&dependencies, from, &input.path)?;

        let object_ids = select_mesh_objects(&main, &self.selection)?;

        let runs = MeshRun::plan(
            &main,
            &output,
            &self.file_stem(&output),
            &object_ids,
            self.split_files,
        )?;

        let format = WriteFormat::Gltf(GltfWriteFormat {
            container,
            options: GltfWriteOptions {
                images: GltfImageStorage::AsLoaded,
            },
        });

        // Every document meshes before any is written, so an error writes
        // nothing.
        let documents = runs
            .into_iter()
            .map(|run| {
                let records = run
                    .targets
                    .iter()
                    .map(|(_, stem)| self.record(stem, profiles.as_ref()))
                    .collect::<Result<Vec<_>>>()?;

                let targets: Vec<MeshTarget<'_>> = run
                    .targets
                    .iter()
                    .zip(&records)
                    .map(|(&(object_id, _), record)| MeshTarget { object_id, record })
                    .collect();

                let document = mesh(&VoxsmithDependenciesImpl, &main, &targets)?;

                Ok((run.output, document))
            })
            .collect::<Result<Vec<_>>>()?;

        for (output, document) in documents {
            save(&dependencies, &format, document, &output)?;
        }

        Ok(())
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
        !self.profile.is_empty() || self.program_flags.uses_profiles()
    }

    /// The record the flags and the profile stack lower into, checked for
    /// what they alone decide: the material and primitive counts, an element
    /// written twice, the attribute naming rules, every expression parsing,
    /// and every image reference pointing at a written PNG. A flag's element
    /// stands at its destination, and the stack fills the rest. `profiles`
    /// holds the loaded set when any flag reads a profile. `file_stem` fills
    /// `{file-stem}` in the profile file templates.
    fn record(
        &self,
        file_stem: &str,
        profiles: Option<&ProfileSet<MeshProfile>>,
    ) -> Result<MeshRecord> {
        let profile = match self.profile.as_slice() {
            [] => None,

            names => {
                let stack = stack_profiles(
                    profiles.expect("--profile loads the profiles"),
                    "--profile",
                    names,
                )?;

                Some((profile_origin(names), stack))
            }
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

            apply_profile_materials(&mut materials, profile, origin, file_stem)?;
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
            apply_profile_files(&mut files, profile, origin, file_stem)?;
            apply_profile_mesh_extras(&mut mesh_extras, profile, origin, file_stem)?;
        }

        let mut builder = ProgramBuilder::new(profiles, self.computed_bindings()?);

        for name in &self.profile {
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

/// The errors' phrase for the stack of the profiles `names`.
fn profile_origin(names: &[String]) -> String {
    let quoted: Vec<_> = names.iter().map(|name| format!("`{name}`")).collect();

    match quoted.as_slice() {
        [name] => format!("the profile {name}"),

        [head @ .., last] => format!("the profile stack {} and {last}", head.join(", ")),

        [] => unreachable!("a stack holds a profile"),
    }
}

/// The voxel size `profile`, which `origin` applies, sets, checked positive.
fn profile_voxel_size(origin: &str, profile: &MeshProfile) -> Result<Option<f64>> {
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

/// The profiles `names`, which `origin` lists, stacked into one profile to
/// apply whole. Lists merge by position and the longest sets the stack's
/// count. An element two members both set errors. The stack carries no
/// program elements because each member lands its values, imports, and
/// computed bindings by name.
fn stack_profiles(
    profiles: &ProfileSet<MeshProfile>,
    origin: &str,
    names: &[String],
) -> Result<MeshProfile> {
    let mut stack = MeshProfile::default();
    let mut claims = BTreeMap::new();

    for (position, name) in (0..).zip(names) {
        if names[..position].contains(name) {
            return Err(Error::usage(format!("{origin} lists `{name}` twice")));
        }

        let member = profiles.get(origin, name)?;

        if let Some(size) = member.voxel_size {
            claim(&mut claims, name, "voxelSize".to_owned())?;
            stack.voxel_size = Some(size);
        }

        if let Some(method) = member.method {
            claim(&mut claims, name, "method".to_owned())?;
            stack.method = Some(method);
        }

        if let Some(shape) = member.texture_shape {
            claim(&mut claims, name, "textureShape".to_owned())?;
            stack.texture_shape = Some(shape);
        }

        for (template, entries) in &member.files.json {
            let target = stack.files.json.entry(template.clone()).or_default();

            for (key, entry) in entries {
                claim(
                    &mut claims,
                    name,
                    format!("files.json template `{template}`'s key `{key}`"),
                )?;
                target.insert(key.clone(), entry.clone());
            }
        }

        for (template, entry) in &member.files.png {
            claim(
                &mut claims,
                name,
                format!("files.png template `{template}`"),
            )?;
            stack.files.png.insert(template.clone(), entry.clone());
        }

        if stack.materials.len() < member.materials.len() {
            stack
                .materials
                .resize_with(member.materials.len(), Default::default);
        }

        for (index, entry) in (0..).zip(&member.materials) {
            let target = &mut stack.materials[index];
            let entry_origin = format!("materials entry {index}");

            if let Some(material_name) = &entry.name {
                claim(&mut claims, name, format!("{entry_origin}'s name"))?;
                target.name = Some(material_name.clone());
            }

            if let Some(uvs) = &entry.uvs {
                claim(&mut claims, name, format!("{entry_origin}'s uvs"))?;
                target.uvs = Some(uvs.clone());
            }

            for (property, slot) in &entry.slots {
                claim(
                    &mut claims,
                    name,
                    format!("{entry_origin}'s slot `{property}`"),
                )?;
                target.slots.insert(property.clone(), slot.clone());
            }

            for (extra_name, extra) in &entry.extras {
                claim(
                    &mut claims,
                    name,
                    format!("{entry_origin}'s extra `{extra_name}`"),
                )?;
                target.extras.insert(extra_name.clone(), extra.clone());
            }
        }

        if stack.primitives.len() < member.primitives.len() {
            stack
                .primitives
                .resize_with(member.primitives.len(), Default::default);
        }

        for (index, entry) in (0..).zip(&member.primitives) {
            let target = &mut stack.primitives[index];
            let entry_origin = format!("primitives entry {index}");

            if let Some(primitive_name) = &entry.name {
                claim(&mut claims, name, format!("{entry_origin}'s name"))?;
                target.name = Some(primitive_name.clone());
            }

            if let Some(select) = &entry.select {
                claim(&mut claims, name, format!("{entry_origin}'s select"))?;
                target.select = Some(select.clone());
            }

            if let Some(material) = entry.material {
                claim(&mut claims, name, format!("{entry_origin}'s material"))?;
                target.material = Some(material);
            }

            if let Some(normal) = entry.normal {
                claim(&mut claims, name, format!("{entry_origin}'s normal"))?;
                target.normal = Some(normal);
            }

            if let Some(uvs) = &entry.uvs {
                claim(&mut claims, name, format!("{entry_origin}'s uvs"))?;
                target.uvs = Some(uvs.clone());
            }

            for (attribute, expression) in &entry.builtins {
                claim(
                    &mut claims,
                    name,
                    format!("{entry_origin}'s builtins key `{attribute}`"),
                )?;
                target
                    .builtins
                    .insert(attribute.clone(), expression.clone());
            }

            for (custom_name, value) in &entry.customs {
                claim(
                    &mut claims,
                    name,
                    format!("{entry_origin}'s customs key `{custom_name}`"),
                )?;
                target.customs.insert(custom_name.clone(), value.clone());
            }
        }

        for (extra_name, extra) in &member.mesh_extras {
            claim(
                &mut claims,
                name,
                format!("meshExtras entry `{extra_name}`"),
            )?;
            stack.mesh_extras.insert(extra_name.clone(), extra.clone());
        }
    }

    Ok(stack)
}

/// Records that the profile `name` sets `element`, erroring when an earlier
/// member of the stack set it.
fn claim<'a>(claims: &mut BTreeMap<String, &'a str>, name: &'a str, element: String) -> Result<()> {
    if let Some(earlier) = claims.get(&element) {
        return Err(Error::usage(format!(
            "the profile `{name}` sets {element}, which the profile `{earlier}` sets already"
        )));
    }

    claims.insert(element, name);

    Ok(())
}

/// The primitive declarations `profile`, which `origin` applies, holds in
/// list order, each drawing with the material it mentions in `materials` or
/// with none. The rest of each entry applies after the flags claim theirs.
fn declare_profile_primitives(
    materials: &mut MaterialTable,
    profile: &MeshProfile,
    origin: &str,
) -> Result<Vec<PrimitiveRecord>> {
    (0..)
        .zip(&profile.primitives)
        .map(|(index, entry)| {
            let entry_origin = format!("{origin}'s primitives entry {index}");

            let material_id = match entry.material {
                Some(material) => {
                    materials.material(&entry_origin, material)?;
                    Some(U32Id::from_u32(material))
                }

                None => None,
            };

            let select = entry.select.clone().unwrap_or_else(|| "true".to_owned());

            check_expression(&format!("{entry_origin}'s select"), &select)?;

            Ok(PrimitiveRecord {
                material_id,
                select,
                name: None,
                normal: true,
                uv_streams: None,
                attributes: Vec::new(),
            })
        })
        .collect()
}

/// Fills every per-primitive element of `profile`, which `origin` applies,
/// whose destination no flag claimed. A flag's element stands at its
/// destination, so the profile's yields to it.
fn apply_profile_primitives(
    primitives: &mut PrimitiveTable,
    profile: &MeshProfile,
    origin: &str,
) -> Result<()> {
    for (index, entry) in (0..).zip(&profile.primitives) {
        let entry_origin = format!("{origin}'s primitives entry {index}");

        if let Some(normal) = entry.normal
            && !primitives.has_normal(index)
        {
            primitives.set_normal(&entry_origin, index, normal)?;
        }

        let primitive = primitives.primitive(&entry_origin, index)?;

        if primitive.name.is_none() {
            primitive.name = entry.name.clone();
        }

        if primitive.uv_streams.is_none()
            && let Some(uvs) = &entry.uvs
        {
            primitive.uv_streams = Some(parse_uv_list(&entry_origin, uvs)?);
        }

        for (attribute, expression) in &entry.builtins {
            if attribute.starts_with('_') {
                return Err(Error::usage(format!(
                    "{entry_origin}'s builtins key `{attribute}` is custom, so it goes under \
                     customs"
                )));
            }

            if primitive
                .attributes
                .iter()
                .any(|write| write.name() == attribute)
            {
                continue;
            }

            check_expression(
                &format!("{entry_origin}'s builtins key `{attribute}`"),
                expression,
            )?;

            primitive.attributes.push(AttributeWrite::Builtin {
                attribute: attribute.clone(),
                expression: expression.clone(),
            });
        }

        for (name, value) in &entry.customs {
            if !name.starts_with('_') {
                return Err(Error::usage(format!(
                    "{entry_origin}'s customs key `{name}` lacks glTF's leading underscore, as \
                     `_{name}`"
                )));
            }

            if primitive
                .attributes
                .iter()
                .any(|write| write.name() == name)
            {
                continue;
            }

            check_expression(
                &format!("{entry_origin}'s customs key `{name}`"),
                &value.value,
            )?;

            primitive.attributes.push(AttributeWrite::Custom {
                name: name.clone(),
                value: WrittenValue {
                    expression: value.value.clone(),
                    transfer: value.transfer.0,
                },
            });
        }
    }

    Ok(())
}

/// Fills every material element of `profile`, which `origin` applies, whose
/// destination no flag claimed. A flag's element stands at its destination,
/// so the profile's yields to it.
fn apply_profile_materials(
    materials: &mut MaterialTable,
    profile: &MeshProfile,
    origin: &str,
    file_stem: &str,
) -> Result<()> {
    for (index, entry) in (0..).zip(&profile.materials) {
        let entry_origin = format!("{origin}'s materials entry {index}");
        let material = materials.material(&entry_origin, index)?;

        if material.name.is_none() {
            material.name = entry.name.clone();
        }

        if material.uv_streams.is_none()
            && let Some(uvs) = &entry.uvs
        {
            material.uv_streams = Some(parse_uv_list(&entry_origin, uvs)?);
        }

        for (property, slot) in &entry.slots {
            if material
                .slots
                .iter()
                .any(|existing| &existing.property == property)
            {
                continue;
            }

            let source = match slot {
                SlotEntry::File { file } => SlotSource::File(fill_file_template(file, file_stem)),

                SlotEntry::Value { value } => {
                    check_expression(&format!("{entry_origin}'s slot `{property}`"), value)?;
                    SlotSource::Value(value.clone())
                }
            };

            material.slots.push(SlotWrite {
                property: property.clone(),
                source,
            });
        }

        for (name, extra) in &entry.extras {
            if material
                .extras
                .iter()
                .any(|existing| &existing.name == name)
            {
                continue;
            }

            material.extras.push(extra_write(
                &format!("{entry_origin}'s extra `{name}`"),
                name,
                extra,
                file_stem,
            )?);
        }
    }

    Ok(())
}

/// Fills every file write of `profile`, which `origin` applies, whose
/// destination no flag claimed, each template filled with `file_stem`. A
/// flag's write stands at its destination, so the profile's yields to it.
fn apply_profile_files(
    files: &mut Vec<FileWrite>,
    profile: &MeshProfile,
    origin: &str,
    file_stem: &str,
) -> Result<()> {
    let claimed: Vec<(String, FileForm)> = files
        .iter()
        .map(|write| (write.file.clone(), write.form.clone()))
        .collect();

    for (template, entries) in &profile.files.json {
        let template_origin = format!("{origin}'s files.json template `{template}`");
        let file = written_file_name(&template_origin, &fill_file_template(template, file_stem))?;

        for (name, entry) in entries {
            let form = FileForm::Json { name: name.clone() };

            if claimed.contains(&(file.clone(), form.clone())) {
                continue;
            }

            check_expression(&format!("{template_origin}'s key `{name}`"), &entry.value)?;

            let write = FileWrite {
                file: file.clone(),
                value: WrittenValue {
                    expression: entry.value.clone(),
                    transfer: entry.transfer.0,
                },
                form,
            };

            push_file_write(files, write, origin)?;
        }
    }

    for (template, entry) in &profile.files.png {
        let template_origin = format!("{origin}'s files.png template `{template}`");
        let file = written_file_name(&template_origin, &fill_file_template(template, file_stem))?;

        if claimed.contains(&(file.clone(), FileForm::Png)) {
            continue;
        }

        check_expression(&template_origin, &entry.value)?;

        let write = FileWrite {
            file,
            value: WrittenValue {
                expression: entry.value.clone(),
                transfer: entry.transfer.0,
            },
            form: FileForm::Png,
        };

        push_file_write(files, write, origin)?;
    }

    Ok(())
}

/// Fills every mesh extra of `profile`, which `origin` applies, whose name no
/// flag claimed. A flag's extra stands under its name, so the profile's
/// yields to it.
fn apply_profile_mesh_extras(
    extras: &mut Vec<ExtraWrite>,
    profile: &MeshProfile,
    origin: &str,
    file_stem: &str,
) -> Result<()> {
    for (name, entry) in &profile.mesh_extras {
        if extras.iter().any(|existing| &existing.name == name) {
            continue;
        }

        extras.push(extra_write(
            &format!("{origin}'s meshExtras entry `{name}`"),
            name,
            entry,
            file_stem,
        )?);
    }

    Ok(())
}

/// The extras write `entry` describes under `name`, which `origin` holds, its
/// file template filled with `file_stem`.
fn extra_write(
    origin: &str,
    name: &str,
    entry: &ExtraEntry,
    file_stem: &str,
) -> Result<ExtraWrite> {
    let (form, source) = match entry {
        ExtraEntry::ImageFile { file } => (
            ExtraForm::Image,
            ExtraSource::File(fill_file_template(file, file_stem)),
        ),

        ExtraEntry::ImageValue { transfer, value } => {
            (ExtraForm::Image, value_source(origin, value, transfer.0)?)
        }

        ExtraEntry::JsonFile { file } => (
            ExtraForm::Json,
            ExtraSource::File(fill_file_template(file, file_stem)),
        ),

        ExtraEntry::JsonValue { transfer, value } => {
            (ExtraForm::Json, value_source(origin, value, transfer.0)?)
        }
    };

    Ok(ExtraWrite {
        name: name.to_owned(),
        form,
        source,
    })
}

/// A written value source holding `value`, checked to parse.
fn value_source(origin: &str, value: &str, transfer: Transfer) -> Result<ExtraSource> {
    check_expression(origin, value)?;

    Ok(ExtraSource::Value(WrittenValue {
        expression: value.to_owned(),
        transfer,
    }))
}

/// The stream list `uvs`, which `origin` declares, each domain a known name
/// listed once.
fn parse_uv_list(origin: &str, uvs: &[String]) -> Result<Vec<ArrayDomain>> {
    let mut streams = Vec::new();

    for domain in uvs {
        let domain: ArrayDomain = parse_flag_value(&format!("{origin}'s uvs"), domain)?;

        push_uv_stream(&mut streams, domain, || format!("{origin}'s uvs lists"))?;
    }

    Ok(streams)
}

/// Pushes `domain` onto a stream list. A domain listed twice errors with a
/// message `lists` opens.
fn push_uv_stream(
    streams: &mut Vec<ArrayDomain>,
    domain: ArrayDomain,
    lists: impl FnOnce() -> String,
) -> Result<()> {
    push_unique(
        streams,
        domain,
        |domain| *domain,
        |domain| format!("{} `{}` twice", lists(), domain.name()),
    )
}

/// Pushes `item` onto `items` unless one shares its `key`, which
/// `duplicate` describes in the usage error.
fn push_unique<T, K: PartialEq>(
    items: &mut Vec<T>,
    item: T,
    key: impl Fn(&T) -> K,
    duplicate: impl FnOnce(&T) -> String,
) -> Result<()> {
    if let Some(existing) = items.iter().find(|existing| key(existing) == key(&item)) {
        return Err(Error::usage(duplicate(existing)));
    }

    items.push(item);

    Ok(())
}

/// Pushes `write`, which `origin` holds, onto `files`. A PNG written twice
/// errors, as does a JSON entry named twice in one file or a path holding
/// both forms. JSON entries under different names merge into one file.
fn push_file_write(files: &mut Vec<FileWrite>, write: FileWrite, origin: &str) -> Result<()> {
    for existing in files.iter().filter(|existing| existing.file == write.file) {
        match (&existing.form, &write.form) {
            (FileForm::Json { name: existing }, FileForm::Json { name }) if existing == name => {
                return Err(Error::usage(format!(
                    "{origin} writes `{name}` into `{}` twice",
                    write.file
                )));
            }

            (FileForm::Json { .. }, FileForm::Json { .. }) => {}

            (FileForm::Png, FileForm::Png) => {
                return Err(Error::usage(format!(
                    "{origin} writes `{}` twice",
                    write.file
                )));
            }

            (FileForm::Json { .. }, FileForm::Png) | (FileForm::Png, FileForm::Json { .. }) => {
                return Err(Error::usage(format!(
                    "`{}` is written as both a PNG and a JSON file",
                    write.file
                )));
            }
        }
    }

    files.push(write);

    Ok(())
}

/// The bare file name `origin` writes beside the mesh. A path errors.
fn written_file_name(origin: &str, file: &str) -> Result<String> {
    require_file_name(file).map_err(|reason| Error::usage(format!("{origin}: {reason}")))
}

/// Validates that `value` is a bare file name written beside the output file:
/// non-empty and free of any path separator. A path is a mistake rather than
/// something to silently strip to its file name.
fn require_file_name(value: &str) -> StdResult<String, String> {
    if value.is_empty() {
        return Err("a file name cannot be empty".to_string());
    }

    if value.contains('/') || value.contains('\\') {
        return Err(format!(
            "`{value}` is a file name written beside the output file, so it cannot contain \
             a path separator"
        ));
    }

    Ok(value.to_string())
}

/// The file name `template` spells with `{file-stem}` replaced by `file_stem`.
fn fill_file_template(template: &str, file_stem: &str) -> String {
    template.replace("{file-stem}", file_stem)
}

/// Errors unless `text`, which `origin` holds, parses as one expression.
fn check_expression(origin: &str, text: &str) -> Result<()> {
    parse_expression(text).map(drop).map_err(|error| {
        Error::usage(format!(
            "{origin} holds `{text}`, which does not parse as an expression: {error}"
        ))
    })
}

/// Errors unless every image reference in `record`, a slot or an image
/// extra sourced from a file, points at a PNG the run writes.
fn check_image_sources(record: &MeshRecord) -> Result<()> {
    let written = |file: &str| {
        record
            .files
            .iter()
            .any(|write| write.form == FileForm::Png && write.file == file)
    };

    let check = |file: &str, reference: String| -> Result<()> {
        if written(file) {
            return Ok(());
        }

        Err(Error::usage(format!(
            "{reference} references `{file}`, which nothing writes as a PNG"
        )))
    };

    let check_extras = |extras: &[ExtraWrite], owner: &str| -> Result<()> {
        extras
            .iter()
            .filter(|extra| extra.form == ExtraForm::Image)
            .try_for_each(|extra| match &extra.source {
                ExtraSource::File(file) => check(file, format!("{owner} extra `{}`", extra.name)),
                ExtraSource::Value(_) => Ok(()),
            })
    };

    for (index, material) in (0..).zip(record.materials.iter()) {
        for slot in &material.slots {
            if let SlotSource::File(file) = &slot.source {
                check(file, format!("material {index}'s slot `{}`", slot.property))?;
            }
        }

        check_extras(&material.extras, &format!("material {index}'s"))?;
    }

    check_extras(&record.mesh_extras, "the mesh")
}

/// The container a mesh writes: `to` when given, else the one `output`'s
/// extension implies, else `.glb`.
fn resolve_gltf_container(to: Option<GltfContainer>, output: Option<&Path>) -> GltfContainer {
    to.or_else(|| {
        let extension = output?.extension()?.to_str()?;
        GltfContainer::from_extension(extension)
    })
    .unwrap_or(GltfContainer::Glb)
}

/// The objects `selection` resolves to in `main`, in document order. A
/// document holding none is a usage error; `resolve` already rejects a
/// selector matching nothing.
fn select_mesh_objects<T: VoxExt>(
    main: &VoxMain<T>,
    selection: &ObjectSelection,
) -> Result<Vec<U32Id<BVoxObject>>> {
    let object_ids = selection.resolve(main)?;

    if object_ids.is_empty() {
        return Err(Error::usage("the document has no objects to mesh"));
    }

    Ok(object_ids)
}

#[cfg(test)]
mod tests {
    use crate::{
        NamedCliValue, ProfileSet, Result,
        commands::{
            ExtraEntry, MaterialTable, MeshProfile, ObjectConfig, ObjectMesh, PrimitiveTable,
            SlotEntry, built_in_profiles,
            object::object_mesh::object_mesh::{
                apply_profile_files, apply_profile_materials, apply_profile_mesh_extras,
                apply_profile_primitives, check_expression, declare_profile_primitives,
                extra_write, fill_file_template, parse_uv_list, push_file_write, push_unique,
                require_file_name, resolve_gltf_container, select_mesh_objects, stack_profiles,
            },
        },
        owned_names, profile_set_from_json, try_parse_object_selection,
    };
    use branded_id::U32Id;
    use clap::Parser;
    use meshconv::gltf::GltfContainer;
    use std::{
        collections::BTreeMap,
        path::{Path, PathBuf},
    };
    use ty_math::TyVector3U32;
    use ty_preferences::{DeserializePrefs, JsoncCodec};
    use voxcore::{VoxMain, VoxObject};
    use voxsmith::operations::object::{
        ArrayDomain, AttributeWrite, Computation, ExtraForm, ExtraSource, ExtraWrite, FileForm,
        FileWrite, MeshRecord, Method, SlotSource, SlotWrite, TextureShape, Transfer, WrittenValue,
    };

    /// The command parsed from `args` after the input.
    fn parse(args: &[&str]) -> ObjectMesh {
        let mut argv = vec!["mesh", "model.voxj"];
        argv.extend_from_slice(args);
        ObjectMesh::try_parse_from(argv).unwrap()
    }

    /// The record `args` lower into over the built-ins under `layers`, each
    /// layer a `.vxlconfig` of the cascade, or the error they raise.
    fn try_record_over(
        layers: Vec<BTreeMap<String, MeshProfile>>,
        args: &[&str],
    ) -> Result<MeshRecord> {
        let mesh = parse(args);
        let (_, output) = mesh.resolve_output();
        let profiles = mesh.uses_profiles().then(|| {
            ProfileSet::layered(
                built_in_profiles(),
                layers
                    .into_iter()
                    .enumerate()
                    .map(|(depth, layer)| (PathBuf::from(format!("/{depth}/.vxlconfig")), layer)),
            )
        });
        mesh.record(&mesh.file_stem(&output), profiles.as_ref())
    }

    /// The record `args` lower into over the built-ins, or the error they
    /// raise.
    fn try_record(args: &[&str]) -> Result<MeshRecord> {
        try_record_over(Vec::new(), args)
    }

    /// The profiles the `.vxlconfig` text supplies.
    fn config_layer(text: &str) -> BTreeMap<String, MeshProfile> {
        let config: Option<ObjectConfig> = JsoncCodec
            .deserialize_prefs(text.as_bytes(), "object")
            .unwrap();
        config
            .expect("the text holds an object section")
            .mesh
            .profiles
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
        "baseColor",
        "occlusionStrength",
        "roughness",
        "metallic",
        "emissiveColor",
        "emissiveStrength",
    ];

    #[test]
    fn the_output_and_container_default_from_the_input() {
        let mesh = parse(&[]);
        assert_eq!(mesh.output, None);
        assert_eq!(mesh.to, None);

        assert_eq!(parse(&["--to", "gltf"]).to, Some(GltfContainer::Gltf));
        assert!(ObjectMesh::try_parse_from(["mesh", "model.voxj", "--to", "obj"]).is_err());
    }

    #[test]
    fn split_files_is_a_bare_flag() {
        assert!(!parse(&[]).split_files);
        assert!(parse(&["--split-files"]).split_files);
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
            "baseColor",
        ]);

        let [steel, glass] = record.materials.as_slice() else {
            panic!("two materials");
        };
        assert_eq!(steel.name, None);
        assert_eq!(
            steel.slots[0].source,
            SlotSource::Value("baseColor".to_owned())
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
            "baseColor",
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
                    expression: "baseColor".to_owned(),
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
            "baseColor",
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
    fn profiles_stack_in_line_order_and_a_shared_element_errors() {
        let record = record(&["--profile", "albedo", "--profile", "orm"]);

        let mut expected = DEFAULTS.to_vec();
        expected.extend(["albedo", "orm"]);
        assert_eq!(bound_names(&record), expected);

        let [material] = record.materials.as_slice() else {
            panic!("one material");
        };
        let properties: Vec<_> = material
            .slots
            .iter()
            .map(|slot| slot.property.as_str())
            .collect();
        assert_eq!(
            properties,
            [
                "baseColorTexture",
                "metallicRoughnessTexture",
                "occlusionTexture"
            ]
        );

        let error = error_of(&["--profile", "pbr", "--profile", "albedo"]);
        assert!(
            error.contains(
                "the profile `albedo` sets materials entry 0's slot `baseColorTexture`, which \
                 the profile `pbr` sets already"
            ),
            "{error}"
        );

        let error = error_of(&[
            "--profile",
            "albedo",
            "--profile",
            "orm",
            "--material-name",
            "1",
            "glow",
        ]);
        assert!(
            error.contains("the profile stack `albedo` and `orm` declares material 0 alone"),
            "{error}"
        );
    }

    #[test]
    fn an_undefined_profile_errors_at_its_flag() {
        assert!(error_of(&["--profile", "metal"]).contains("--profile"));
        assert!(error_of(&["--values-from", "metal"]).contains("--values-from"));
    }

    #[test]
    fn a_config_layer_overriding_defaults_changes_the_profiles_built_on_it() {
        let layer = config_layer(
            r#"{ "object": { "mesh": { "profiles": {
                "defaults": {
                    "values": ["baseColor = swatch(default(baseColor, rgba(0, 0, 0, 1)))"],
                },
            } } } }"#,
        );

        let record = try_record_over(vec![layer], &["--profile", "albedo"]).unwrap();

        assert_eq!(bound_names(&record), ["baseColor", "albedo"]);
    }

    #[test]
    fn a_later_layer_replaces_a_profile_wholesale() {
        let outer = config_layer(
            r#"{ "object": { "mesh": { "profiles": { "orm": {
                "values": ["orm = 1"],
                "materials": [
                    { "slots": { "occlusionTexture": { "kind": "value", "value": "orm" } } },
                ],
            } } } } }"#,
        );
        let inner = config_layer(
            r#"{ "object": { "mesh": { "profiles": { "orm": { "values": ["orm = 2"] } } } } }"#,
        );

        let record = try_record_over(vec![outer, inner], &["--profile", "orm"]).unwrap();

        assert_eq!(record.program, "orm = 2;");
        assert!(record.materials.is_empty());
    }

    #[test]
    fn a_config_profile_lands_its_primitives_files_and_mesh_extras() {
        let layer = config_layer(
            r#"{
      "object": {
    "mesh": {
      "profiles": {
        "split": {
          "valuesFrom": ["defaults"],
          "values": ["albedo = baseColor", "heat = emissiveStrength"],
          "materials": [
            {
              "name": "body",
              "slots": { "baseColorTexture": { "kind": "value", "value": "albedo" } },
            },
            {
              "name": "glow",
              "slots": { "emissiveTexture": { "kind": "file", "file": "{file-stem}-heat.png" } },
            },
          ],
          "primitives": [
            { "name": "body", "select": "heat == 0", "material": 0 },
            {
              "name": "glow",
              "select": "heat > 0",
              "material": 1,
              "normal": false,
              "uvs": ["swatch"],
              "builtins": { "COLOR_0": "albedo" },
              "customs": { "_heat": { "transfer": "linear", "value": "heat" } },
            },
          ],
          "files": {
            "json": {
              "{file-stem}-palette.json": { "rows": { "transfer": "linear", "value": "albedo" } },
            },
            "png": { "{file-stem}-heat.png": { "transfer": "linear", "value": "heat" } },
          },
          "meshExtras": {
            "heat": { "kind": "image-file", "file": "{file-stem}-heat.png" },
            "meta": { "kind": "json-value", "transfer": "linear", "value": "1" },
          },
        },
      },
    },
      },
    }"#,
        );

        let record = try_record_over(vec![layer], &["--profile", "split", "turret.glb"]).unwrap();

        let mut expected = DEFAULTS.to_vec();
        expected.extend(["albedo", "heat"]);
        assert_eq!(bound_names(&record), expected);

        let [body, glow] = record.materials.as_slice() else {
            panic!("two materials");
        };
        assert_eq!(body.name.as_deref(), Some("body"));
        assert_eq!(glow.name.as_deref(), Some("glow"));
        assert_eq!(
            glow.slots[0].source,
            SlotSource::File("turret-heat.png".to_owned())
        );

        let [first, second] = record.primitives.as_slice() else {
            panic!("two primitives");
        };
        assert_eq!(first.name.as_deref(), Some("body"));
        assert_eq!(first.select, "heat == 0");
        assert_eq!(first.material_id, Some(U32Id::from_u32(0)));
        assert!(first.normal);
        assert_eq!(second.material_id, Some(U32Id::from_u32(1)));
        assert!(!second.normal);
        assert_eq!(second.uv_streams, Some(vec![ArrayDomain::Swatch]));
        assert_eq!(
            second.attributes,
            [
                AttributeWrite::Builtin {
                    attribute: "COLOR_0".to_owned(),
                    expression: "albedo".to_owned(),
                },
                AttributeWrite::Custom {
                    name: "_heat".to_owned(),
                    value: WrittenValue {
                        expression: "heat".to_owned(),
                        transfer: Transfer::Linear,
                    },
                },
            ]
        );

        let files: Vec<_> = record
            .files
            .iter()
            .map(|write| (write.file.as_str(), &write.form))
            .collect();
        assert_eq!(
            files,
            [
                (
                    "turret-palette.json",
                    &FileForm::Json {
                        name: "rows".to_owned()
                    }
                ),
                ("turret-heat.png", &FileForm::Png),
            ]
        );

        let extras: Vec<_> = record
            .mesh_extras
            .iter()
            .map(|write| (write.name.as_str(), write.form, &write.source))
            .collect();
        assert_eq!(extras.len(), 2);
        assert_eq!(
            extras[0],
            (
                "heat",
                ExtraForm::Image,
                &ExtraSource::File("turret-heat.png".to_owned())
            )
        );
        assert_eq!(extras[1].0, "meta");
        assert_eq!(extras[1].1, ExtraForm::Json);
    }

    #[test]
    fn the_writers_merge_by_position_and_the_program_stays_behind() {
        let profiles = profile_set_from_json(&[
            (
                "albedo",
                r#"{
                    "valuesFrom": ["defaults"],
                    "computeOcclusion": "ao",
                    "values": ["albedo = baseColor"],
                    "voxelSize": 0.1,
                    "materials": [
                        {
                            "name": "body",
                            "slots": { "baseColorTexture": { "kind": "value", "value": "albedo" } }
                        }
                    ],
                    "primitives": [{ "select": "solid", "material": 0 }],
                    "files": { "json": { "{file-stem}.json": { "ior": { "transfer": "linear", "value": "ior" } } } },
                    "meshExtras": { "accent": { "kind": "json-value", "transfer": "srgb", "value": "accent" } }
                }"#,
            ),
            (
                "orm",
                r#"{
                    "method": "culled",
                    "materials": [
                        { "slots": { "occlusionTexture": { "kind": "value", "value": "orm" } } },
                        { "name": "glow" }
                    ],
                    "primitives": [{ "builtins": { "COLOR_0": "albedo" } }],
                    "files": {
                        "json": { "{file-stem}.json": { "tint": { "transfer": "srgb", "value": "tint" } } },
                        "png": { "{file-stem}-orm.png": { "transfer": "linear", "value": "orm" } }
                    },
                    "meshExtras": { "heat": { "kind": "image-file", "file": "{file-stem}-heat.png" } }
                }"#,
            ),
        ]);

        let stack =
            stack_profiles(&profiles, "--profile", &owned_names(&["albedo", "orm"])).unwrap();

        assert!(stack.values_from.is_empty());
        assert!(stack.values.is_empty());
        assert!(stack.compute_occlusion.0.is_empty());
        assert_eq!(stack.voxel_size, Some(0.1));
        assert_eq!(stack.method, Some(NamedCliValue(Method::Culled)));

        let [body, glow] = stack.materials.as_slice() else {
            panic!("two materials");
        };
        assert_eq!(body.name.as_deref(), Some("body"));
        assert_eq!(
            body.slots.keys().collect::<Vec<_>>(),
            ["baseColorTexture", "occlusionTexture"]
        );
        assert_eq!(
            body.slots["occlusionTexture"],
            SlotEntry::Value {
                value: "orm".to_owned()
            }
        );
        assert_eq!(glow.name.as_deref(), Some("glow"));

        let [primitive] = stack.primitives.as_slice() else {
            panic!("one primitive");
        };
        assert_eq!(primitive.select.as_deref(), Some("solid"));
        assert_eq!(primitive.material, Some(0));
        assert_eq!(primitive.builtins["COLOR_0"], "albedo");

        assert_eq!(
            stack.files.json["{file-stem}.json"]
                .keys()
                .collect::<Vec<_>>(),
            ["ior", "tint"]
        );
        assert_eq!(stack.files.png.len(), 1);
        assert_eq!(
            stack.mesh_extras.keys().collect::<Vec<_>>(),
            ["accent", "heat"]
        );
    }

    #[test]
    fn an_element_two_members_set_errors_naming_both() {
        let profiles = profile_set_from_json(&[
            (
                "a",
                r#"{ "materials": [{ "slots": { "baseColorTexture": { "kind": "value", "value": "a" } } }] }"#,
            ),
            (
                "b",
                r#"{ "materials": [{ "slots": { "baseColorTexture": { "kind": "value", "value": "b" } } }] }"#,
            ),
            ("greedy", r#"{ "method": "greedy" }"#),
            ("culled", r#"{ "method": "culled" }"#),
        ]);

        let error = stack_profiles(&profiles, "--profile", &owned_names(&["a", "b"]))
            .unwrap_err()
            .to_string();
        assert!(
            error.contains(
                "the profile `b` sets materials entry 0's slot `baseColorTexture`, which the \
                 profile `a` sets already"
            ),
            "{error}"
        );

        let error = stack_profiles(&profiles, "--profile", &owned_names(&["greedy", "culled"]))
            .unwrap_err()
            .to_string();
        assert!(error.contains("`culled` sets method"), "{error}");
    }

    #[test]
    fn a_member_listed_twice_errors() {
        let profiles = ProfileSet::layered(built_in_profiles(), []);

        let error = stack_profiles(&profiles, "--profile", &owned_names(&["orm", "orm"]))
            .unwrap_err()
            .to_string();
        assert!(error.contains("--profile lists `orm` twice"), "{error}");
    }

    #[test]
    fn an_undefined_member_errors() {
        let profiles = ProfileSet::layered(built_in_profiles(), []);

        assert!(
            stack_profiles(&profiles, "--profile", &owned_names(&["albedo", "metal"])).is_err()
        );
    }

    #[test]
    fn each_entry_declares_its_material_and_select() {
        let profile: MeshProfile = serde_json::from_str(
            r#"{
                "materials": [{}, {}],
                "primitives": [
                    { "select": "solid", "material": 0 },
                    { "select": "glowing", "material": 1 },
                    {}
                ]
            }"#,
        )
        .unwrap();
        let mut materials = MaterialTable::declared(2, "the profile `x`".to_owned());

        let primitives =
            declare_profile_primitives(&mut materials, &profile, "the profile `x`").unwrap();

        assert_eq!(primitives.len(), 3);
        assert_eq!(primitives[0].material_id, Some(U32Id::from_u32(0)));
        assert_eq!(primitives[0].select, "solid");
        assert_eq!(primitives[1].material_id, Some(U32Id::from_u32(1)));
        assert_eq!(primitives[2].material_id, None);
        assert_eq!(primitives[2].select, "true");
    }

    #[test]
    fn a_material_outside_the_count_errors() {
        let profile: MeshProfile =
            serde_json::from_str(r#"{ "materials": [{}], "primitives": [{ "material": 1 }] }"#)
                .unwrap();
        let mut materials = MaterialTable::declared(1, "the profile `x`".to_owned());

        let error = declare_profile_primitives(&mut materials, &profile, "the profile `x`")
            .unwrap_err()
            .to_string();
        assert!(error.contains("primitives entry 0"), "{error}");
    }

    #[test]
    fn the_flags_primitives_stand_and_the_rest_fill() {
        let profile = profile(
            r#"{
                "primitives": [
                    {
                        "name": "body",
                        "normal": false,
                        "uvs": ["face"],
                        "builtins": { "COLOR_0": "albedo" },
                        "customs": { "_HEAT": { "value": "heat", "transfer": "linear" } }
                    }
                ]
            }"#,
        );
        let mut primitives = PrimitiveTable::new(Vec::new(), 0);
        primitives
            .set_normal("--write-primitive-normal", 0, true)
            .unwrap();
        primitives
            .primitive("--write-primitive-uv", 0)
            .unwrap()
            .uv_streams = Some(vec![ArrayDomain::Swatch]);
        primitives
            .primitive("--write-primitive-builtin-value", 0)
            .unwrap()
            .attributes
            .push(AttributeWrite::Builtin {
                attribute: "COLOR_0".to_owned(),
                expression: "hand".to_owned(),
            });

        apply_profile_primitives(&mut primitives, &profile, "the profile `x`").unwrap();

        let primitives = primitives.finish();
        let [primitive] = primitives.as_slice() else {
            panic!("one primitive");
        };
        assert_eq!(primitive.name.as_deref(), Some("body"));
        assert!(primitive.normal);
        assert_eq!(primitive.uv_streams, Some(vec![ArrayDomain::Swatch]));
        assert_eq!(primitive.attributes.len(), 2);
        assert_eq!(
            primitive.attributes[0],
            AttributeWrite::Builtin {
                attribute: "COLOR_0".to_owned(),
                expression: "hand".to_owned(),
            }
        );
        assert_eq!(primitive.attributes[1].name(), "_HEAT");
    }

    #[test]
    fn the_attribute_keys_enforce_the_underscore_rule() {
        let mut primitives = PrimitiveTable::new(Vec::new(), 0);

        assert!(
            apply_profile_primitives(
                &mut primitives,
                &profile(r#"{ "primitives": [{ "builtins": { "_HEAT": "heat" } }] }"#),
                "the profile `x`",
            )
            .is_err()
        );
        assert!(
            apply_profile_primitives(
                &mut primitives,
                &profile(
                    r#"{ "primitives": [{ "customs": { "HEAT": { "value": "heat", "transfer": "linear" } } }] }"#
                ),
                "the profile `x`",
            )
            .is_err()
        );
    }

    fn materials_profile() -> MeshProfile {
        serde_json::from_str(
            r#"{
                "materials": [
                    {
                        "name": "body",
                        "uvs": ["swatch", "face"],
                        "slots": {
                            "baseColorTexture": { "kind": "value", "value": "albedo" },
                            "occlusionTexture": { "kind": "file", "file": "{file-stem}-ao.png" }
                        }
                    },
                    { "name": "glow" }
                ]
            }"#,
        )
        .unwrap()
    }

    #[test]
    fn the_flags_materials_stand_and_the_rest_fill() {
        let mut materials = MaterialTable::declared(2, "the profile `x`".to_owned());
        let hand = materials.material("--material-name", 0).unwrap();
        hand.name = Some("hand".to_owned());
        hand.uv_streams = Some(vec![ArrayDomain::Voxel]);
        hand.slots.push(SlotWrite {
            property: "baseColorTexture".to_owned(),
            source: SlotSource::Value("hand".to_owned()),
        });

        apply_profile_materials(
            &mut materials,
            &materials_profile(),
            "the profile `x`",
            "lamp",
        )
        .unwrap();

        let materials = materials.finish().unwrap();
        let [body, glow] = materials.as_slice() else {
            panic!("two materials");
        };
        assert_eq!(body.name.as_deref(), Some("hand"));
        assert_eq!(body.uv_streams, Some(vec![ArrayDomain::Voxel]));
        assert_eq!(
            body.slots,
            [
                SlotWrite {
                    property: "baseColorTexture".to_owned(),
                    source: SlotSource::Value("hand".to_owned()),
                },
                SlotWrite {
                    property: "occlusionTexture".to_owned(),
                    source: SlotSource::File("lamp-ao.png".to_owned()),
                },
            ]
        );
        assert_eq!(glow.name.as_deref(), Some("glow"));
    }

    #[test]
    fn a_material_past_the_flags_count_errors() {
        let mut materials = MaterialTable::declared(1, "--material-count 1".to_owned());

        let error = apply_profile_materials(
            &mut materials,
            &materials_profile(),
            "the profile `x`",
            "lamp",
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("materials entry 1"), "{error}");
        assert!(error.contains("--material-count 1"), "{error}");
    }

    fn profile(json: &str) -> MeshProfile {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn templates_fill_and_the_flags_writes_stand() {
        let profile = profile(
            r#"{
                "files": {
                    "png": {
                        "{file-stem}-orm.png": { "transfer": "linear", "value": "orm" },
                        "{file-stem}-mse.png": { "transfer": "linear", "value": "mse" }
                    },
                    "json": {
                        "{file-stem}.json": {
                            "ior": { "transfer": "linear", "value": "ior" },
                            "tint": { "transfer": "srgb", "value": "tint" }
                        }
                    }
                }
            }"#,
        );
        let hand = FileWrite {
            file: "lamp-orm.png".to_owned(),
            value: WrittenValue {
                expression: "hand".to_owned(),
                transfer: Transfer::Srgb,
            },
            form: FileForm::Png,
        };
        let mut files = vec![hand.clone()];

        apply_profile_files(&mut files, &profile, "the profile `x`", "lamp").unwrap();

        let names: Vec<_> = files
            .iter()
            .map(|write| (write.file.as_str(), write.value.expression.as_str()))
            .collect();
        assert_eq!(
            names,
            [
                ("lamp-orm.png", "hand"),
                ("lamp.json", "ior"),
                ("lamp.json", "tint"),
                ("lamp-mse.png", "mse"),
            ]
        );
    }

    #[test]
    fn a_template_filling_to_a_path_errors() {
        let profile = profile(
            r#"{ "files": { "png": { "maps/{file-stem}.png": { "transfer": "linear", "value": "v" } } } }"#,
        );

        let error = apply_profile_files(&mut Vec::new(), &profile, "the profile `x`", "lamp")
            .unwrap_err()
            .to_string();
        assert!(error.contains("`maps/{file-stem}.png`"), "{error}");
    }

    #[test]
    fn two_templates_filling_to_one_file_error() {
        let profile = profile(
            r#"{
                "files": {
                    "png": {
                        "{file-stem}.png": { "transfer": "linear", "value": "a" },
                        "lamp.png": { "transfer": "linear", "value": "b" }
                    }
                }
            }"#,
        );

        assert!(apply_profile_files(&mut Vec::new(), &profile, "the profile `x`", "lamp").is_err());
    }

    #[test]
    fn the_flags_extra_stands_and_the_rest_fills() {
        let profile: MeshProfile = serde_json::from_str(
            r#"{
                "meshExtras": {
                    "albedo": { "kind": "json-value", "value": "albedo", "transfer": "linear" },
                    "heat": { "kind": "image-file", "file": "{file-stem}-heat.png" }
                }
            }"#,
        )
        .unwrap();
        let hand = ExtraWrite {
            name: "albedo".to_owned(),
            form: ExtraForm::Json,
            source: ExtraSource::File("albedo.json".to_owned()),
        };
        let mut extras = vec![hand.clone()];

        apply_profile_mesh_extras(&mut extras, &profile, "the profile `x`", "lamp").unwrap();

        assert_eq!(extras[0], hand);
        assert_eq!(extras[1].name, "heat");
        assert_eq!(
            extras[1].source,
            ExtraSource::File("lamp-heat.png".to_owned())
        );
    }

    #[test]
    fn each_kind_lowers_to_its_form_and_source() {
        let write = extra_write(
            "the profile `x`",
            "heat",
            &serde_json::from_str::<ExtraEntry>(
                r#"{ "kind": "image-file", "file": "{file-stem}-heat.png" }"#,
            )
            .unwrap(),
            "lamp",
        )
        .unwrap();
        assert_eq!(write.form, ExtraForm::Image);
        assert_eq!(write.source, ExtraSource::File("lamp-heat.png".to_owned()));

        let write = extra_write(
            "the profile `x`",
            "accent",
            &serde_json::from_str::<ExtraEntry>(
                r#"{ "kind": "json-value", "value": "avg(baseColor.rgb)", "transfer": "srgb" }"#,
            )
            .unwrap(),
            "lamp",
        )
        .unwrap();
        assert_eq!(write.form, ExtraForm::Json);
        let ExtraSource::Value(value) = write.source else {
            panic!("a value source");
        };
        assert_eq!(value.transfer, Transfer::Srgb);

        assert!(
            extra_write(
                "the profile `x`",
                "accent",
                &serde_json::from_str::<ExtraEntry>(
                    r#"{ "kind": "json-value", "value": "1 +", "transfer": "srgb" }"#,
                )
                .unwrap(),
                "lamp",
            )
            .is_err()
        );
    }

    #[test]
    fn a_list_parses_in_order_and_repeats_no_domain() {
        assert_eq!(
            parse_uv_list("the profile `x`", &["swatch".to_owned(), "face".to_owned()]).unwrap(),
            [ArrayDomain::Swatch, ArrayDomain::Face]
        );
        assert!(parse_uv_list("the profile `x`", &["face".to_owned(), "face".to_owned()]).is_err());
        assert!(parse_uv_list("the profile `x`", &["edge".to_owned()]).is_err());
    }

    #[test]
    fn a_new_key_pushes_and_a_seen_key_errors() {
        let mut items = vec![("a", 1)];

        assert!(push_unique(&mut items, ("b", 2), |item| item.0, |_| String::new()).is_ok());
        assert!(push_unique(&mut items, ("a", 3), |item| item.0, |_| String::new()).is_err());
        assert_eq!(items, [("a", 1), ("b", 2)]);
    }

    /// A write of `x` to `file` in `form`.
    fn write(file: &str, form: FileForm) -> FileWrite {
        FileWrite {
            file: file.to_owned(),
            value: WrittenValue {
                expression: "x".to_owned(),
                transfer: Transfer::Linear,
            },
            form,
        }
    }

    /// A JSON entry named `name`.
    fn json(name: &str) -> FileForm {
        FileForm::Json {
            name: name.to_owned(),
        }
    }

    #[test]
    fn json_entries_merge_by_name_and_pngs_never_repeat() {
        let mut files = Vec::new();
        let origin = "--write-file-json-value";

        assert!(push_file_write(&mut files, write("v.json", json("a")), origin).is_ok());
        assert!(push_file_write(&mut files, write("v.json", json("b")), origin).is_ok());
        assert!(push_file_write(&mut files, write("v.json", json("a")), origin).is_err());

        let origin = "--write-file-png-value";

        assert!(push_file_write(&mut files, write("m.png", FileForm::Png), origin).is_ok());
        assert!(push_file_write(&mut files, write("m.png", FileForm::Png), origin).is_err());
        assert!(push_file_write(&mut files, write("m.png", json("a")), origin).is_err());
        assert!(push_file_write(&mut files, write("v.json", FileForm::Png), origin).is_err());

        assert_eq!(files.len(), 3);
    }

    #[test]
    fn accepts_a_bare_file_name() {
        assert_eq!(require_file_name("skin.png").unwrap(), "skin.png");
    }

    #[test]
    fn rejects_an_empty_name() {
        assert!(require_file_name("").is_err());
    }

    #[test]
    fn rejects_a_name_with_a_path_separator() {
        assert!(require_file_name("textures/skin.png").is_err());
        assert!(require_file_name("textures\\skin.png").is_err());
        assert!(require_file_name("../skin.png").is_err());
    }

    #[test]
    fn the_placeholder_fills_and_a_literal_stays() {
        assert_eq!(
            fill_file_template("{file-stem}-mse.png", "turret"),
            "turret-mse.png"
        );
        assert_eq!(
            fill_file_template("metallic-smoothness.png", "turret"),
            "metallic-smoothness.png"
        );
    }

    #[test]
    fn a_broken_expression_errors_at_its_origin() {
        assert!(check_expression("--primitive", "emissiveStrength > 0").is_ok());

        let error = check_expression("--primitive", "1 +")
            .unwrap_err()
            .to_string();
        assert!(error.contains("--primitive"), "{error}");
        assert!(error.contains("`1 +`"), "{error}");
    }

    #[test]
    fn to_beats_the_output_extension_beats_glb() {
        assert_eq!(
            resolve_gltf_container(Some(GltfContainer::Gltf), Some(Path::new("out.glb"))),
            GltfContainer::Gltf
        );
        assert_eq!(
            resolve_gltf_container(None, Some(Path::new("out.gltf"))),
            GltfContainer::Gltf
        );
        assert_eq!(
            resolve_gltf_container(None, Some(Path::new("out.mesh"))),
            GltfContainer::Glb
        );
        assert_eq!(resolve_gltf_container(None, None), GltfContainer::Glb);
    }

    #[test]
    fn every_selected_object_is_meshed_and_an_empty_document_errors() {
        let mut main: VoxMain = VoxMain::default();
        let a = main
            .retain_object(VoxObject::new("a".to_owned(), TyVector3U32::ONE).unwrap())
            .unwrap();
        let b = main
            .retain_object(VoxObject::new("b".to_owned(), TyVector3U32::ONE).unwrap())
            .unwrap();

        assert_eq!(
            select_mesh_objects(&main, &try_parse_object_selection(&[]).unwrap()).unwrap(),
            vec![a, b]
        );
        assert_eq!(
            select_mesh_objects(
                &main,
                &try_parse_object_selection(&["--select-index", "1"]).unwrap()
            )
            .unwrap(),
            vec![b]
        );

        let empty: VoxMain = VoxMain::default();
        assert!(select_mesh_objects(&empty, &try_parse_object_selection(&[]).unwrap()).is_err());
    }
}
