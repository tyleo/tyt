//! End-to-end tests for `add_command_to_crate` against a throwaway crate on a
//! temp directory, covering command groups nested one, two, and three levels
//! deep.

use crate::{
    Dependencies, Error, Result,
    commands::{CreateCommand, create_command::add_command_to_crate},
};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
    sync::atomic::{AtomicU64, Ordering},
};

/// A `Dependencies` backed by the real filesystem but rooted at a fixed
/// workspace, so tests drive the scaffolder without touching the repo.
struct TestDeps {
    root: PathBuf,
}

impl Dependencies for TestDeps {
    fn create_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        Ok(fs::create_dir_all(path)?)
    }

    fn read_dir<P: AsRef<Path>>(&self, path: P) -> Result<Vec<PathBuf>> {
        let mut entries = Vec::new();
        for entry in fs::read_dir(path)? {
            entries.push(entry?.path());
        }
        Ok(entries)
    }

    fn read_to_string<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        Ok(fs::read_to_string(path)?)
    }

    fn write<P: AsRef<Path>>(&self, path: P, contents: &str) -> Result<()> {
        Ok(fs::write(path, contents)?)
    }

    fn write_stdout(&self, _contents: &[u8]) -> Result<()> {
        Ok(())
    }

    fn workspace_root(&self) -> Result<PathBuf> {
        Ok(self.root.clone())
    }
}

/// A temp workspace holding one scaffolded standalone crate, removed on drop.
struct Fixture {
    root: PathBuf,
    suffix: String,
}

impl Fixture {
    /// Scaffolds `projects/utilities/{suffix}` with an empty root command enum
    /// and an empty `commands/mod.rs`, ready for commands to be added.
    fn new(suffix: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = env::temp_dir().join(format!("tyt-meta-test-{}-{unique}", process::id()));

        let src = root.join(format!("projects/utilities/{suffix}/src"));
        fs::create_dir_all(src.join("commands")).unwrap();
        let root_enum = format!(
            "use clap::Subcommand;\n\n#[derive(Clone, Debug, Subcommand)]\npub enum {} {{}}\n\nimpl {} {{\n    pub fn execute(self, dependencies: impl crate::Dependencies) -> crate::Result<()> {{\n        match self {{}}\n    }}\n}}\n",
            pascal(suffix),
            pascal(suffix),
        );
        fs::write(src.join(format!("{suffix}.rs")), root_enum).unwrap();
        fs::write(src.join("commands/mod.rs"), "").unwrap();

        Self {
            root,
            suffix: suffix.to_string(),
        }
    }

    fn deps(&self) -> TestDeps {
        TestDeps {
            root: self.root.clone(),
        }
    }

    /// Adds a command under `groups` (kebab segments) with the given leaf name
    /// and CLI command.
    fn add(&self, name: &str, command: &str, groups: &[&str]) -> Result<()> {
        let mut parents = vec![self.suffix.clone()];
        parents.extend(groups.iter().map(|group| group.to_string()));
        let cmd = CreateCommand {
            name: name.to_string(),
            command: command.to_string(),
            description: format!("The {command} command."),
            parent: parents.clone(),
            dir: Some("utilities".to_string()),
            prefix: Some(false),
        };
        add_command_to_crate(&cmd, &self.deps(), &parents)
    }

    fn src(&self) -> PathBuf {
        self.root
            .join(format!("projects/utilities/{}/src", self.suffix))
    }

    fn read(&self, relative: &str) -> String {
        fs::read_to_string(self.src().join(relative))
            .unwrap_or_else(|_| panic!("missing file: {relative}"))
    }

