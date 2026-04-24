use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct DumpRuntimeConfig {
    pub app_id: String,
    pub embedded_dir: PathBuf,
    pub user_override_dir: Option<PathBuf>,
}

impl DumpRuntimeConfig {
    pub fn user_dump_dir(&self) -> PathBuf {
        if let Some(path) = &self.user_override_dir {
            return path.clone();
        }

        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(&self.app_id)
            .join("dumps")
            .join("active")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DumpInfo {
    pub source: String,
    pub build_number: Option<u32>,
    pub generated_at: Option<String>,
    pub dumper_version: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DumpPack {
    pub root_dir: PathBuf,
    pub info: DumpInfo,
    pub offsets: Value,
    pub schema: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DumpSource {
    User,
    Embedded,
}

#[derive(Debug, Clone)]
pub struct ActiveDump {
    pub source: DumpSource,
    pub pack: DumpPack,
    pub warnings: Vec<String>,
}

#[derive(Debug, Error)]
pub enum DumpError {
    #[error("dump file missing: {0}")]
    MissingFile(String),
    #[error("invalid json in {path}: {source}")]
    InvalidJson {
        path: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("io error at {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("no usable dump pack found")]
    NoUsableDump,
}

fn read_json(path: &Path) -> Result<Value, DumpError> {
    let raw = fs::read_to_string(path).map_err(|source| DumpError::Io {
        path: path.display().to_string(),
        source,
    })?;

    serde_json::from_str::<Value>(&raw).map_err(|source| DumpError::InvalidJson {
        path: path.display().to_string(),
        source,
    })
}

fn parse_info(value: &Value) -> DumpInfo {
    let source = value
        .get("source")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();

    let build_number = value
        .get("build_number")
        .and_then(Value::as_u64)
        .or_else(|| value.get("buildNumber").and_then(Value::as_u64))
        .and_then(|n| u32::try_from(n).ok());

    let generated_at = value
        .get("generated_at")
        .and_then(Value::as_str)
        .or_else(|| value.get("generatedAt").and_then(Value::as_str))
        .map(ToOwned::to_owned);

    let dumper_version = value
        .get("dumper_version")
        .and_then(Value::as_str)
        .or_else(|| value.get("dumperVersion").and_then(Value::as_str))
        .map(ToOwned::to_owned);

    DumpInfo {
        source,
        build_number,
        generated_at,
        dumper_version,
    }
}

pub fn load_dump_pack(dir: &Path) -> Result<DumpPack, DumpError> {
    let info_path = dir.join("info.json");
    let offsets_path = dir.join("offsets.json");
    let schema_path = dir.join("client_dll.json");

    for file in [&info_path, &offsets_path, &schema_path] {
        if !file.exists() {
            return Err(DumpError::MissingFile(file.display().to_string()));
        }
    }

    let info_value = read_json(&info_path)?;
    let offsets = read_json(&offsets_path)?;
    let schema = read_json(&schema_path)?;

    Ok(DumpPack {
        root_dir: dir.to_path_buf(),
        info: parse_info(&info_value),
        offsets,
        schema,
    })
}

pub fn resolve_active_dump(
    config: &DumpRuntimeConfig,
    expected_build_number: Option<u32>,
) -> Result<ActiveDump, DumpError> {
    let user_dir = config.user_dump_dir();
    let embedded_dir = config.embedded_dir.clone();

    let mut warnings = Vec::new();
    let user_pack = load_dump_pack(&user_dir).ok();
    let embedded_pack = load_dump_pack(&embedded_dir).ok();

    let pick = |pack: &DumpPack, expected: Option<u32>| -> bool {
        match expected {
            Some(expected_build) => pack.info.build_number == Some(expected_build),
            None => true,
        }
    };

    if let Some(pack) = user_pack {
        if pick(&pack, expected_build_number) {
            return Ok(ActiveDump {
                source: DumpSource::User,
                pack,
                warnings,
            });
        }

        warnings.push(format!(
            "user dump build mismatch: expected {:?}, got {:?}",
            expected_build_number, pack.info.build_number
        ));
    }

    if let Some(pack) = embedded_pack {
        if pick(&pack, expected_build_number) || expected_build_number.is_none() {
            return Ok(ActiveDump {
                source: DumpSource::Embedded,
                pack,
                warnings,
            });
        }

        warnings.push(format!(
            "embedded dump build mismatch: expected {:?}, got {:?}",
            expected_build_number, pack.info.build_number
        ));
    }

    Err(DumpError::NoUsableDump)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn write_pack(path: &Path, build: u32, source: &str) {
        fs::create_dir_all(path).expect("create dir");
        fs::write(
            path.join("info.json"),
            format!(
                "{{\"source\":\"{}\",\"build_number\":{},\"generated_at\":\"2026-01-01T00:00:00Z\"}}",
                source, build
            ),
        )
        .expect("info");
        fs::write(path.join("offsets.json"), "{\"dwGameEntitySystem\": 123}").expect("offsets");
        fs::write(path.join("client_dll.json"), "{\"classes\": []}").expect("schema");
    }

    #[test]
    fn prefers_user_pack_when_build_matches() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let user = tmp.path().join("user");
        let embedded = tmp.path().join("embedded");
        write_pack(&user, 15000, "user");
        write_pack(&embedded, 14000, "embedded");

        let config = DumpRuntimeConfig {
            app_id: "x".to_string(),
            embedded_dir: embedded,
            user_override_dir: Some(user),
        };

        let resolved = resolve_active_dump(&config, Some(15000)).expect("resolve");
        assert_eq!(resolved.source, DumpSource::User);
    }

    #[test]
    fn falls_back_to_embedded_when_user_mismatches() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let user = tmp.path().join("user");
        let embedded = tmp.path().join("embedded");
        write_pack(&user, 15000, "user");
        write_pack(&embedded, 16000, "embedded");

        let config = DumpRuntimeConfig {
            app_id: "x".to_string(),
            embedded_dir: embedded,
            user_override_dir: Some(user),
        };

        let resolved = resolve_active_dump(&config, Some(16000)).expect("resolve");
        assert_eq!(resolved.source, DumpSource::Embedded);
        assert_eq!(resolved.warnings.len(), 1);
    }

    #[test]
    fn errors_when_no_pack_matches() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let user = tmp.path().join("user");
        let embedded = tmp.path().join("embedded");
        write_pack(&user, 15000, "user");
        write_pack(&embedded, 16000, "embedded");

        let config = DumpRuntimeConfig {
            app_id: "x".to_string(),
            embedded_dir: embedded,
            user_override_dir: Some(user),
        };

        let error = resolve_active_dump(&config, Some(17000)).expect_err("should fail");
        assert!(matches!(error, DumpError::NoUsableDump));
    }
}
