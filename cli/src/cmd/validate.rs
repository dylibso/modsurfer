use std::{collections::BTreeMap, fmt::Display, path::PathBuf, process::ExitCode};

use anyhow::Result;
use comfy_table::{modifiers::UTF8_SOLID_INNER_BORDERS, presets::UTF8_FULL, Row, Table};
use extism::{Context, Plugin};
use human_bytes::human_bytes;
use modsurfer_convert::from_api;
use parse_size::parse_size;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Validation {
    pub validate: Check,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Check {
    pub url: Option<String>,
    pub allow_wasi: Option<bool>,
    pub imports: Option<Imports>,
    pub exports: Option<Exports>,
    pub size: Option<Size>,
    pub complexity: Option<Complexity>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
enum RiskLevel {
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "high")]
    High,
}

/// The output of the "Cyclomatic Complexity" algorithm run on a graph analysis of the WebAssembly
/// code inside the provided module. The risk is purely related to computational resource usage,
/// not code security or any other interpretation of risk.
impl RiskLevel {
    fn max(&self) -> u32 {
        match self {
            RiskLevel::Low => std::env::var("MODSURFER_RISK_LOW")
                .unwrap_or(2500.to_string())
                .parse::<u32>()
                .expect("valid low risk level setting"),
            RiskLevel::Medium => std::env::var("MODSURFER_RISK_MEDIUM")
                .unwrap_or(50000.to_string())
                .parse::<u32>()
                .expect("valid medium risk level setting"),
            RiskLevel::High => std::env::var("MODSURFER_RISK_HIGH")
                .unwrap_or(u32::MAX.to_string())
                .parse::<u32>()
                .expect("valid high risk level setting"),
        }
    }
}

impl From<u32> for RiskLevel {
    fn from(value: u32) -> Self {
        if value <= RiskLevel::Low.max() {
            RiskLevel::Low
        } else if value <= RiskLevel::Medium.max() {
            RiskLevel::Medium
        } else {
            RiskLevel::High
        }
    }
}

impl Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            RiskLevel::Low => "low",
            RiskLevel::Medium => "medium",
            RiskLevel::High => "high",
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Complexity {
    pub max_risk: Option<RiskLevel>,
    pub max_score: Option<u32>,
}

#[allow(unused)]
enum ComplexityKind {
    MaxRisk(RiskLevel),
    MaxScore(u32),
}

impl Complexity {
    fn kind(&self) -> Result<ComplexityKind> {
        match (self.max_risk.clone(), self.max_score) {
            (None, None) => anyhow::bail!("No complexity check found."),
            (None, Some(_score)) => {
                anyhow::bail!("Only `complexity.max_risk` is currently supported.")
            }
            (Some(risk), None) => Ok(ComplexityKind::MaxRisk(risk)),
            (Some(_), Some(_)) => {
                anyhow::bail!("Only `complexity.max_risk` is currently supported.")
            }
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Namespace {
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Imports {
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub namespace: Option<Namespace>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Exports {
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub max: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Size {
    pub max: Option<String>,
}

#[derive(Debug, Serialize)]
enum Classification {
    AbiCompatibilty,
    ResourceLimit,
    Security,
}

impl Display for Classification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Classification::AbiCompatibilty => f.write_str("ABI Compatibility")?,
            Classification::ResourceLimit => f.write_str("Resource Limit")?,
            Classification::Security => f.write_str("Security")?,
        }
        Ok(())
    }
}

#[derive(Debug, Serialize)]
struct FailureDetail {
    actual: String,
    expected: String,
    severity: usize,
    classification: Classification,
}

#[derive(Debug, Serialize)]
pub struct Report {
    /// k/v pair of the dot-separated path to validation field and expectation info
    fails: BTreeMap<String, FailureDetail>,
}

impl Report {
    pub fn as_exit_code(&self) -> ExitCode {
        match self.fails.len() {
            0 => ExitCode::SUCCESS,
            _ => ExitCode::FAILURE,
        }
    }

    pub fn has_failures(&self) -> bool {
        !self.fails.is_empty()
    }
}

impl Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.fails.is_empty() {
            return Ok(());
        }

        let mut table = Table::new();
        table.load_preset(UTF8_FULL);
        table.apply_modifier(UTF8_SOLID_INNER_BORDERS);
        table.set_header(vec![
            "Status",
            "Property",
            "Expected",
            "Actual",
            "Classification",
            "Severity",
        ]);

        self.fails.iter().for_each(|fail| {
            const SEVERITY_MAX: usize = 10;
            let severity = if fail.1.severity <= SEVERITY_MAX {
                fail.1.severity
            } else {
                SEVERITY_MAX
            };

            table.add_row(Row::from(vec![
                "FAIL",
                fail.0.as_str(),
                fail.1.expected.as_str(),
                fail.1.actual.as_str(),
                fail.1.classification.to_string().as_str(),
                "|".repeat(severity).as_str(),
            ]));
        });

        f.write_str(table.to_string().as_str())
    }
}

impl Report {
    fn new() -> Self {
        Self {
            fails: Default::default(),
        }
    }

