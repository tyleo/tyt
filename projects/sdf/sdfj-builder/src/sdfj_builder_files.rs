use crate::SdfjBuilderFile;

/// The builder file at `$path`, embedded.
macro_rules! builder_file {
    ($path:literal) => {
        SdfjBuilderFile {
            path: $path,
            text: include_str!(concat!("../", $path)),
        }
    };
}

/// Every file a run of the builder needs, by its path in the package.
pub const SDFJ_BUILDER_FILES: &[SdfjBuilderFile] = &[
    builder_file!("deno.json"),
    builder_file!("package.json"),
    builder_file!("ts/api.ts"),
    builder_file!("ts/booleans.ts"),
    builder_file!("ts/check.ts"),
    builder_file!("ts/entry.ts"),
    builder_file!("ts/font.ts"),
    builder_file!("ts/globals.d.ts"),
    builder_file!("ts/json_text.ts"),
    builder_file!("ts/library.ts"),
    builder_file!("ts/main.ts"),
    builder_file!("ts/material.ts"),
    builder_file!("ts/model_list.ts"),
    builder_file!("ts/part.ts"),
    builder_file!("ts/paths.ts"),
    builder_file!("ts/pattern.ts"),
    builder_file!("ts/primitives.ts"),
    builder_file!("ts/primitives2d.ts"),
    builder_file!("ts/profiles.ts"),
    builder_file!("ts/put_api_in_scope.ts"),
    builder_file!("ts/random.ts"),
    builder_file!("ts/sdfj_document.ts"),
    builder_file!("ts/sdfj_json.ts"),
    builder_file!("ts/shades.ts"),
    builder_file!("ts/shape2d.ts"),
    builder_file!("ts/shape3d.ts"),
    builder_file!("ts/step.ts"),
    builder_file!("ts/strict_record.ts"),
    builder_file!("ts/types.ts"),
    builder_file!("ts/vars.ts"),
];

#[cfg(test)]
mod tests {
    use crate::SDFJ_BUILDER_FILES;
    use std::{fs, path::Path};

    /// The names of the files in `directory`, without its subdirectories.
    fn file_names(directory: &Path) -> impl Iterator<Item = String> {
        fs::read_dir(directory)
            .unwrap()
            .map(|entry| entry.unwrap())
            .filter(|entry| entry.file_type().unwrap().is_file())
            .map(|entry| entry.file_name().into_string().unwrap())
    }

    #[test]
    fn the_files_list_every_config_and_source_file() {
        let package = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut names: Vec<String> = file_names(package)
            .filter(|name| name.ends_with(".json"))
            .chain(
                file_names(&package.join("ts"))
                    .filter(|name| name.ends_with(".ts") && !name.ends_with(".test.ts"))
                    .map(|name| format!("ts/{name}")),
            )
            .collect();
        names.sort();
        let paths: Vec<&str> = SDFJ_BUILDER_FILES.iter().map(|file| file.path).collect();
        assert_eq!(names, paths);
    }
}
