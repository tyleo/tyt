use crate::{
    Atlas, AttributeBinding, AttributeType, ChannelPacking, ChannelSource, ColorComponent,
    Dependencies, Format, MeshFormat, MeshMethod, MeshTextureMap, ResourceStorage, Result,
    SelectIndex, Texture, TextureBake,
};
use clap::{Parser, ValueEnum};
use std::{
    collections::HashMap,
    io::{Error as IOError, ErrorKind},
    path::{Path, PathBuf},
};
use voxsmith::{BASE_COLOR_FACTOR, EMISSIVE_FACTOR};

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
    #[arg(value_name = "to", long)]
    to: Option<MeshFormat>,

    /// Source voxel format. Inferred from the input extension when omitted.
    #[arg(value_name = "from", long)]
    from: Option<Format>,

    /// Real-world edge length of one voxel in meters, applied as a uniform scale
    /// to every output vertex.
    #[arg(value_name = "scale", long, default_value = "1.0")]
    scale: f64,

    /// Meshing strategy.
    #[arg(value_name = "method", long, default_value = "greedy")]
    method: MeshMethod,

    /// Material-map atlas layout. Only `palette` is supported for now.
    #[arg(value_name = "atlas", long, default_value = "palette")]
    atlas: Atlas,

    /// The object layer whose materials this mesh bakes, a 0-based index into
    /// the object's layers in reference order, defaulting to the first.
    #[arg(value_name = "layer", long, default_value = "0")]
    layer: usize,

    /// Bake a preset material map, `<name> [path]`, repeatable. Quote the value
    /// to override the default path, as `--texture "albedo model-albedo.png"`.
    ///
    /// Presets:
    /// - albedo: RGBA base color
    /// - orm: glTF occlusion, roughness, metallic
    /// - metallic-roughness: glTF metallic, roughness
    /// - metallic-smoothness: Unity metallic, smoothness
    /// - mse: metallic, smoothness, emissive
    /// - emissive: self-lit base color
    /// - occlusion: grayscale occlusion
    /// - roughness: grayscale roughness
    /// - smoothness: grayscale smoothness
    #[arg(value_name = "texture", long, verbatim_doc_comment)]
    texture: Vec<String>,

    /// Bake a custom material map, `<path> <channels>`, repeatable, where
    /// `channels` is a comma-separated `R=<expr>,...` list over the `R`, `G`,
    /// `B`, `A` channels, at least one named.
    ///
    /// Each `<expr>` is one of:
    /// - <attribute>: a voxel attribute by name, as `metallicFactor` or
    ///   `baseColorFactor`
    /// - <attribute>.<r|g|b|a>: one component of a color attribute, as
    ///   `baseColorFactor.r`
    /// - 1-<attribute>: the inverse `1 - value`, as `1-roughnessFactor`
    /// - 0, 1: a constant channel
    ///
    /// Attribute keys are voxj attributes or `--define-attribute` aliases;
    /// `smoothness` is shorthand for `1-roughnessFactor`.
    #[arg(
        value_names = ["path", "channels"],
        long = "texture-map",
        num_args = 2,
        action = clap::ArgAction::Append,
        verbatim_doc_comment,
    )]
    texture_map: Vec<String>,

    /// Name a custom attribute for `--texture-map`, `<name> <key> [type]`,
    /// repeatable. Quote the whole value, as `--define-attribute "tint tint color"`.
    #[arg(value_name = "define-attribute", long = "define-attribute")]
    define_attribute: Vec<AttributeBinding>,

    /// Where the baked images go. Defaults to `embedded` for `.glb` and
    /// `external` for `.gltf`.
    #[arg(value_name = "texture-storage", long)]
    texture_storage: Option<ResourceStorage>,

    /// Choose the object by hierarchy-path glob, matched as `hierarchy show`
    /// matches node paths, so a node path selects its subtree. Repeatable;
    /// unions with `--select-index`.
    #[arg(value_name = "select", long)]
    select: Vec<String>,

    /// Choose the object by index, an integer or an `a-b` range. Repeatable;
    /// unions with `--select`.
    #[arg(value_name = "select-index", long)]
    select_index: Vec<SelectIndex>,
}

impl Mesh {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        if self.scale <= 0.0 || self.scale.is_nan() {
            return Err(usage("--scale must be greater than 0"));
        }

        let format = self
            .to
            .or_else(|| self.output.as_deref().and_then(MeshFormat::from_path))
            .unwrap_or(MeshFormat::Glb);