    fn exists(&self, relative: &str) -> bool {
        self.src().join(relative).exists()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// PascalCase of a single lowercase segment (test crates use simple names).
fn pascal(segment: &str) -> String {
    let mut chars = segment.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[test]
fn adds_a_command_one_group_deep() {
    let fixture = Fixture::new("demo");
    fixture.add("Show", "show", &["hierarchy"]).unwrap();

    // The group became a directory with its own mod.rs, struct, and enum.
    assert!(fixture.exists("commands/hierarchy/mod.rs"));
    assert!(fixture.exists("commands/hierarchy/hierarchy.rs"));
    assert!(fixture.exists("commands/hierarchy/hierarchy_command.rs"));
    assert!(fixture.exists("commands/hierarchy/hierarchy_show.rs"));

    // The crate root registers the group directory and wires its struct.
    let root_mod = fixture.read("commands/mod.rs");
    assert!(root_mod.contains("mod hierarchy;"));
    assert!(root_mod.contains("pub use hierarchy::*;"));
    let root_enum = fixture.read("demo.rs");
    assert!(root_enum.contains("Hierarchy(Hierarchy),"));

    // The group's own mod.rs allows module_inception on the same-named struct
    // module and registers the new leaf.
    let group_mod = fixture.read("commands/hierarchy/mod.rs");
    assert!(group_mod.contains("#[allow(clippy::module_inception)]\nmod hierarchy;"));
    assert!(group_mod.contains("mod hierarchy_command;"));
    assert!(group_mod.contains("mod hierarchy_show;"));

    // The leaf is wired into the group's enum with its bare CLI name.
    let group_enum = fixture.read("commands/hierarchy/hierarchy_command.rs");
    assert!(group_enum.contains("use crate::commands::{HierarchyShow};"));
    assert!(group_enum.contains("#[command(name = \"show\")]\n    HierarchyShow(HierarchyShow),"));
    assert!(group_enum.contains("HierarchyCommand::HierarchyShow(show) => show.execute"));
}

#[test]
fn adds_a_command_two_groups_deep() {
    let fixture = Fixture::new("demo");
    fixture
        .add("Camera", "camera", &["alpha", "bravo"])
        .unwrap();

    // Directories nest; files keep the ancestor-prefixed name; the inner dir is
    // the bare segment.
    assert!(fixture.exists("commands/alpha/alpha_command.rs"));
    assert!(fixture.exists("commands/alpha/bravo/alpha_bravo.rs"));
    assert!(fixture.exists("commands/alpha/bravo/alpha_bravo_command.rs"));
    assert!(fixture.exists("commands/alpha/bravo/alpha_bravo_camera.rs"));

    // The outer group registers the inner group dir by its bare segment.
    let alpha_mod = fixture.read("commands/alpha/mod.rs");
    assert!(alpha_mod.contains("mod bravo;"));
    assert!(alpha_mod.contains("pub use bravo::*;"));

    // The inner group's struct module is NOT same-named as its dir, so no
    // module_inception allow is emitted.
    let bravo_mod = fixture.read("commands/alpha/bravo/mod.rs");
    assert!(bravo_mod.contains("mod alpha_bravo;"));
    assert!(!bravo_mod.contains("module_inception"));
    assert!(bravo_mod.contains("mod alpha_bravo_camera;"));

    // Each level's enum is wired: root <- Alpha, alpha <- AlphaBravo, bravo <- leaf.
    assert!(fixture.read("demo.rs").contains("Alpha(Alpha),"));
    assert!(
        fixture
            .read("commands/alpha/alpha_command.rs")
            .contains("AlphaBravo(AlphaBravo),")
    );
    assert!(
        fixture
            .read("commands/alpha/bravo/alpha_bravo_command.rs")
            .contains("AlphaBravoCamera(AlphaBravoCamera),")
    );
}

#[test]
fn adds_a_command_three_groups_deep() {
    let fixture = Fixture::new("demo");
    fixture
        .add("Delta", "delta", &["alpha", "bravo", "charlie"])
        .unwrap();

    assert!(fixture.exists("commands/alpha/bravo/charlie/alpha_bravo_charlie.rs"));
    assert!(fixture.exists("commands/alpha/bravo/charlie/alpha_bravo_charlie_command.rs"));
    assert!(fixture.exists("commands/alpha/bravo/charlie/alpha_bravo_charlie_delta.rs"));

    let charlie_mod = fixture.read("commands/alpha/bravo/charlie/mod.rs");
    assert!(charlie_mod.contains("mod alpha_bravo_charlie;"));
    assert!(charlie_mod.contains("mod alpha_bravo_charlie_delta;"));

    assert!(
        fixture
            .read("commands/alpha/bravo/alpha_bravo_command.rs")
            .contains("AlphaBravoCharlie(AlphaBravoCharlie),")
    );
    assert!(
        fixture
            .read("commands/alpha/bravo/charlie/alpha_bravo_charlie_command.rs")
            .contains("AlphaBravoCharlieDelta(AlphaBravoCharlieDelta),")
    );
}

#[test]
fn a_second_command_reuses_an_existing_group() {
    let fixture = Fixture::new("demo");
    fixture.add("List", "list", &["palette"]).unwrap();
    fixture.add("Show", "show", &["palette"]).unwrap();

    // The group is not recreated or duplicated; both leaves are registered and
    // wired in sorted order.
    let group_mod = fixture.read("commands/palette/mod.rs");
    assert_eq!(group_mod.matches("mod palette;").count(), 1);
    assert!(group_mod.contains("mod palette_list;"));
    assert!(group_mod.contains("mod palette_show;"));

    let group_enum = fixture.read("commands/palette/palette_command.rs");
    assert!(group_enum.contains("use crate::commands::{PaletteList, PaletteShow};"));
    let list = group_enum.find("PaletteList(PaletteList)").unwrap();
    let show = group_enum.find("PaletteShow(PaletteShow)").unwrap();
    assert!(list < show);

    // The crate root has the single `Palette` variant, added once.
    assert_eq!(
        fixture.read("demo.rs").matches("Palette(Palette),").count(),
        1
    );
}

#[test]
fn a_flat_command_with_no_groups_stays_a_single_file() {
    let fixture = Fixture::new("demo");
    fixture.add("Run", "run", &[]).unwrap();

    assert!(fixture.exists("commands/run.rs"));
    assert!(!fixture.exists("commands/run/mod.rs"));
    let root_mod = fixture.read("commands/mod.rs");
    assert!(root_mod.contains("mod run;"));
    assert!(fixture.read("demo.rs").contains("Run(Run),"));
}

#[test]
fn refuses_to_convert_an_existing_leaf_into_a_group() {
    let fixture = Fixture::new("demo");
    // `edit` is a leaf command directly under the crate.
    fixture.add("Edit", "edit", &[]).unwrap();
    // Trying to nest under it should error rather than clobber the leaf.
    let result = fixture.add("Undo", "undo", &["edit"]);
    assert!(matches!(result, Err(Error::Meta(_))));
    // The leaf file is untouched and no group enum was created.
    assert!(fixture.exists("commands/edit.rs"));
    assert!(!fixture.exists("commands/edit/edit_command.rs"));
}
