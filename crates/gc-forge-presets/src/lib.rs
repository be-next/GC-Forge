//! Presets shipped with GC-Forge.
//!
//! At compile time the `build.rs` walks `<workspace>/presets/*.yaml` and
//! generates an `&[(name, body)]` array; this module wraps it in three
//! lookup helpers used by `gc-forge presets`, `selftest`, and
//! `variance-check`.

include!(concat!(env!("OUT_DIR"), "/presets_generated.rs"));

/// Returns the names of all embedded presets, in alphabetical order.
#[must_use]
pub fn list_preset_names() -> Vec<&'static str> {
    PRESETS.iter().map(|(n, _)| *n).collect()
}

/// Returns `(name, body)` for every embedded preset.
#[must_use]
pub fn all_presets() -> &'static [(&'static str, &'static str)] {
    PRESETS
}

/// Returns the YAML body of an embedded preset by name.
#[must_use]
pub fn embedded_yaml(name: &str) -> Option<&'static str> {
    PRESETS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, body)| *body)
}

/// Returns this crate's version.
#[must_use]
pub const fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_some_presets() {
        let names = list_preset_names();
        assert!(!names.is_empty(), "no presets discovered");
        assert!(names.contains(&"steady-g1-baseline"));
    }

    #[test]
    fn embedded_bodies_parse_yaml() {
        for (name, body) in all_presets() {
            let parsed: serde_yaml::Value =
                serde_yaml::from_str(body).unwrap_or_else(|e| panic!("preset {name}: {e}"));
            assert!(parsed.is_mapping(), "{name} top level should be a mapping");
        }
    }

    #[test]
    fn embedded_yaml_lookup_returns_some() {
        assert!(embedded_yaml("steady-g1-baseline").is_some());
        assert!(embedded_yaml("nonexistent").is_none());
    }

    #[test]
    fn version_is_non_empty() {
        assert!(!version().is_empty());
    }
}