        let output = self
            .output
            .clone()
            .unwrap_or_else(|| self.input.with_extension(format.extension()));

        // The unwrap atlas, and the computed-occlusion map it carries, are a
        // later pass; only the palette atlas bakes for now.
        if let Atlas::Unwrap = self.atlas {
            return Err(usage(
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

        let objects = dependencies.resolve_objects(
            &self.input,
            self.from,
            &self.select,
            &self.select_index,
        )?;

        // `mesh` outputs one object, so the selection must name exactly one; the
        // resolver stays flag-agnostic and this policy, with its flag-named
        // guidance, lives here on the command.
        let object = match objects.as_slice() {
            [object] => *object,

            [] => {
                return Err(usage(
                    "no object matched the selection; check --select and --select-index",
                ));
            }

            objects => {
                return Err(usage(&format!(
                    "the selection resolved to {} objects, but `mesh` outputs exactly one; \
                     narrow it with --select or --select-index",
                    objects.len(),
                )));
            }
        };

        dependencies.mesh_object(
            &self.input,
            self.from,
            &output,
            format,
            self.scale,
            self.method,
            object,
            self.layer,
            &maps,
            storage,
        )
    }

    /// Resolves the `--texture` presets and `--texture-map` packings into the
    /// maps the writer bakes, in flag order, presets first. Custom packings read
    /// the `--define-attribute` bindings; presets always read the spec
    /// attributes.
    fn resolve_maps(&self, output: &Path) -> Result<Vec<MeshTextureMap>> {
        let stem = output
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("mesh");

        let bindings: HashMap<&str, (&str, AttributeType)> = self
            .define_attribute
            .iter()
            .map(|binding| (binding.name(), (binding.key(), binding.ty())))
            .collect();

        let mut maps = Vec::new();

        for texture in &self.texture {
            maps.push(resolve_preset(texture, stem)?);
        }

        for chunk in self.texture_map.chunks(2) {
            maps.push(resolve_custom(&chunk[0], &chunk[1], &bindings)?);
        }

        Ok(maps)
    }
}

/// Resolves one `--texture <name> [path]` preset into a map. The default file
/// name is the mesh stem plus the preset name.
fn resolve_preset(argument: &str, stem: &str) -> Result<MeshTextureMap> {
    let mut tokens = argument.split_whitespace();

    let name = tokens
        .next()
        .ok_or_else(|| usage("--texture needs a preset name"))?;

    let path = tokens.next();

    if tokens.next().is_some() {
        return Err(usage(&format!(
            "--texture takes `<name> [path]`, but got `{argument}`"
        )));
    }

    let texture = Texture::from_str(name, true)
        .map_err(|_| usage(&format!("`{name}` is not a --texture preset")))?;

    if let Texture::ComputedOcclusion = texture {
        return Err(usage(
            "--texture computed-occlusion requires --atlas unwrap, which is not yet supported",
        ));
    }

    let file = match path {
        Some(path) => file_name(path)?,
        None => format!("{stem}-{}.png", preset_name(texture)),
    };

    Ok(MeshTextureMap {
        name: file,
        preset: Some(texture),
        bake: texture.bake(),
    })
}

/// Resolves one `--texture-map <path> <channels>` packing into a map, applying
/// the custom-attribute bindings and validating each channel.
fn resolve_custom(
    path: &str,
    channels: &str,
    bindings: &HashMap<&str, (&str, AttributeType)>,
) -> Result<MeshTextureMap> {
    let packing = channels
        .parse::<ChannelPacking>()
        .map_err(|message| usage(&message))?;

    let resolved = resolve_packing(&packing, bindings)?;

    Ok(MeshTextureMap {
        name: file_name(path)?,
        preset: None,
        bake: TextureBake::Packing(resolved),
    })
}

/// Resolves every channel of `packing` against the bindings, returning a packing
/// of the same shape whose attribute keys are concrete.
fn resolve_packing(
    packing: &ChannelPacking,
    bindings: &HashMap<&str, (&str, AttributeType)>,
) -> Result<ChannelPacking> {
    let resolved = packing
        .sources()
        .iter()
        .map(|source| resolve_source(source, bindings))
        .collect::<Result<Vec<_>>>()?;

    let channel = |index: usize| resolved.get(index).cloned();

    Ok(ChannelPacking::new(
        channel(0),
        channel(1),
        channel(2),
        channel(3),
    ))
}

/// Resolves one channel source: a binding alias becomes its concrete key, and a
/// color component is required on a color attribute and rejected on a scalar.
/// `computed-occlusion` is rejected under the palette atlas.
fn resolve_source(
    source: &ChannelSource,
    bindings: &HashMap<&str, (&str, AttributeType)>,
) -> Result<ChannelSource> {
    let ChannelSource::Attribute {
        key,
        component,
        invert,
    } = source
    else {
        if let ChannelSource::ComputedOcclusion = source {
            return Err(usage(
                "computed-occlusion requires --atlas unwrap, which is not yet supported",
            ));
        }
        return Ok(source.clone());
    };

    // A binding gives the key a concrete voxel attribute key and a kind; a bare
    // key resolves to itself, a color only when it is a built-in glTF color,
    // else a scalar.
    let (resolved_key, ty) = match bindings.get(key.as_str()) {
        Some((bound_key, ty)) => (bound_key.to_string(), *ty),
        None => match builtin_color(key) {
            Some(ty) => (key.clone(), ty),
            None => (key.clone(), AttributeType::Float),
        },
    };

    if ty.is_color() {
        match component {
            None => {
                return Err(usage(&format!(
                    "`{key}` is a color; name a component, as `{key}.r`"
                )));
            }
            Some(ColorComponent::A) if !ty.has_alpha() => {
                return Err(usage(&format!(
                    "`{key}` is a color with no alpha; use r, g, or b"
                )));
            }
            _ => {}
        }
    } else if component.is_some() {
        return Err(usage(&format!(
            "`{key}` is a scalar and has no color component"
        )));
    }

    Ok(ChannelSource::Attribute {
        key: resolved_key,
        component: *component,
        invert: *invert,
    })
}

/// The kind of a built-in glTF color attribute, or `None` otherwise. The base
/// color has alpha (`srgba`); the emissive color does not (`srgb`).
fn builtin_color(key: &str) -> Option<AttributeType> {
    match key {
        BASE_COLOR_FACTOR => Some(AttributeType::Srgba),
        EMISSIVE_FACTOR => Some(AttributeType::Srgb),
        _ => None,
    }
}

/// The CLI name of a preset, as its default file-name stem.
fn preset_name(texture: Texture) -> String {
    texture
        .to_possible_value()
        .expect("every texture preset has a value")
        .get_name()
        .to_owned()
}

/// The file name of `path`, the map's relative name beside the mesh.
fn file_name(path: &str) -> Result<String> {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_owned)
        .ok_or_else(|| usage(&format!("`{path}` has no file name")))
}

/// A usage error for a rule clap cannot express, exiting non-zero with a
/// message.
fn usage(message: &str) -> crate::Error {
    IOError::new(ErrorKind::InvalidInput, message).into()
}

#[cfg(test)]
mod tests {
    use super::{Mesh, resolve_preset, resolve_source};
    use crate::{AttributeType, ChannelSource, ColorComponent, Texture, TextureBake};
    use clap::{CommandFactory, Parser, ValueEnum};
    use std::collections::HashMap;
    use voxsmith::{BASE_COLOR_FACTOR, METALLIC_FACTOR, ROUGHNESS_FACTOR};

