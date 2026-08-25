use crate::UsrPrefs;
use std::io::Result as IOResult;
use tyt_preferences::DeserializePrefs;

impl DeserializePrefs for UsrPrefs {
    fn deserialize_prefs(config_jsonc: &[u8], key: &str) -> IOResult<Option<Self>> {
        tyt_preferences::deserialize_prefs_jsonc(config_jsonc, key)
    }
}
