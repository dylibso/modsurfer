#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
#[cfg(feature = "api")]
use chrono::Utc;
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
#[cfg(feature = "api")]
use modsurfer_module::{Export, Import, SourceLanguage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Order {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pagination {
    pub offset: u32,
    pub limit: u32,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            offset: 0,
            limit: 10,
        }
    }
}

impl<'a> From<&'a modsurfer_proto_v1::api::Pagination> for Pagination {
    fn from(page: &'a modsurfer_proto_v1::api::Pagination) -> Self {
        Pagination {
            offset: page.offset,
            limit: page.limit,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortField {
    CreatedAt,
    Name,
    Size,
    Language,
    ImportsCount,
    ExportsCount,
    Sha256,
    Complexity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sort {
    pub order: Order,
    pub field: SortField,
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
#[cfg(feature = "api")]
#[derive(Default)]
pub struct Search {
    pub page: Pagination,
    pub hash: Option<String>,
    pub location: Option<String>,
    pub imports: Vec<Import>,
    pub exports: Vec<Export>,
    pub function_name: Option<String>,
    pub module_name: Option<String>,
    pub source_language: Option<SourceLanguage>,
    pub inserted_after: Option<chrono::DateTime<Utc>>,
    pub inserted_before: Option<chrono::DateTime<Utc>>,
    pub strings: Option<Vec<String>>,
    pub sort: Option<Sort>,
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
#[cfg(feature = "api")]
#[derive(Debug, Clone)]
pub enum AuditOutcome {
    Pass,
    Fail,
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
#[cfg(feature = "api")]
impl std::fmt::Display for AuditOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditOutcome::Pass => f.write_str("pass"),
            _ => f.write_str("fail"),
        }
    }
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
#[cfg(feature = "api")]
impl From<&std::ffi::OsStr> for AuditOutcome {
    fn from(value: &std::ffi::OsStr) -> Self {
        value.to_str().unwrap_or_else(|| "fail").to_string().into()
    }
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
#[cfg(feature = "api")]
impl From<String> for AuditOutcome {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "pass" => AuditOutcome::Pass,
            _ => AuditOutcome::Fail,
        }
    }
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
#[cfg(feature = "api")]
#[derive(Debug)]
pub struct Audit {
    pub page: Pagination,
    pub outcome: AuditOutcome,
    pub checkfile: Vec<u8>,
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
#[cfg(feature = "api")]
impl Default for AuditOutcome {
    fn default() -> Self {
        AuditOutcome::Fail
    }
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
#[cfg(feature = "api")]
impl From<modsurfer_proto_v1::api::AuditOutcome> for AuditOutcome {
    fn from(outcome: modsurfer_proto_v1::api::AuditOutcome) -> AuditOutcome {
        match outcome {
            modsurfer_proto_v1::api::AuditOutcome::PASS => AuditOutcome::Pass,
            modsurfer_proto_v1::api::AuditOutcome::FAIL => AuditOutcome::Fail,
        }
    }
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
#[cfg(feature = "api")]
impl From<AuditOutcome> for modsurfer_proto_v1::api::AuditOutcome {
    fn from(outcome: AuditOutcome) -> Self {
        match outcome {
            AuditOutcome::Pass => modsurfer_proto_v1::api::AuditOutcome::PASS,
            AuditOutcome::Fail => modsurfer_proto_v1::api::AuditOutcome::FAIL,
        }
    }
}