    fn bindings() -> HashMap<&'static str, (&'static str, AttributeType)> {
        HashMap::from([
            ("gloss", (ROUGHNESS_FACTOR, AttributeType::Float)),
            ("tint", ("tint", AttributeType::Srgba)),
            ("glow", ("glow", AttributeType::Srgb)),
        ])
    }

    fn attribute(key: &str, component: Option<ColorComponent>, invert: bool) -> ChannelSource {
        ChannelSource::Attribute {
            key: key.to_owned(),
            component,
            invert,
        }
    }

    #[test]
    fn a_preset_defaults_its_path_from_the_stem() {
        let map = resolve_preset("albedo", "model").unwrap();
        assert_eq!(map.name, "model-albedo.png");
        assert!(matches!(map.preset, Some(Texture::Albedo)));
        assert_eq!(map.bake, TextureBake::RgbaColor);
    }

    #[test]
    fn a_preset_path_overrides_the_default_and_takes_its_file_name() {
        let map = resolve_preset("orm textures/custom.png", "model").unwrap();
        assert_eq!(map.name, "custom.png");
    }

    #[test]
    fn a_preset_rejects_unknown_names_computed_occlusion_and_extra_tokens() {
        assert!(resolve_preset("bogus", "model").is_err());
        assert!(resolve_preset("computed-occlusion", "model").is_err());
        assert!(resolve_preset("albedo a b", "model").is_err());
    }