    fn validate_fn(
        &mut self,
        name: &str,
        expected: String,
        actual: String,
        valid: bool,
        severity: usize,
        classification: Classification,
    ) {
        if !valid {
            self.fails.insert(
                name.to_string(),
                FailureDetail {
                    actual,
                    expected,
                    severity,
                    classification,
                },
            );
        }
    }
}

struct Exist(bool);

impl Display for Exist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 {
            f.write_str("included")?;
        } else {
            f.write_str("excluded")?;
        }

        Ok(())
    }
}

struct Module {}

impl Module {
    // NOTE: this function executes WebAssembly code as a plugin managed by Extism (https://extism.org)
    // and is distributed under the same license as the primary codebase. See LICENSE file in the
    // root of this repository.
    //
    // The source code to the WebAssembly binary is not open source.
    //
    // Importantly, this code has no side-effects, and uses no system resources. The `false`
    // parameter provided to `Plugin::new` below, ensures that the module is run without functions
    // provided by the WASI module namespace. Therefore it has no access to your running environment
    // nor any system resources such as a filesystem or network.
    //
    // The function within the WebAssembly, "parse_module", only parses bytes provided to it from
    // the host context (the `wasm`), and collects parsed information into the `Module` which is
    // returned as a protobuf-encoded struct.
    fn parse(wasm: impl AsRef<[u8]>) -> Result<modsurfer_module::Module> {
        let ctx = Context::new();
        let mut plugin = Plugin::new(&ctx, crate::plugins::MODSURFER_WASM, [], false)?;
        let data = plugin.call("parse_module", wasm)?;
        let a: modsurfer_proto_v1::api::Module = protobuf::Message::parse_from_bytes(&data)?;
        let metadata = if a.metadata.is_empty() {
            None
        } else {
            Some(a.metadata)
        };

        let inserted_at: std::time::SystemTime = a
            .inserted_at
            .unwrap_or_else(|| protobuf::well_known_types::timestamp::Timestamp::new())
            .into();

        let module = modsurfer_module::Module {
            hash: a.hash,
            imports: from_api::imports(a.imports),
            exports: from_api::exports(a.exports),
            size: a.size as u64,
            location: a.location,
            source_language: from_api::source_language(a.source_language.enum_value_or_default()),
            metadata,
            inserted_at: inserted_at.into(),
            strings: a.strings,
            complexity: a.complexity,
            graph: a.graph,
        };

        Ok(module)
    }
}

