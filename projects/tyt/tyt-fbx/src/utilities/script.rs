/// Script content to be executed by Blender.
pub struct Script<'a> {
    pub relative_file_path: &'a str,
    pub content: &'a str,
}
