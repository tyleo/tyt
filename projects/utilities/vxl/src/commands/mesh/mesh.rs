use crate::{
    Dependencies, Error, Format, PositiveF64, Result, cli_value_parser,
    commands::{
        Atlas, PropertyBinding, Texture, TextureArg, TextureMap, TextureName,
        computed_occlusion_unsupported, parse_atlas_shape,
    },
    parse_index_range, require_file_name,
};
use clap::Parser;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};
use voxsmith::{
    AtlasShape, IndexRange, MaterialMap, MaterialMeshRequest, MeshFormat, MeshMethod,
    ResourceStorage,
};

/// Triangulates one object's voxels into a glTF or GLB mesh, optionally baking
/// its palette materials into textures the mesh's UVs sample.
#[derive(Clone, Debug, Parser)]
#[command(name = "mesh")]
pub struct Mesh {
    /// The input voxel file, in any supported format.
    #[arg(value_name = "input")]
    input: PathBuf,

    /// The output mesh. Defaults to the input path with the mesh extension.
    #[arg(value_name = "output")]
    output: Option<PathBuf>,

    /// Target mesh format, glTF text (`.gltf`) or binary (`.glb`). Inferred from
    /// the output extension when omitted, defaulting to `.glb`.
    #[arg(value_name = "to", long, value_parser = cli_value_parser::<MeshFormat>())]
    to: Option<MeshFormat>,

    /// Source voxel format. Inferred from the input extension when omitted.
    #[arg(value_name = "from", long)]
    from: Option<Format>,

    /// Real-world edge length of one voxel in meters, applied as a uniform scale
    /// to every output vertex. The mesh twin of `voxelize`'s `--voxel-size`.
    #[arg(value_name = "voxel-size", long, default_value = "1.0")]
    voxel_size: PositiveF64,

    /// Meshing strategy.
    #[arg(
        value_name = "method",
        long,
        default_value = "greedy",
        value_parser = cli_value_parser::<MeshMethod>()
    )]
    method: MeshMethod,

    /// Material-map atlas layout. Only `palette` is supported for now.
    #[arg(value_name = "atlas", long, default_value = "palette")]
    atlas: Atlas,

    /// Shape of the baked atlas canvas, `pot` by default. Unused cells are
    /// transparent black the mesh never samples.
    ///
    /// 1. `line`: a single row of texels, no padding.
    /// 2. `fit`: the near-square packing.
    /// 3. `square`: the smallest square.
    /// 4. `pot`: the smallest square power-of-two.
    /// 5. `n`: an exact `n`x`n` canvas, rejected when too small to fit.
    #[arg(
        value_name = "texture-shape",
        long,
        default_value = "pot",
        value_parser = parse_atlas_shape,
        verbatim_doc_comment
    )]
    texture_shape: AtlasShape,

    /// Bake a preset material map or bundle, repeatable, as `--texture albedo
    /// --texture orm` or `--texture pbr`. A bundle expands to several single
    /// maps.
    #[arg(value_name = "texture", long)]
    texture: Vec<TextureArg>,

    /// Name one preset map's file exactly, `<preset> <file-name>`, repeatable,
    /// as `--texture-name albedo skin.png`. Highest precedence, over
    /// `--texture-name-prefix` and the default. Errors if the preset is not
    /// baked or is named twice.
    #[arg(
        value_names = ["preset", "file-name"],
        long,
        num_args = 2,
        action = clap::ArgAction::Append,
    )]
    texture_name: Vec<String>,

    /// Replace the default `<output-stem>-` prefix on every preset map's file
    /// name (those without an explicit `--texture-name`); the preset follows
    /// verbatim, so the prefix carries its own separator: `--texture-name-prefix
    /// hero-` gives `hero-albedo.png` and `test.` gives `test.albedo.png`. A
    /// bare file name, no path separator.
    #[arg(value_name = "file-name", long)]
    texture_name_prefix: Option<String>,

    /// Bake a custom material map, `<file-name> <channels>`, repeatable, where
    /// `channels` is a comma-separated `R=<expr>,...` list over the `R`, `G`,
    /// `B`, `A` channels, at least one named. The file name is written beside
    /// the mesh, no path separator.
    ///
    /// Each `<expr>` is one of:
    /// - `<property>`: a voxel property by name, as `metallic` or
    ///   `baseColor`
    /// - `<property>.<r|g|b|a|x|y|z|w>`: one component of a color property,
    ///   as `baseColor.r`
    /// - `1-<property>`: the inverse `1 - value`, as `1-roughness`
    /// - 0, 1: a constant channel
    ///
    /// Property keys are voxj properties or `--define-property` aliases.
    #[arg(
        value_names = ["file-name", "channels"],
        long = "texture-map",
        num_args = 2,
        action = clap::ArgAction::Append,
        verbatim_doc_comment,
    )]
    texture_map: Vec<String>,

    /// Name a custom property for `--texture-map`, `<property> <name>`,
    /// repeatable, as `--define-property sss subsurface`. The property's type
    /// is read from its value pool in its winning layer's palette, not
    /// declared.
    #[arg(
        value_names = ["property", "name"],
        long = "define-property",
        num_args = 2,
        action = clap::ArgAction::Append,
    )]
    define_property: Vec<String>,

    /// Where the baked images go. Defaults to `embedded` for `.glb` and
    /// `external` for `.gltf`.
    #[arg(
        value_name = "texture-storage",
        long,
        value_parser = cli_value_parser::<ResourceStorage>()
    )]
    texture_storage: Option<ResourceStorage>,

    /// Choose the object by hierarchy-path glob, matched as `hierarchy show`
    /// matches node paths, so a node path selects its subtree. Repeatable;
    /// unions with `--select-index`.
    #[arg(value_name = "select", long)]
    select: Vec<String>,

    /// Choose the object by index, an integer or an `a-b` range. Repeatable;
    /// unions with `--select`.
    #[arg(value_name = "select-index", long, value_parser = parse_index_range)]
    select_index: Vec<IndexRange>,
}

