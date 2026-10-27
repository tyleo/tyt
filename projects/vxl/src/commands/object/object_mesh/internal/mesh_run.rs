use crate::{Error, Result};
use branded_id::U32Id;
use std::path::{Path, PathBuf};
use voxcore::{BVoxObject, VoxExt, VoxMain};

/// One output of a meshing run and the objects it holds, each beside the stem
/// its files fill `{file-stem}` with.
#[derive(Debug, PartialEq)]
pub struct MeshRun {
    /// The mesh the run writes.
    pub output: PathBuf,

    /// The objects the mesh holds, in order, each with its file stem.
    pub targets: Vec<(U32Id<BVoxObject>, String)>,
}

impl MeshRun {
    /// The runs meshing `object_ids` of `main` to `output` under `base_stem`:
    /// one run holding every object, or with `split` one run per object beside
    /// `output` under the object's name. A run over several objects, or a
    /// split run, joins the object's name to the stem with a hyphen so the
    /// objects' files stay apart.
    pub(crate) fn plan<T: VoxExt>(
        main: &VoxMain<T>,
        output: &Path,
        base_stem: &str,
        object_ids: &[U32Id<BVoxObject>],
        split: bool,
    ) -> Result<Vec<MeshRun>> {
        if !split && object_ids.len() == 1 {
            return Ok(vec![MeshRun {
                output: output.to_path_buf(),
                targets: vec![(object_ids[0], base_stem.to_owned())],
            }]);
        }

        let stems = object_stems(main, object_ids)?;

        let targets = object_ids
            .iter()
            .zip(&stems)
            .map(|(&object_id, stem)| (object_id, format!("{base_stem}-{stem}")));

        if split {
            return Ok(targets
                .zip(&stems)
                .map(|(target, stem)| MeshRun {
                    output: split_output(output, stem),
                    targets: vec![target],
                })
                .collect());
        }

        Ok(vec![MeshRun {
            output: output.to_path_buf(),
            targets: targets.collect(),
        }])
    }
}

/// The stem component each object of `object_ids` lends to its files in a run
/// that tells the objects' files apart: the object's name. An empty name, a
/// path separator, or a name two of the objects share is a usage error.
fn object_stems<T: VoxExt>(
    main: &VoxMain<T>,
    object_ids: &[U32Id<BVoxObject>],
) -> Result<Vec<String>> {
    let mut stems: Vec<String> = Vec::with_capacity(object_ids.len());

    for &object_id in object_ids {
        let name = main
            .object(object_id)
            .expect("the selection resolved an id from the main's objects")
            .name();

        if name.is_empty() {
            return Err(Error::usage(format!(
                "object {object_id} has no name, and a run over several objects or `--split-files` \
                 names each object's files by it; name the object or select one"
            )));
        }

        if name.contains('/') || name.contains('\\') {
            return Err(Error::usage(format!(
                "object {object_id} is named `{name}`, which names its files, so it cannot contain \
                 a path separator"
            )));
        }

        if let Some(earlier) = stems.iter().position(|stem| stem == name) {
            return Err(Error::usage(format!(
                "objects {} and {object_id} share the name `{name}`, which names their files; \
                 select one or rename them",
                object_ids[earlier]
            )));
        }

        stems.push(name.to_owned());
    }

    Ok(stems)
}

/// The output one object of a split run writes: `output`'s stem, a hyphen,
/// and `stem`, under `output`'s extension beside it.
fn split_output(output: &Path, stem: &str) -> PathBuf {
    let output_stem = output
        .file_stem()
        .expect("the output path carries a file name")
        .to_string_lossy();

    let mut file_name = format!("{output_stem}-{stem}");

    if let Some(extension) = output.extension() {
        file_name = format!("{file_name}.{}", extension.to_string_lossy());
    }

    output.with_file_name(file_name)
}

#[cfg(test)]
mod tests {
    use crate::commands::{
        MeshRun,
        object::object_mesh::internal::mesh_run::{object_stems, split_output},
    };
    use branded_id::U32Id;
    use std::path::{Path, PathBuf};
    use ty_math::TyVector3U32;
    use voxcore::{BVoxObject, VoxMain, VoxObject};

