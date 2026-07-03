use crate::{
    Atlas, AttributeBinding, AttributeType, ChannelPacking, ChannelSource, Dependencies, Format,
    MeshFormat, MeshMethod, MeshTextureMap, ResourceStorage, Result, SelectIndex, Texture,
    TextureBake,
};
use clap::{Parser, ValueEnum};
use std::{
    collections::HashMap,
    io::{Error as IOError, ErrorKind},
    path::{Path, PathBuf},
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

    /// Bake a preset material map, `<name> [path]`, repeatable. Quote the value
    /// to override the default path, as `--texture "albedo model-albedo.png"`.
    #[arg(value_name = "texture", long)]
    texture: Vec<String>,

    /// Bake a custom material map, `<path> <channels>`, repeatable, where
    /// `channels` is a comma-separated `R=<expr>,...` list.
    #[arg(
        value_names = ["path", "channels"],
        long = "texture-map",
        num_args = 2,
        action = clap::ArgAction::Append,
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

    // A binding gives the key a concrete voxel attribute key and a type; a bare
    // key resolves to itself, a color only when it is `rgba`.
    let (resolved_key, ty) = match bindings.get(key.as_str()) {
        Some((bound_key, ty)) => (bound_key.to_string(), *ty),
        None if key == "rgba" => (key.clone(), AttributeType::Color),
        None => (key.clone(), AttributeType::Scalar),
    };

    match ty {
        AttributeType::Color if component.is_none() => {
            return Err(usage(&format!(
                "`{key}` is a color; name a component, as `{key}.r`"
            )));
        }
        AttributeType::Scalar if component.is_some() => {
            return Err(usage(&format!(
                "`{key}` is a scalar and has no color component"
            )));
        }
        _ => {}
    }

    Ok(ChannelSource::Attribute {
        key: resolved_key,
        component: *component,
        invert: *invert,
    })
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
    use super::{resolve_preset, resolve_source};
    use crate::{AttributeType, ChannelSource, ColorComponent, Texture, TextureBake};
    use std::collections::HashMap;

    fn bindings() -> HashMap<&'static str, (&'static str, AttributeType)> {
        HashMap::from([
            ("gloss", ("roughness", AttributeType::Scalar)),
            ("tint", ("tint", AttributeType::Color)),
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
        // The scalar binding `gloss` reads the merged `roughness`.
        let resolved = resolve_source(&attribute("gloss", None, true), &bindings()).unwrap();
        assert_eq!(resolved, attribute("roughness", None, true));

        // The color binding `tint` reads a component of the merged `tint`.
        let resolved = resolve_source(
            &attribute("tint", Some(ColorComponent::R), false),
            &bindings(),
        )
        .unwrap();
        assert_eq!(resolved, attribute("tint", Some(ColorComponent::R), false));
    }

    #[test]
    fn a_bare_scalar_and_rgba_color_validate_their_components() {
        // A scalar takes no component; `rgba` is the one built-in color.
        assert!(resolve_source(&attribute("metallic", None, false), &bindings()).is_ok());
        assert!(
            resolve_source(
                &attribute("metallic", Some(ColorComponent::R), false),
                &bindings()
            )
            .is_err()
        );
        assert!(
            resolve_source(
                &attribute("rgba", Some(ColorComponent::A), false),
                &bindings()
            )
            .is_ok()
        );
        assert!(resolve_source(&attribute("rgba", None, false), &bindings()).is_err());
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
}
