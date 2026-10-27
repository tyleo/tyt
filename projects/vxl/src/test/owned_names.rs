/// Each of `names` as an owned string.
pub fn owned_names(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).to_owned()).collect()
}