    /// A main holding `turret` and `base`, and their ids.
    fn main() -> (VoxMain, [U32Id<BVoxObject>; 2]) {
        let mut main: VoxMain = VoxMain::default();
        let ids = ["turret", "base"].map(|name| {
            main.retain_object(VoxObject::new(name.to_owned(), TyVector3U32::ONE).unwrap())
                .unwrap()
        });
        (main, ids)
    }

    #[test]
    fn one_object_keeps_the_output_and_the_stem() {
        let (main, [turret, _]) = main();

        let runs = MeshRun::plan(&main, Path::new("scene.glb"), "scene", &[turret], false).unwrap();

        assert_eq!(
            runs,
            vec![MeshRun {
                output: PathBuf::from("scene.glb"),
                targets: vec![(turret, "scene".to_owned())],
            }]
        );
    }

    #[test]
    fn several_objects_share_the_output_under_qualified_stems() {
        let (main, [turret, base]) = main();

        let runs = MeshRun::plan(
            &main,
            Path::new("scene.glb"),
            "scene",
            &[turret, base],
            false,
        )
        .unwrap();

        assert_eq!(
            runs,
            vec![MeshRun {
                output: PathBuf::from("scene.glb"),
                targets: vec![
                    (turret, "scene-turret".to_owned()),
                    (base, "scene-base".to_owned()),
                ],
            }]
        );
    }

    #[test]
    fn a_split_run_writes_one_output_per_object_even_for_one() {
        let (main, [turret, base]) = main();

        let runs = MeshRun::plan(
            &main,
            Path::new("out/scene.glb"),
            "custom",
            &[turret, base],
            true,
        )
        .unwrap();

        assert_eq!(
            runs,
            vec![
                MeshRun {
                    output: PathBuf::from("out/scene-turret.glb"),
                    targets: vec![(turret, "custom-turret".to_owned())],
                },
                MeshRun {
                    output: PathBuf::from("out/scene-base.glb"),
                    targets: vec![(base, "custom-base".to_owned())],
                },
            ]
        );

        let runs = MeshRun::plan(&main, Path::new("scene.glb"), "scene", &[base], true).unwrap();
        assert_eq!(runs[0].output, PathBuf::from("scene-base.glb"));
        assert_eq!(runs[0].targets, vec![(base, "scene-base".to_owned())]);
    }

    /// A main holding one object per name, and their ids.
    fn main_of(names: &[&str]) -> VoxMain {
        let mut main: VoxMain = VoxMain::default();
        for name in names {
            main.retain_object(VoxObject::new(name.to_string(), TyVector3U32::ONE).unwrap())
                .unwrap();
        }
        main
    }

    /// The stems of every object of the main named by `names`, or the error.
    fn stems_of(names: &[&str]) -> Result<Vec<String>, String> {
        let main = main_of(names);
        let object_ids: Vec<_> = main
            .iter_objects()
            .map(|(object_id, _)| object_id)
            .collect();
        object_stems(&main, &object_ids).map_err(|error| error.to_string())
    }

    #[test]
    fn the_stems_are_the_names() {
        assert_eq!(stems_of(&["turret", "base"]).unwrap(), ["turret", "base"]);
    }

    #[test]
    fn an_empty_name_a_path_or_a_shared_name_errors() {
        assert!(
            stems_of(&["turret", ""])
                .unwrap_err()
                .contains("object 1 has no name")
        );
        assert!(
            stems_of(&["a/b"])
                .unwrap_err()
                .contains("cannot contain a path separator")
        );
        assert!(
            stems_of(&["turret", "base", "turret"])
                .unwrap_err()
                .contains("objects 0 and 2 share the name `turret`")
        );
    }

    #[test]
    fn the_object_joins_the_stem_under_the_extension() {
        assert_eq!(
            split_output(Path::new("out/scene.glb"), "turret"),
            PathBuf::from("out/scene-turret.glb")
        );
        assert_eq!(
            split_output(Path::new("scene"), "turret"),
            PathBuf::from("scene-turret")
        );
    }
}