pub async fn validate_module(file: &PathBuf, check: &PathBuf) -> Result<Report> {
    // read the wasm file and parse a Module from it to later validate against the check file.
    // NOTE: the Module is produced by executing plugin code, linked and called from the
    // `Module::parse` function.
    let module_data = tokio::fs::read(file).await?;
    let module = Module::parse(&module_data)?;

    let mut buf = tokio::fs::read(check).await?;

    let mut validation: Validation = serde_yaml::from_slice(&buf)?;
    if let Some(url) = validation.validate.url {
        // fetch remote validation file
        println!("Fetching validation schema from URL: {}", url);

        let resp = reqwest::get(&url).await?;
        if !resp.status().is_success() {
            anyhow::bail!(
                "Failed to make request for remote validation schema: {}",
                url
            );
        }

        buf.clear();
        buf = resp.bytes().await?.into();

        // parse the file again & reassign `validation`
        validation = serde_yaml::from_slice(&buf)?;
    }

    let mut report = Report::new();

    // WASI
    if let Some(allowed) = validation.validate.allow_wasi {
        let actual = module
            .get_import_namespaces()
            .contains(&"wasi_snapshot_preview1");
        report.validate_fn(
            "allow_wasi",
            allowed.to_string(),
            actual.to_string(),
            !(allowed == false && actual),
            10,
            Classification::AbiCompatibilty,
        );
    }

    // Imports
    if let Some(imports) = validation.validate.imports {
        let import_func_names = module
            .imports
            .iter()
            .map(|im| im.func.name.clone())
            .collect::<Vec<_>>();

        let import_module_names = module.get_import_namespaces();

        if let Some(include) = imports.include {
            include.iter().for_each(|name| {
                let test = import_func_names.contains(name);
                report.validate_fn(
                    &format!("imports.include.{}", name),
                    Exist(true).to_string(),
                    Exist(test).to_string(),
                    test,
                    8,
                    Classification::AbiCompatibilty,
                )
            });
        }

        if let Some(exclude) = imports.exclude {
            exclude.iter().for_each(|name| {
                let test = import_func_names.contains(name);
                report.validate_fn(
                    &format!("imports.exclude.{}", name),
                    Exist(false).to_string(),
                    Exist(test).to_string(),
                    test,
                    5,
                    Classification::AbiCompatibilty,
                );
            });
        }

        if let Some(namespace) = imports.namespace {
            if let Some(include) = namespace.include {
                include.iter().for_each(|name| {
                    let test = import_module_names.contains(&name.as_str());
                    report.validate_fn(
                        &format!("imports.namespace.include.{}", name),
                        Exist(true).to_string(),
                        Exist(test).to_string(),
                        test,
                        8,
                        Classification::AbiCompatibilty,
                    );
                });
            }

            if let Some(exclude) = namespace.exclude {
                exclude.iter().for_each(|name| {
                    let test = import_module_names.contains(&name.as_str());
                    report.validate_fn(
                        &format!("imports.namespace.exclude.{}", name),
                        Exist(false).to_string(),
                        Exist(test).to_string(),
                        !test,
                        10,
                        Classification::AbiCompatibilty,
                    )
                });
            }
        }
    }

    // Exports
    if let Some(exports) = validation.validate.exports {
        let export_func_names = module
            .exports
            .iter()
            .map(|exp| exp.func.name.clone())
            .collect::<Vec<_>>();
        if let Some(max) = exports.max {
            let num = export_func_names.len() as u32;
            let overage = num.saturating_sub(max);
            let max = if max == 0 { 1 } else { max };
            let severity = ((overage as f32 / max as f32) * 10.0).ceil() as usize;
            let test = num <= max;
            report.validate_fn(
                "exports.max",
                format!("<= {max}"),
                num.to_string(),
                test,
                severity,
                Classification::Security,
            );
        }

        if let Some(include) = exports.include {
            include.iter().for_each(|name| {
                let test = export_func_names.contains(name);
                report.validate_fn(
                    &format!("exports.include.{}", name),
                    Exist(true).to_string(),
                    Exist(test).to_string(),
                    test,
                    10,
                    Classification::AbiCompatibilty,
                );
            });
        }

        if let Some(exclude) = exports.exclude {
            exclude.iter().for_each(|name| {
                let test = export_func_names.contains(name);
                report.validate_fn(
                    &format!("exports.exclude.{}", name),
                    Exist(false).to_string(),
                    Exist(test).to_string(),
                    !test,
                    5,
                    Classification::Security,
                );
            });
        }
    }

    // Size
    if let Some(size) = validation.validate.size {
        if let Some(max) = size.max {
            let parsed = parse_size(&max).unwrap();
            let human_actual = human_bytes(module.size as f64);
            let test = module.size <= parsed;
            report.validate_fn(
                "size.max",
                format!("<= {max}"),
                human_actual.to_string(),
                test,
                (module.size / parsed) as usize,
                Classification::ResourceLimit,
            );
        }
    }

    // Complexity
    if let Some(complexity) = validation.validate.complexity {
        let module_complexity = module.complexity.ok_or_else(|| anyhow::anyhow!("Could not determine module complexity, please remove the complexity parameter from your checkfile."))?;
        match complexity.kind()? {
            ComplexityKind::MaxRisk(risk) => {
                report.validate_fn(
                    "complexity.max_risk",
                    format!("<= {}", risk),
                    RiskLevel::from(module_complexity).to_string(),
                    risk.max() >= module_complexity,
                    (module_complexity / risk.max()) as usize,
                    Classification::ResourceLimit,
                );
            }
            _ => unreachable!(),
        }
    }

    Ok(report)
}
