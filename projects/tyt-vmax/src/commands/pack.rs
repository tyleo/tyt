use crate::{Dependencies, Result};
use clap::Parser;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

const HISTORY_EXTENSIONS: &[&str] = &["vmaxhb", "vmaxhvsb", "vmaxhvsc"];

/// Packs a .vmax directory by stripping history, renumbering the surviving
/// contents/palettes/thumbnails, and deleting files no longer referenced by
/// `scene.json`.
#[derive(Clone, Debug, Parser)]
#[command(name = "pack")]
pub struct Pack {
    /// The input `.vmax` directory to pack.
    #[arg(value_name = "input-vmax")]
    input_vmax: PathBuf,

    /// Optional output `.vmax` directory. If provided, copies the input first.
    #[arg(value_name = "output-vmax", long)]
    output_vmax: Option<PathBuf>,
}

impl Pack {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let target = if let Some(ref output) = self.output_vmax {
            dependencies.copy_dir(&self.input_vmax, output)?;
            output.clone()
        } else {
            self.input_vmax.clone()
        };

        let scene_path = target.join("scene.json");
        let scene_bytes = dependencies.read_file(&scene_path)?;
        let object_refs = dependencies.scene_object_refs(&scene_bytes)?;

        // Map each still-referenced content/palette suffix to a compact,
        // blank-first replacement (`""`, `"1"`, `"2"`, ...).
        let content_map = renumber(referenced(
            object_refs.iter().map(|(data, _)| data.as_str()),
            "contents",
            ".vmaxb",
        ));
        let palette_map = renumber(referenced(
            object_refs.iter().map(|(_, pal)| pal.as_str()),
            "palette",
            ".png",
        ));

        // Rewrite scene.json: clear history fields and repoint `data`/`pal`.
        let data_renames = full_renames(&content_map, "contents", ".vmaxb");
        let pal_renames = full_renames(&palette_map, "palette", ".png");
        let packed = dependencies.pack_scene_json(
            &scene_bytes,
            &borrow_pairs(&data_renames),
            &borrow_pairs(&pal_renames),
        )?;
        dependencies.write_file(&scene_path, &packed)?;

        // Plan file renames/removals. History is always stripped; content and
        // palette families are only touched when the scene still has objects.
        let renumber_files = !object_refs.is_empty();
        let quicklook_dir = target.join("QuickLook");
        let root_entries = dependencies.list_dir(&target)?;
        let has_quicklook = root_entries
            .iter()
            .any(|p| p.file_name().is_some_and(|n| n == "QuickLook"));
        let mut plan = Plan::default();

        for entry in &root_entries {
            let Some(name) = entry.file_name().and_then(|n| n.to_str()) else {
                continue;
            };

            let is_history = entry
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| HISTORY_EXTENSIONS.contains(&ext));
            if is_history {
                plan.removals.push(entry.clone());
                continue;
            }

            if !renumber_files {
                continue;
            }

            if let Some(suffix) = parse_suffix(name, "contents", ".selection.vmaxb") {
                plan_file(
                    &content_map,
                    &target,
                    entry,
                    suffix,
                    "contents",
                    ".selection.vmaxb",
                    &mut plan,
                );
            } else if let Some(suffix) = parse_suffix(name, "contents", ".vmaxb") {
                plan_file(
                    &content_map,
                    &target,
                    entry,
                    suffix,
                    "contents",
                    ".vmaxb",
                    &mut plan,
                );
            } else if let Some(suffix) = parse_suffix(name, "palette", ".settings.vmaxpsb") {
                plan_file(
                    &palette_map,
                    &target,
                    entry,
                    suffix,
                    "palette",
                    ".settings.vmaxpsb",
                    &mut plan,
                );
            } else if let Some(suffix) = parse_suffix(name, "palette", ".png") {
                plan_file(
                    &palette_map,
                    &target,
                    entry,
                    suffix,
                    "palette",
                    ".png",
                    &mut plan,
                );
            }
        }

        if renumber_files && has_quicklook {
            for entry in dependencies.list_dir(&quicklook_dir)? {
                let Some(name) = entry.file_name().and_then(|n| n.to_str()) else {
                    continue;
                };
                if let Some(suffix) = parse_suffix(name, "contents", ".vmaxb.png") {
                    plan_file(
                        &content_map,
                        &quicklook_dir,
                        &entry,
                        suffix,
                        "contents",
                        ".vmaxb.png",
                        &mut plan,
                    );
                }
            }
        }

        // Remove orphans first so freed names can't collide with renames, then
        // rename survivors ascending by original suffix so each target is free.
        let mut removed_msgs = String::new();
        for path in &plan.removals {
            dependencies.remove_file(path)?;
            removed_msgs.push_str(&format!("Removed: {}\n", path.display()));
        }

        plan.renames.sort_by_key(|(order, _, _)| *order);
        let mut renamed_msgs = String::new();
        for (_, from, to) in &plan.renames {
            dependencies.rename_file(from, to)?;
            renamed_msgs.push_str(&format!(
                "Renamed: {} -> {}\n",
                from.display(),
                to.display()
            ));
        }

        let mut output_buf = format!("Edited: {}\n", scene_path.display());
        output_buf.push_str(&renamed_msgs);
        output_buf.push_str(&removed_msgs);
        dependencies.write_stdout(output_buf.as_bytes())?;

        Ok(())
    }
}

