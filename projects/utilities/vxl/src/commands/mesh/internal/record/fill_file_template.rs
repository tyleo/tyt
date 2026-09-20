/// The file name `template` spells with `{file-stem}` replaced by `file_stem`.
pub(crate) fn fill_file_template(template: &str, file_stem: &str) -> String {
    template.replace("{file-stem}", file_stem)
}

#[cfg(test)]
mod tests {
    use super::fill_file_template;

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
}
