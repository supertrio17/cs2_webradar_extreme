use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::Serialize;
use serde_json::{json, Value};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RefreshError {
    #[error("io error at {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("schema json parse error at {path}: {source}")]
    SchemaParse {
        path: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
pub struct RefreshReport {
    pub output_dir: PathBuf,
    pub offsets_count: usize,
    pub schema_classes: usize,
}

#[derive(Debug, Clone, Serialize)]
struct OutputInfo<'a> {
    source: &'a str,
    build_number: u32,
    generated_at: String,
    dumper_version: &'a str,
}

fn read_binary(path: &Path) -> Result<Vec<u8>, RefreshError> {
    fs::read(path).map_err(|source| RefreshError::Io {
        path: path.display().to_string(),
        source,
    })
}

fn read_schema(path: &Path) -> Result<Value, RefreshError> {
    let raw = fs::read_to_string(path).map_err(|source| RefreshError::Io {
        path: path.display().to_string(),
        source,
    })?;

    serde_json::from_str(&raw).map_err(|source| RefreshError::SchemaParse {
        path: path.display().to_string(),
        source,
    })
}

fn parse_pattern(pattern: &str) -> Vec<Option<u8>> {
    pattern
        .split_whitespace()
        .map(|part| match part {
            "??" => None,
            _ => u8::from_str_radix(part, 16).ok(),
        })
        .collect()
}

fn scan_pattern(binary: &[u8], pattern: &[Option<u8>]) -> Option<usize> {
    if pattern.is_empty() || binary.len() < pattern.len() {
        return None;
    }

    binary.windows(pattern.len()).position(|window| {
        pattern.iter().zip(window.iter()).all(|(pat, byte)| match pat {
            Some(expected) => expected == byte,
            None => true,
        })
    })
}

fn extract_offsets(binary: Option<&[u8]>) -> BTreeMap<String, u64> {
    let mut offsets = BTreeMap::new();
    let signatures = [
        ("dwGameEntitySystem", "48 8B 0D ?? ?? ?? ?? 48 85 C9"),
        ("dwViewMatrix", "48 8D 0D ?? ?? ?? ?? 48 C1 E0 06"),
        ("dwLocalPlayerPawn", "48 89 05 ?? ?? ?? ?? 48 8D 85"),
        (
            "dwPlantedC4",
            "48 8B 15 ?? ?? ?? ?? 4C 8B 0D ?? ?? ?? ??",
        ),
    ];

    match binary {
        Some(data) => {
            for (name, signature) in signatures {
                let pattern = parse_pattern(signature);
                if let Some(index) = scan_pattern(data, &pattern) {
                    offsets.insert(name.to_string(), index as u64);
                }
            }
        }
        None => {
            for (name, _) in signatures {
                offsets.insert(name.to_string(), 0);
            }
        }
    }

    offsets
}

fn normalize_schema(schema: Option<Value>) -> Value {
    let Some(schema) = schema else {
        return json!({ "classes": [], "source": "minimal" });
    };

    let classes = schema
        .get("classes")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|class| {
            let name = class
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string();
            let fields = class
                .get("fields")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|f| {
                    json!({
                        "name": f.get("name").and_then(Value::as_str).unwrap_or("unknown"),
                        "type": f.get("type").and_then(Value::as_str).unwrap_or("unknown"),
                        "offset": f.get("offset").and_then(Value::as_u64).unwrap_or(0)
                    })
                })
                .collect::<Vec<_>>();

            json!({ "name": name, "fields": fields })
        })
        .collect::<Vec<_>>();

    json!({ "classes": classes, "source": "normalized" })
}

pub fn refresh_from_files(
    output_dir: &Path,
    binary_path: Option<&Path>,
    schema_path: Option<&Path>,
    build_number: u32,
) -> Result<RefreshReport, RefreshError> {
    fs::create_dir_all(output_dir).map_err(|source| RefreshError::Io {
        path: output_dir.display().to_string(),
        source,
    })?;

    let binary = match binary_path {
        Some(path) => Some(read_binary(path)?),
        None => None,
    };

    let raw_schema = match schema_path {
        Some(path) => Some(read_schema(path)?),
        None => None,
    };

    let offsets = extract_offsets(binary.as_deref());
    let schema = normalize_schema(raw_schema);

    let info = OutputInfo {
        source: "dump_refresh",
        build_number,
        generated_at: Utc::now().to_rfc3339(),
        dumper_version: "a2x-port-minimal-1",
    };

    let info_json = serde_json::to_string_pretty(&info)?;
    let offsets_json = serde_json::to_string_pretty(&offsets)?;
    let schema_json = serde_json::to_string_pretty(&schema)?;

    fs::write(output_dir.join("info.json"), info_json).map_err(|source| RefreshError::Io {
        path: output_dir.join("info.json").display().to_string(),
        source,
    })?;
    fs::write(output_dir.join("offsets.json"), offsets_json).map_err(|source| RefreshError::Io {
        path: output_dir.join("offsets.json").display().to_string(),
        source,
    })?;
    fs::write(output_dir.join("client_dll.json"), schema_json).map_err(|source| RefreshError::Io {
        path: output_dir.join("client_dll.json").display().to_string(),
        source,
    })?;

    let schema_classes = schema
        .get("classes")
        .and_then(Value::as_array)
        .map_or(0, |classes| classes.len());

    Ok(RefreshReport {
        output_dir: output_dir.to_path_buf(),
        offsets_count: offsets.len(),
        schema_classes,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn extracts_signature_offsets_from_binary() {
        let signature = [0x48, 0x8B, 0x0D, 0xAA, 0xBB, 0xCC, 0xDD, 0x48, 0x85, 0xC9];
        let mut binary = vec![0x90; 200];
        binary[50..60].copy_from_slice(&signature);

        let offsets = extract_offsets(Some(&binary));
        assert_eq!(offsets.get("dwGameEntitySystem"), Some(&50));
    }

    #[test]
    fn refresh_writes_expected_pack_files() {
        let dir = tempfile::tempdir().expect("tempdir");
        let out = dir.path().join("pack");

        let report = refresh_from_files(&out, None, None, 15000).expect("refresh");

        assert!(out.join("info.json").exists());
        assert!(out.join("offsets.json").exists());
        assert!(out.join("client_dll.json").exists());
        assert_eq!(report.offsets_count, 4);

        let offsets_text = fs::read_to_string(out.join("offsets.json")).expect("offsets read");
        assert!(offsets_text.contains("dwGameEntitySystem"));
    }
}
