use std::{collections::BTreeMap, fmt::Display, path::PathBuf, process::ExitCode};

use anyhow::Result;
use comfy_table::{modifiers::UTF8_SOLID_INNER_BORDERS, presets::UTF8_FULL, Row, Table};
use extism::{Context, Plugin};
use human_bytes::human_bytes;
use modsurfer_convert::from_api;
use parse_size::parse_size;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Debug, Deserialize, Default, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Validation {
    pub validate: Check,
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Default, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Check {
    pub url: Option<String>,
    pub allow_wasi: Option<bool>,
    pub imports: Option<Imports>,
    pub exports: Option<Exports>,
    pub size: Option<Size>,
    pub complexity: Option<Complexity>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub enum RiskLevel {
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

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Complexity {
    pub max_risk: Option<RiskLevel>,
    pub max_score: Option<u32>,
}

#[allow(unused)]
pub enum ComplexityKind {
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

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum NamespaceItem {
    Name(String),
    Item {
        name: String,
        #[serde(default)]
        functions: Vec<FunctionItem>,
    },
}

impl NamespaceItem {
    fn name(&self) -> &String {
        match self {
            NamespaceItem::Name(name) => name,
            NamespaceItem::Item { name, .. } => name,
        }
    }

    fn functions(&self) -> &[FunctionItem] {
        match self {
            NamespaceItem::Name(_) => &[],
            NamespaceItem::Item { functions, .. } => functions,
        }
    }
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[serde(deny_unknown_fields)]
pub enum ImportItem {
    Name(String),
    Item {
        namespace: Option<String>,
        name: String,
        params: Option<Vec<modsurfer_module::ValType>>,
        results: Option<Vec<modsurfer_module::ValType>>,
    },
}

impl ImportItem {
    fn name(&self) -> &String {
        match self {
            ImportItem::Name(name) => name,
            ImportItem::Item { name, .. } => name,
        }
    }

    fn namespace(&self) -> Option<&str> {
        match self {
            ImportItem::Name(_) => None,
            ImportItem::Item { namespace, .. } => namespace.as_deref(),
        }
    }

    fn results(&self) -> Option<&[modsurfer_module::ValType]> {
        match self {
            ImportItem::Name(_) => None,
            ImportItem::Item { results, .. } => results.as_deref(),
        }
    }

    fn params(&self) -> Option<&[modsurfer_module::ValType]> {
        match self {
            ImportItem::Name(_) => None,
            ImportItem::Item { params, .. } => params.as_deref(),
        }
    }
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[serde(deny_unknown_fields)]
pub enum FunctionItem {
    Name(String),
    Item {
        name: String,
        params: Option<Vec<modsurfer_module::ValType>>,
        results: Option<Vec<modsurfer_module::ValType>>,
    },
}

impl FunctionItem {
    fn name(&self) -> &String {
        match self {
            FunctionItem::Name(name) => name,
            FunctionItem::Item { name, .. } => name,
        }
    }

    fn results(&self) -> Option<&[modsurfer_module::ValType]> {
        match self {
            FunctionItem::Name(_) => None,
            FunctionItem::Item { results, .. } => results.as_deref(),
        }
    }

    fn params(&self) -> Option<&[modsurfer_module::ValType]> {
        match self {
            FunctionItem::Name(_) => None,
            FunctionItem::Item { params, .. } => params.as_deref(),
        }
    }
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Namespace {
    pub include: Option<Vec<NamespaceItem>>,
    pub exclude: Option<Vec<NamespaceItem>>,
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Imports {
    pub include: Option<Vec<ImportItem>>,
    pub exclude: Option<Vec<ImportItem>>,
    pub namespace: Option<Namespace>,
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Exports {
    pub include: Option<Vec<FunctionItem>>,
    pub exclude: Option<Vec<FunctionItem>>,
    pub max: Option<u32>,
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Size {
    pub max: Option<String>,
}

#[derive(Debug, Serialize)]
pub enum Classification {
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

    fn validate_fn_type(
        &mut self,
        name: &str,
        actual: &modsurfer_module::FunctionType,
        params: Option<&[modsurfer_module::ValType]>,
        results: Option<&[modsurfer_module::ValType]>,
    ) {
        if let Some(params) = params {
            let test_params = actual.args == params;
            self.validate_fn(
                &format!("{name}.params"),
                format!("{:?}", params),
                format!("{:?}", actual.args),
                test_params,
                8,
                Classification::AbiCompatibilty,
            );
        };

        if let Some(results) = results {
            let test_results = actual.returns == results;
            self.validate_fn(
                &format!("{name}.results"),
                format!("{:?}", results),
                format!("{:?}", actual.returns),
                test_results,
                8,
                Classification::AbiCompatibilty,
            );
        };
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

pub struct Module {}

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
    pub fn parse(wasm: impl AsRef<[u8]>) -> Result<modsurfer_module::Module> {
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

fn namespace_prefix(import_item: &ImportItem, fn_name: &str) -> String {
    match import_item.namespace() {
        Some(ns) => format!("{}::{}", ns, fn_name),
        None => fn_name.into(),
    }
}

pub async fn validate(validation: Validation, module: modsurfer_module::Module) -> Result<Report> {
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
        let import_module_func_types = module
            .imports
            .iter()
            .map(|im| {
                (
                    (im.module_name.as_str(), im.func.name.as_str()),
                    &im.func.ty,
                )
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        let import_func_types = import_module_func_types
            .iter()
            .map(|((_, k), ty)| (*k, ty.clone()))
            .collect::<BTreeMap<_, _>>();

        let import_module_names = module.get_import_namespaces();

        if let Some(include) = imports.include {
            include.iter().for_each(|imp| {
                let name = imp.name();
                let test = if let Some(ns) = imp.namespace() {
                    import_module_func_types.contains_key(&(ns, name))
                } else {
                    import_func_types.contains_key(&name.as_str())
                };

                let ty = if let Some(ns) = imp.namespace() {
                    import_module_func_types.get(&(ns, name))
                } else {
                    import_func_types.get(name.as_str())
                };

                if test {
                    let ty = ty.unwrap();
                    report.validate_fn_type(
                        &format!("imports.include.{}", namespace_prefix(&imp, name)),
                        *ty,
                        imp.params(),
                        imp.results(),
                    );
                }

                report.validate_fn(
                    &format!("imports.include.{}", namespace_prefix(&imp, name)),
                    Exist(true).to_string(),
                    Exist(test).to_string(),
                    test,
                    8,
                    Classification::AbiCompatibilty,
                );
            });
        }

        if let Some(exclude) = imports.exclude {
            exclude.iter().for_each(|imp| {
                let name = imp.name();
                let test = if let Some(ns) = imp.namespace() {
                    import_module_func_types.contains_key(&(ns, name))
                } else {
                    import_func_types.contains_key(&name.as_str())
                };

                let ty = if let Some(ns) = imp.namespace() {
                    import_module_func_types.get(&(ns, name))
                } else {
                    import_func_types.get(name.as_str())
                };

                if test {
                    let ty = ty.unwrap();
                    report.validate_fn_type(
                        &format!("imports.exclude.{}", namespace_prefix(&imp, name)),
                        *ty,
                        imp.params(),
                        imp.results(),
                    );
                };

                report.validate_fn(
                    &format!("imports.exclude.{}", namespace_prefix(&imp, name)),
                    Exist(false).to_string(),
                    Exist(test).to_string(),
                    !test,
                    5,
                    Classification::AbiCompatibilty,
                );
            });
        }

        if let Some(namespace) = imports.namespace {
            if let Some(include) = namespace.include {
                include.iter().for_each(|ns| {
                    let name = ns.name();
                    let functions = ns.functions();
                    let test = import_module_names.contains(&name.as_str());
                    report.validate_fn(
                        &format!("imports.namespace.include.{}", name),
                        Exist(true).to_string(),
                        Exist(test).to_string(),
                        test,
                        8,
                        Classification::AbiCompatibilty,
                    );

                    for f in functions.iter() {
                        let test =
                            import_module_func_types.contains_key(&(name, f.name().as_str()));
                        report.validate_fn(
                            &format!("imports.namespace.include.{name}::{}", f.name()),
                            Exist(true).to_string(),
                            Exist(test).to_string(),
                            test,
                            8,
                            Classification::AbiCompatibilty,
                        );

                        if test {
                            let ty = import_module_func_types
                                .get(&(name, f.name().as_str()))
                                .unwrap();
                            report.validate_fn_type(
                                &format!("imports.namespace.include.{name}::{}", f.name()),
                                *ty,
                                f.params(),
                                f.results(),
                            );
                        }
                    }
                });
            }

            if let Some(exclude) = namespace.exclude {
                exclude.iter().for_each(|ns| {
                    let name = ns.name();
                    let functions = ns.functions();
                    let test = import_module_names.contains(&name.as_str());

                    report.validate_fn(
                        &format!("imports.namespace.exclude.{}", name),
                        Exist(false).to_string(),
                        Exist(test).to_string(),
                        !test,
                        10,
                        Classification::AbiCompatibilty,
                    );

                    for f in functions.iter() {
                        let test =
                            import_module_func_types.contains_key(&(name, f.name().as_str()));

                        if test {
                            let ty = import_module_func_types
                                .get(&(name, f.name().as_str()))
                                .unwrap();

                            report.validate_fn_type(
                                &format!("imports.namespace.exclude.{name}::{}", f.name()),
                                *ty,
                                f.params(),
                                f.results(),
                            );
                        };

                        report.validate_fn(
                            &format!("imports.namespace.exclude.{name}::{}", f.name()),
                            Exist(false).to_string(),
                            Exist(test).to_string(),
                            !test,
                            10,
                            Classification::AbiCompatibilty,
                        );
                    }
                });
            }
        }
    }

    // Exports
    if let Some(exports) = validation.validate.exports {
        let export_func_types = module
            .exports
            .iter()
            .map(|im| (im.func.name.as_str(), &im.func.ty))
            .collect::<std::collections::BTreeMap<_, _>>();

        if let Some(max) = exports.max {
            let num = export_func_types.len() as u32;
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
            include.iter().for_each(|f| {
                let name = f.name();
                let test = export_func_types.contains_key(name.as_str());
                report.validate_fn(
                    &format!("exports.include.{}", name),
                    Exist(true).to_string(),
                    Exist(test).to_string(),
                    test,
                    10,
                    Classification::AbiCompatibilty,
                );

                if test {
                    let ty = export_func_types.get(name.as_str()).unwrap();
                    report.validate_fn_type(
                        &format!("exports.include.{}", name),
                        *ty,
                        f.params(),
                        f.results(),
                    );
                }
            });
        }

        if let Some(exclude) = exports.exclude {
            exclude.iter().for_each(|f| {
                let name = f.name();

                let ty = export_func_types.get(name.as_str());
                let test = ty.is_some();
                if test {
                    let ty = ty.unwrap();
                    report.validate_fn_type(
                        &format!("exports.include.{}", name),
                        *ty,
                        f.params(),
                        f.results(),
                    );
                }

                report.validate_fn(
                    &format!("exports.exclude.{}", name),
                    Exist(false).to_string(),
                    Exist(test).to_string(),
                    !test,
                    5,
                    Classification::AbiCompatibilty,
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

    validate(validation, module).await
}