    #[test]
    fn a_binding_resolves_to_its_concrete_key() {
        // The scalar binding `gloss` reads the layer's `roughnessFactor`.
        let resolved = resolve_source(&attribute("gloss", None, true), &bindings()).unwrap();
        assert_eq!(resolved, attribute(ROUGHNESS_FACTOR, None, true));

        // The color binding `tint` reads a component of the layer's `tint`.
        let resolved = resolve_source(
            &attribute("tint", Some(ColorComponent::R), false),
            &bindings(),
        )
        .unwrap();
        assert_eq!(resolved, attribute("tint", Some(ColorComponent::R), false));
    }

    #[test]
    fn a_bare_scalar_and_base_color_validate_their_components() {
        // A scalar takes no component; `baseColorFactor` is a built-in color.
        assert!(resolve_source(&attribute(METALLIC_FACTOR, None, false), &bindings()).is_ok());
        assert!(
            resolve_source(
                &attribute(METALLIC_FACTOR, Some(ColorComponent::R), false),
                &bindings()
            )
            .is_err()
        );
        assert!(
            resolve_source(
                &attribute(BASE_COLOR_FACTOR, Some(ColorComponent::A), false),
                &bindings()
            )
            .is_ok()
        );
        assert!(resolve_source(&attribute(BASE_COLOR_FACTOR, None, false), &bindings()).is_err());
    }

    #[test]
    fn emissive_factor_is_an_alpha_less_built_in_color() {
        use voxsmith::EMISSIVE_FACTOR;

        // The emissive color is the second built-in color, so a bare channel
        // must name a component; but glTF emissive has no alpha, so `.a` is
        // rejected while `.g` is fine.
        assert!(
            resolve_source(
                &attribute(EMISSIVE_FACTOR, Some(ColorComponent::G), false),
                &bindings()
            )
            .is_ok()
        );
        assert!(resolve_source(&attribute(EMISSIVE_FACTOR, None, false), &bindings()).is_err());
        assert!(
            resolve_source(
                &attribute(EMISSIVE_FACTOR, Some(ColorComponent::A), false),
                &bindings()
            )
            .is_err()
        );
    }

    #[test]
    fn a_three_component_color_rejects_alpha() {
        // `glow` is declared `srgb`, so it has r/g/b but no alpha.
        assert!(
            resolve_source(
                &attribute("glow", Some(ColorComponent::B), false),
                &bindings()
            )
            .is_ok()
        );
        assert!(
            resolve_source(
                &attribute("glow", Some(ColorComponent::A), false),
                &bindings()
            )
            .is_err()
        );
    }

    #[test]
    fn a_color_binding_needs_a_component() {
        assert!(resolve_source(&attribute("tint", None, false), &bindings()).is_err());
    }

    #[test]
    fn constants_pass_and_computed_occlusion_is_rejected() {
        assert_eq!(
            resolve_source(&ChannelSource::One, &bindings()).unwrap(),
            ChannelSource::One
        );
        assert!(resolve_source(&ChannelSource::ComputedOcclusion, &bindings()).is_err());
    }

    #[test]
    fn the_layer_selector_defaults_to_the_first_layer_and_parses_a_value() {
        let default = Mesh::try_parse_from(["mesh", "model.vox"]).unwrap();
        assert_eq!(default.layer, 0);

        let chosen = Mesh::try_parse_from(["mesh", "model.vox", "--layer", "2"]).unwrap();
        assert_eq!(chosen.layer, 2);
    }

    #[test]
    fn the_texture_help_lists_every_usable_preset() {
        let command = Mesh::command();
        let help = command
            .get_arguments()
            .find(|arg| arg.get_id() == "texture")
            .and_then(|arg| arg.get_long_help())
            .map(|help| help.to_string())
            .expect("the --texture argument has long help");

        for texture in Texture::value_variants() {
            let name = texture
                .to_possible_value()
                .expect("every preset has a value")
                .get_name()
                .to_owned();

            // `computed-occlusion` needs the unwrap atlas, which `mesh` rejects,
            // so it stays off the list; every other preset must appear.
            if let Texture::ComputedOcclusion = texture {
                assert!(!help.contains(&name), "{name} should not be listed");
            } else {
                assert!(
                    help.contains(&name),
                    "{name} is missing from --texture help"
                );
            }
        }
    }
}