/// If `name` is `{prefix}{suffix}{tail}` where `suffix` is empty or all ASCII
/// digits, returns the suffix; otherwise `None`.
fn parse_suffix<'a>(name: &'a str, prefix: &str, tail: &str) -> Option<&'a str> {
    let middle = name.strip_prefix(prefix)?.strip_suffix(tail)?;
    (middle.is_empty() || middle.bytes().all(|b| b.is_ascii_digit())).then_some(middle)
}

/// Numeric ordering key for a suffix; the blank suffix sorts first.
fn suffix_order(suffix: &str) -> u64 {
    suffix.parse().unwrap_or(0)
}

/// Collects the distinct, numerically sorted suffixes referenced by `values`,
/// where each reference is a `{prefix}{suffix}{tail}` filename.
fn referenced<'a>(values: impl Iterator<Item = &'a str>, prefix: &str, tail: &str) -> Vec<String> {
    let unique: HashSet<String> = values
        .filter_map(|value| parse_suffix(value, prefix, tail).map(str::to_owned))
        .collect();
    let mut suffixes: Vec<String> = unique.into_iter().collect();
    suffixes.sort_by(|a, b| suffix_order(a).cmp(&suffix_order(b)).then_with(|| a.cmp(b)));
    suffixes
}

/// Maps each referenced suffix to its compact, blank-first replacement.
fn renumber(suffixes: Vec<String>) -> HashMap<String, String> {
    suffixes
        .into_iter()
        .enumerate()
        .map(|(i, old)| {
            let new = if i == 0 { String::new() } else { i.to_string() };
            (old, new)
        })
        .collect()
}

/// Builds `({prefix}{old}{tail}, {prefix}{new}{tail})` pairs for every entry in
/// `map`, for rewriting `scene.json` references.
fn full_renames(map: &HashMap<String, String>, prefix: &str, tail: &str) -> Vec<(String, String)> {
    map.iter()
        .map(|(old, new)| {
            (
                format!("{prefix}{old}{tail}"),
                format!("{prefix}{new}{tail}"),
            )
        })
        .collect()
}

/// Borrows owned rename pairs as `&str` pairs for the dependency call.
fn borrow_pairs(pairs: &[(String, String)]) -> Vec<(&str, &str)> {
    pairs
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect()
}

/// The file operations planned during a pack: `renames` keyed by original
/// suffix order (so survivors apply ascending), and outright `removals`.
#[derive(Default)]
struct Plan {
    renames: Vec<(u64, PathBuf, PathBuf)>,
    removals: Vec<PathBuf>,
}

/// Plans one content/palette file: rename it if its suffix survived
/// renumbering, otherwise mark it for removal.
fn plan_file(
    map: &HashMap<String, String>,
    dir: &Path,
    entry: &Path,
    suffix: &str,
    prefix: &str,
    tail: &str,
    plan: &mut Plan,
) {
    match map.get(suffix) {
        Some(new) if new != suffix => {
            let new_path = dir.join(format!("{prefix}{new}{tail}"));
            plan.renames
                .push((suffix_order(suffix), entry.to_path_buf(), new_path));
        }
        Some(_) => {}
        None => plan.removals.push(entry.to_path_buf()),
    }
}