impl Mesh {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let format = self
            .to
            .or_else(|| {
                let extension = self.output.as_deref()?.extension()?.to_str()?;
                MeshFormat::from_extension(extension)
            })
            .unwrap_or(MeshFormat::Glb);

        let output = self
            .output
            .clone()
            .unwrap_or_else(|| self.input.with_extension(format.extension()));

        // The unwrap atlas, and the computed-occlusion maps only it can hold, are
        // a later pass; only the palette atlas bakes for now. This runs before
        // the maps resolve, so their computed-occlusion checks fire only under
        // the palette atlas.
        if let Atlas::Unwrap = self.atlas {
            return Err(Error::usage(
                "--atlas unwrap is not yet supported; use --atlas palette",
            ));
        }

        // The image storage follows the target unless set: a `.glb` embeds, a
        // `.gltf` writes loose files beside itself.
        let storage = self.texture_storage.unwrap_or(match format {
            MeshFormat::Glb => ResourceStorage::Embedded,
            MeshFormat::Gltf => ResourceStorage::External,
        });

        let maps = self.resolve_maps(&output)?;

        let object_indices = dependencies.resolve_objects(
            &self.input,
            self.from,
            &self.select,
            &self.select_index,
        )?;

        // `mesh` outputs one object, so the selection must name exactly one; the
        // resolver stays flag-agnostic and this policy, with its flag-named
        // guidance, lives here on the command.
        let object_index = match object_indices.as_slice() {
            [object_index] => *object_index,

            [] => {
                return Err(Error::usage(
                    "no object matched the selection; check --select and --select-index",
                ));
            }

            object_indices => {
                return Err(Error::usage(format!(
                    "the selection resolved to {} objects, but `mesh` outputs exactly one; \
                     narrow it with --select or --select-index",
                    object_indices.len(),
                )));
            }
        };

        let request = MaterialMeshRequest {
            method: self.method,
            scale: self.voxel_size.0,
            maps,
            storage,
            shape: self.texture_shape,
        };

