use std::{
    fmt, fs,
    path::{Component, Path, PathBuf},
};

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const SUPPORTED_API: &str = "1.0.0";
pub const MAX_MANIFEST_BYTES: u64 = 64 * 1024;
pub const MAX_PACKAGE_BYTES: u64 = 32 * 1024 * 1024;
pub const MAX_ICON_BYTES: u64 = 512 * 1024;
pub const MAX_WASM_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ModuleKind {
    Declarative,
    Wasm,
}

impl fmt::Display for ModuleKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Declarative => formatter.write_str("declarative"),
            Self::Wasm => formatter.write_str("wasm"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Permissions {
    pub network: Vec<String>,
    pub cache: bool,
    pub clipboard: bool,
    pub notifications: bool,
    pub commands: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderManifest {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub triggers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleManifest {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: Version,
    pub api_version: VersionReq,
    pub license: String,
    pub source: String,
    #[serde(default)]
    pub homepage: Option<String>,
    pub icon: PathBuf,
    pub kind: ModuleKind,
    #[serde(default)]
    pub permissions: Permissions,
    pub providers: Vec<ProviderManifest>,
}

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("failed to read {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("module manifest is larger than {MAX_MANIFEST_BYTES} bytes")]
    TooLarge,
    #[error("invalid TOML: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("invalid module manifest: {0}")]
    Invalid(String),
}

impl ModuleManifest {
    pub fn load(path: &Path, allow_official: bool) -> Result<Self, ManifestError> {
        let metadata = fs::metadata(path).map_err(|source| ManifestError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        if metadata.len() > MAX_MANIFEST_BYTES {
            return Err(ManifestError::TooLarge);
        }
        let contents = fs::read_to_string(path).map_err(|source| ManifestError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let manifest: Self = toml::from_str(&contents)?;
        manifest.validate(allow_official)?;
        Ok(manifest)
    }

    pub fn validate(&self, allow_official: bool) -> Result<(), ManifestError> {
        validate_module_id(&self.id)?;
        if !allow_official
            && (self.id.starts_with("rayslash.") || self.author.eq_ignore_ascii_case("rayslash"))
        {
            return Err(ManifestError::Invalid(
                "the rayslash namespace and author are reserved".into(),
            ));
        }
        validate_text("name", &self.name, 1, 80)?;
        validate_text("description", &self.description, 1, 200)?;
        validate_text("author", &self.author, 1, 80)?;
        validate_text("license", &self.license, 1, 80)?;
        if !self.source.starts_with("https://github.com/") || self.source.contains(['?', '#']) {
            return Err(ManifestError::Invalid(
                "source must be a canonical HTTPS GitHub repository URL".into(),
            ));
        }
        validate_relative_path("icon", &self.icon)?;
        let supported = Version::parse(SUPPORTED_API).expect("valid SDK API version");
        if !self.api_version.matches(&supported) {
            return Err(ManifestError::Invalid(format!(
                "api_version {} does not include supported API {SUPPORTED_API}",
                self.api_version
            )));
        }
        if self.providers.is_empty() {
            return Err(ManifestError::Invalid(
                "at least one provider is required".into(),
            ));
        }
        for provider in &self.providers {
            validate_provider_id(&provider.id)?;
            validate_text("provider name", &provider.name, 1, 80)?;
            validate_text("provider description", &provider.description, 1, 200)?;
            for trigger in &provider.triggers {
                validate_text("trigger", trigger, 1, 40)?;
                if trigger.chars().any(char::is_whitespace) {
                    return Err(ManifestError::Invalid(format!(
                        "trigger {trigger:?} must not contain whitespace"
                    )));
                }
            }
        }
        for origin in &self.permissions.network {
            if !is_https_origin(origin) {
                return Err(ManifestError::Invalid(format!(
                    "network permission {origin:?} must be an HTTPS origin without path/query/fragment"
                )));
            }
        }
        Ok(())
    }
}

pub fn validate_module_id(id: &str) -> Result<(), ManifestError> {
    if id.len() > 128 || id.split('.').count() < 2 || id.split('.').any(|part| !valid_id_part(part))
    {
        return Err(ManifestError::Invalid(format!(
            "module id {id:?} must be dot-separated lowercase ASCII identifiers"
        )));
    }
    Ok(())
}

fn validate_provider_id(id: &str) -> Result<(), ManifestError> {
    if id.len() > 64 || !valid_id_part(id) {
        return Err(ManifestError::Invalid(format!(
            "provider id {id:?} must be a lowercase ASCII identifier"
        )));
    }
    Ok(())
}

fn valid_id_part(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 63
        && !value.starts_with('-')
        && !value.ends_with('-')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn validate_text(field: &str, value: &str, min: usize, max: usize) -> Result<(), ManifestError> {
    let length = value.chars().count();
    if length < min || length > max || value.trim() != value || value.chars().any(char::is_control)
    {
        return Err(ManifestError::Invalid(format!(
            "{field} must be trimmed printable text between {min} and {max} characters"
        )));
    }
    Ok(())
}

fn validate_relative_path(field: &str, path: &Path) -> Result<(), ManifestError> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(ManifestError::Invalid(format!(
            "{field} must be a safe relative package path"
        )));
    }
    Ok(())
}

fn is_https_origin(value: &str) -> bool {
    let Some(authority) = value.strip_prefix("https://") else {
        return false;
    };
    !authority.is_empty() && !authority.contains(['/', '?', '#']) && !authority.contains('@')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> ModuleManifest {
        toml::from_str(
            r#"
id = "io.github.example.docs"
name = "Docs"
description = "Search documentation."
author = "example"
version = "1.0.0"
api_version = "^1.0"
license = "MIT"
source = "https://github.com/example/docs"
icon = "icon.svg"
kind = "declarative"

[permissions]
network = ["https://docs.example.com"]

[[providers]]
id = "docs"
name = "Docs"
description = "Search documentation."
triggers = ["docs"]
"#,
        )
        .expect("manifest fixture")
    }

    #[test]
    fn accepts_valid_community_manifest() {
        manifest().validate(false).unwrap();
    }

    #[test]
    fn rejects_reserved_identity() {
        let mut value = manifest();
        value.author = "rayslash".into();
        assert!(value.validate(false).is_err());
        assert!(value.validate(true).is_ok());
    }

    #[test]
    fn rejects_paths_and_non_origin_permissions() {
        let mut value = manifest();
        value.icon = "../icon.svg".into();
        assert!(value.validate(false).is_err());
        value.icon = "icon.svg".into();
        value.permissions.network = vec!["https://example.com/path".into()];
        assert!(value.validate(false).is_err());
    }
}