        dependencies.mesh_object(
            &self.input,
            self.from,
            &output,
            format,
            object_index,
            &request,
        )
    }

    /// Resolves the `--texture` presets, then the `--texture-map` custom maps,
    /// into the maps the writer bakes. Preset names follow the `--texture-name`
    /// / `--texture-name-prefix` precedence; custom names are given. Errors on
    /// two maps with the same file name, so the presets-first order is only a
    /// layout, not a flag-order guarantee.
    fn resolve_maps(&self, output: &Path) -> Result<Vec<MaterialMap>> {
        let stem = output
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("mesh");

        let mut maps = self.resolve_presets(stem)?;
        maps.extend(self.resolve_custom_maps()?);

        self.check_unique_names(&maps)?;

        Ok(maps)
    }

    /// Resolves the `--texture` presets into their maps, naming each by the
    /// three-level precedence and rejecting a `--texture-name` that is unbaked or
    /// repeated.
    fn resolve_presets(&self, stem: &str) -> Result<Vec<MaterialMap>> {
        if let Some(prefix) = &self.texture_name_prefix {
            require_file_name(prefix).map_err(Error::usage)?;
        }

        // The presets to bake, a bundle expanded in place, in `--texture` order.
        let presets: Vec<Texture> = self.texture.iter().flat_map(|arg| arg.textures()).collect();

        // The explicit `--texture-name` overrides, each occurrence's two tokens
        // paired into a typed value, keyed by preset and rejecting a preset named
        // twice.
        let texture_names = self
            .texture_name
            .chunks(2)
            .map(|pair| TextureName::new(&pair[0], &pair[1]))
            .collect::<Result<Vec<_>>>()?;

        let mut names: HashMap<Texture, String> = HashMap::new();
        for texture_name in &texture_names {
            if names
                .insert(texture_name.preset(), texture_name.file_name().to_owned())
                .is_none()
            {
                continue;
            }

            return Err(Error::usage(format!(
                "--texture-name names `{}` twice",
                texture_name.preset().cli_name()
            )));
        }

        // A named preset must actually be baked, else the name applies to nothing.
        let baked: HashSet<Texture> = presets.iter().copied().collect();
        for texture_name in &texture_names {
            if baked.contains(&texture_name.preset()) {
                continue;
            }

            return Err(Error::usage(format!(
                "--texture-name names `{}`, which is not being baked; add it to --texture",
                texture_name.preset().cli_name()
            )));
        }

        presets
            .into_iter()
            .map(|preset| {
                if let Texture::ComputedOcclusion = preset {
                    return Err(computed_occlusion_unsupported());
                }
                Ok(preset.map(self.preset_name(&names, preset, stem)))
            })
            .collect()
    }

    /// The file name a preset map takes: an explicit `--texture-name`, else a
    /// `--texture-name-prefix` with the preset appended verbatim, else the output
    /// stem plus a `-` and the preset.
    fn preset_name(&self, names: &HashMap<Texture, String>, preset: Texture, stem: &str) -> String {
        if let Some(name) = names.get(&preset) {
            return name.clone();
        }

        match &self.texture_name_prefix {
            Some(prefix) => format!("{prefix}{}.png", preset.cli_name()),
            None => format!("{stem}-{}.png", preset.cli_name()),
        }
    }

    /// Resolves the `--texture-map` custom maps, pairing each occurrence into a
    /// typed value and resolving its channels against the `--define-property`
    /// bindings.
    fn resolve_custom_maps(&self) -> Result<Vec<MaterialMap>> {
        let bindings = self.property_bindings()?;

        // clap's `num_args = 2` guarantees an even length, so each chunk is a
        // full pair.
        let maps = self
            .texture_map
            .chunks(2)
            .map(|pair| TextureMap::new(&pair[0], &pair[1]))
            .collect::<Result<Vec<_>>>()?;

        maps.iter().map(|map| map.resolve(&bindings)).collect()
    }

    /// Pairs each `--define-property <property> <name>` occurrence into a
    /// typed binding. clap's `num_args = 2` guarantees an even length, so each
    /// chunk is a full pair.
    fn property_bindings(&self) -> Result<Vec<PropertyBinding>> {
        self.define_property
            .chunks(2)
            .map(|pair| PropertyBinding::new(&pair[0], &pair[1]))
            .collect()
    }

    /// Rejects two maps that resolve to the same file name, which would race to
    /// write one image beside the mesh.
    fn check_unique_names(&self, maps: &[MaterialMap]) -> Result<()> {
        let mut seen = HashSet::new();
        for map in maps {
            if seen.insert(map.name.as_str()) {
                continue;
            }

            return Err(Error::usage(format!(
                "two maps resolve to the file name `{}`; name them apart with \
                 --texture-name, or drop the duplicate",
                map.name
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Mesh;
    use clap::{CommandFactory, Parser};
    use std::path::Path;
    use voxsmith::AtlasShape;

    /// The resolved map file names for `args`, meshed to `output`, in order.
    fn names(args: &[&str], output: &str) -> Vec<String> {
        let mut argv = vec!["mesh", "model.vox"];
        argv.extend_from_slice(args);
        Mesh::try_parse_from(argv)
            .unwrap()
            .resolve_maps(Path::new(output))
            .unwrap()
            .into_iter()
            .map(|map| map.name)
            .collect()
    }

    /// Whether `args` resolve their maps or error, meshed to `model.glb`.
    fn resolves(args: &[&str]) -> bool {
        let mut argv = vec!["mesh", "model.vox"];
        argv.extend_from_slice(args);
        Mesh::try_parse_from(argv)
            .unwrap()
            .resolve_maps(Path::new("model.glb"))
            .is_ok()
    }

    #[test]
    fn a_preset_defaults_its_name_from_the_output_stem() {
        assert_eq!(
            names(&["--texture", "albedo"], "turret.glb"),
            ["turret-albedo.png"]
        );
    }

    #[test]
    fn the_pbr_bundle_expands_to_albedo_orm_and_emissive_in_order() {
        assert_eq!(
            names(&["--texture", "pbr"], "model.glb"),
            ["model-albedo.png", "model-orm.png", "model-emissive.png",]
        );
    }

    #[test]
    fn repeated_flags_and_a_bundle_bake_several() {
        assert_eq!(
            names(&["--texture", "albedo", "--texture", "orm"], "model.glb"),
            ["model-albedo.png", "model-orm.png"]
        );
        assert_eq!(
            names(&["--texture", "pbr", "--texture", "roughness"], "model.glb"),
            [
                "model-albedo.png",
                "model-orm.png",
                "model-emissive.png",
                "model-roughness.png",
            ]
        );
    }

    #[test]
    fn an_explicit_name_beats_a_prefix_beats_the_stem() {
        // Albedo is named outright; orm and emissive take the prefix.
        assert_eq!(
            names(
                &[
                    "--texture",
                    "pbr",
                    "--texture-name",
                    "albedo",
                    "skin.png",
                    "--texture-name-prefix",
                    "hero-",
                ],
                "turret.glb",
            ),
            ["skin.png", "hero-orm.png", "hero-emissive.png"]
        );
    }

    #[test]
    fn a_prefix_alone_replaces_the_stem_for_every_preset() {
        assert_eq!(
            names(
                &[
                    "--texture",
                    "albedo",
                    "--texture",
                    "orm",
                    "--texture-name-prefix",
                    "hero-",
                ],
                "turret.glb"
            ),
            ["hero-albedo.png", "hero-orm.png"]
        );
    }

    #[test]
    fn a_prefix_carries_its_own_separator() {
        // The preset follows the prefix verbatim, so a trailing `.` stays a `.`
        // rather than forcing the default `-`.
        assert_eq!(
            names(
                &["--texture", "albedo", "--texture-name-prefix", "test."],
                "turret.glb"
            ),
            ["test.albedo.png"]
        );
    }

    #[test]
    fn a_custom_map_follows_the_presets_and_keeps_its_given_name() {
        assert_eq!(
            names(
                &[
                    "--texture",
                    "albedo",
                    "--texture-map",
                    "skin.png",
                    "R=metallic"
                ],
                "model.glb",
            ),
            ["model-albedo.png", "skin.png"]
        );
    }

    #[test]
    fn a_texture_name_for_an_unbaked_preset_is_rejected() {
        assert!(!resolves(&[
            "--texture",
            "albedo",
            "--texture-name",
            "orm",
            "x.png",
        ]));
    }

    #[test]
    fn a_preset_named_twice_is_rejected() {
        assert!(!resolves(&[
            "--texture",
            "albedo",
            "--texture-name",
            "albedo",
            "a.png",
            "--texture-name",
            "albedo",
            "b.png",
        ]));
    }

    #[test]
    fn two_maps_with_the_same_resolved_name_are_rejected() {
        // Two albedo presets both default to `model-albedo.png`.
        assert!(!resolves(&["--texture", "albedo", "--texture", "albedo"]));

        // A custom map can collide with a preset's default name too.
        assert!(!resolves(&[
            "--texture",
            "albedo",
            "--texture-map",
            "model-albedo.png",
            "R=metallic",
        ]));
    }

    #[test]
    fn an_empty_or_path_prefix_is_rejected() {
        assert!(!resolves(&[
            "--texture",
            "albedo",
            "--texture-name-prefix",
            ""
        ]));
        assert!(!resolves(&[
            "--texture",
            "albedo",
            "--texture-name-prefix",
            "dir/hero",
        ]));
    }

    #[test]
    fn a_texture_map_with_a_path_name_is_rejected() {
        assert!(!resolves(&[
            "--texture-map",
            "textures/skin.png",
            "R=metallic",
        ]));
    }

    #[test]
    fn a_define_property_alias_reaches_a_spaced_name_in_a_map() {
        // The two-token form quotes a voxel name with spaces; the space-free
        // alias then stands in for it in --texture-map.
        assert!(resolves(&[
            "--define-property",
            "spark",
            "super emissive thing",
            "--texture-map",
            "spark.png",
            "R=spark",
        ]));
    }

    #[test]
    fn a_spaced_property_typed_straight_into_a_map_is_rejected() {
        assert!(!resolves(&[
            "--texture-map",
            "spark.png",
            "R=super emissive thing",
        ]));
    }

    #[test]
    fn computed_occlusion_is_rejected_under_the_palette_atlas() {
        assert!(!resolves(&["--texture", "computed-occlusion"]));
    }

    #[test]
    fn a_bundle_cannot_be_named_by_texture_name() {
        // `pbr` is a bundle, not a single-map preset the left side accepts.
        assert!(!resolves(&[
            "--texture",
            "pbr",
            "--texture-name",
            "pbr",
            "x.png"
        ]));
    }

    #[test]
    fn the_texture_shape_defaults_to_pot_and_parses_keywords_and_a_pixel_size() {
        let default = Mesh::try_parse_from(["mesh", "model.vox"]).unwrap();
        assert_eq!(default.texture_shape, AtlasShape::Pot);

        for (value, expected) in [
            ("line", AtlasShape::Line),
            ("fit", AtlasShape::Fit),
            ("square", AtlasShape::Square),
            ("pot", AtlasShape::Pot),
            ("256", AtlasShape::Exact(256)),
        ] {
            let parsed =
                Mesh::try_parse_from(["mesh", "model.vox", "--texture-shape", value]).unwrap();
            assert_eq!(parsed.texture_shape, expected);
        }

        // Neither a stray keyword nor a zero side parses.
        assert!(Mesh::try_parse_from(["mesh", "model.vox", "--texture-shape", "huge"]).is_err());
        assert!(Mesh::try_parse_from(["mesh", "model.vox", "--texture-shape", "0"]).is_err());
    }

    #[test]
    fn hidden_variants_are_absent_from_help() {
        let command = Mesh::command();
        let visible = |id: &str| -> Vec<String> {
            command
                .get_arguments()
                .find(|arg| arg.get_id() == id)
                .expect("the argument exists")
                .get_possible_values()
                .into_iter()
                .filter(|value| !value.is_hide_set())
                .map(|value| value.get_name().to_owned())
                .collect()
        };

        // The presets and the bundle are listed; computed-occlusion is hidden.
        let textures = visible("texture");
        assert!(textures.contains(&"albedo".to_owned()));
        assert!(textures.contains(&"pbr".to_owned()));
        assert!(!textures.contains(&"computed-occlusion".to_owned()));

        // Only the shipped palette atlas is listed; unwrap is hidden.
        assert_eq!(visible("atlas"), ["palette".to_owned()]);
    }
}
