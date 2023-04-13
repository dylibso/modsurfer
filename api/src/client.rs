use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use modsurfer_convert::{
    api::{self, Direction, Field, Sort},
    to_api, Audit,
};
use modsurfer_module::{Export, Import, Module};
use protobuf::{self, EnumOrUnknown, Message, MessageField, SpecialFields};
use reqwest;
use url::Url;

use super::{ApiClient, List, Persisted};

enum ModserverCommand {
    CreateModule(api::CreateModuleRequest),
    GetModule(api::GetModuleRequest),
    ListModules(api::ListModulesRequest),
    SearchModules(api::SearchModulesRequest),
    DeleteModules(api::DeleteModulesRequest),
    AuditModules(api::AuditModulesRequest),
    DiffModules(api::DiffRequest),
}

#[derive(PartialEq, Clone, Debug)]
pub enum SortDirection {
    Asc,
    Desc,
}

impl SortDirection {
    pub fn default() -> SortDirection {
        SortDirection::Desc
    }

    pub fn from_str(f: &str) -> Option<SortDirection> {
        match f.to_lowercase().as_str() {
            "asc" => Some(SortDirection::Asc),
            "desc" => Some(SortDirection::Desc),
            _ => None,
        }
    }
    fn to_proto(self) -> Direction {
        match self {
            SortDirection::Asc => Direction::Asc,
            SortDirection::Desc => Direction::Desc,
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum SortField {
    Size,
    Name,
    CreatedAt,
    Language,
    ImportsCount,
    ExportsCount,
    Sha256,
    Complexity,
}

impl SortField {
    pub fn from_str(f: &str) -> Option<SortField> {
        match f.to_lowercase().as_str() {
            "size" => Some(SortField::Size),
            "name" => Some(SortField::Name),
            "created_at" => Some(SortField::CreatedAt),
            "language" => Some(SortField::Language),
            "imports_count" => Some(SortField::ImportsCount),
            "exports_count" => Some(SortField::ExportsCount),
            "sha256" => Some(SortField::Sha256),
            "complexity" => Some(SortField::Complexity),
            _ => None,
        }
    }

    fn to_proto(self) -> Field {
        match self {
            SortField::Size => Field::Size,
            SortField::Name => Field::Name,
            SortField::CreatedAt => Field::CreatedAt,
            SortField::Language => Field::Language,
            SortField::ImportsCount => Field::ImportsCount,
            SortField::ExportsCount => Field::ExportsCount,
            SortField::Sha256 => Field::Sha256,
            SortField::Complexity => Field::Complexity,
        }
    }
}

/// The API Client implementation.
#[derive(Clone)]
pub struct Client {
    inner: reqwest::Client,
    base_url: String,
}

#[async_trait(?Send)]
impl ApiClient for Client {
    /// Construct an API Client using the `base_url`, which should be the server host address and
    /// port needed to communicate with a Modsurfer backend. Many backends default to http://localhost:1739.
    fn new(base_url: &str) -> Result<Self> {
        let inner = reqwest::ClientBuilder::new()
            .build()
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        Ok(Client {
            inner,
            base_url: base_url.to_string(),
        })
    }

    /// Find a module by its ID.
    async fn get_module(&self, module_id: i64) -> Result<Persisted<Module>> {
        let req = api::GetModuleRequest {
            module_id,
            ..Default::default()
        };
        let res: api::GetModuleResponse = self.send(ModserverCommand::GetModule(req)).await?;
        if res.error.is_some() {
            return Err(api_error(res.error, "get module request failed"));
        }

        if res.module.is_some() {
            Ok(res.module.unwrap().into())
        } else {
            Err(anyhow::anyhow!("No module found."))
        }
    }

    /// List all modules stored in the database. Provide an offset and limit to control the pagination
    /// and size of the result set returned.
    async fn list_modules(&self, offset: u32, limit: u32) -> Result<List<Persisted<Module>>> {
        let mut pagination: api::Pagination = Default::default();
        pagination.limit = limit;
        pagination.offset = offset;

        let mut req = api::ListModulesRequest::new();
        req.pagination = MessageField::some(pagination);

        let res: api::ListModulesResponse = self.send(ModserverCommand::ListModules(req)).await?;
        if res.error.is_some() {
            return Err(api_error(res.error, "list module request failed"));
        }

        let modules = res.modules.into_iter().map(Into::into).collect();

        Ok(List::new(modules, res.total as u32, offset, limit))
    }

    /// Create a new module entry in Modsurfer. If no `location` is set, the module will be named
    /// by its SHA-256 hash + some timestamp in milliseconds. A `location` must be a valid URL, and
    /// can use arbitrary schemes such as `file://<PATH>`, `s3://<BUCKET>/<PATH>`, etc. Use the
    /// `location` to indicate the module's current or eventual storage identifier.
    async fn create_module(
        &self,
        wasm: impl AsRef<[u8]> + Send,
        metadata: Option<HashMap<String, String>>,
        location: std::option::Option<Url>,
    ) -> Result<(i64, String)> {
        let req = api::CreateModuleRequest {
            wasm: wasm.as_ref().to_vec(),
            metadata: metadata.unwrap_or_default(),
            location: location.map(Into::into),
            ..Default::default()
        };

        let res: api::CreateModuleResponse = self.send(ModserverCommand::CreateModule(req)).await?;
        if res.error.is_some() {
            return Err(api_error(res.error, "create module request failed"));
        }

        Ok((res.module_id, res.hash))
    }

    /// Search for modules based on input parameters. The query will combine these inputs using
    /// `AND` conditions.
    async fn search_modules(
        &self,
        module_id: Option<i64>,
        hash: Option<String>,
        function_name: Option<String>,
        module_name: Option<String>,
        imports: Option<Vec<Import>>,
        exports: Option<Vec<Export>>,
        min_size: Option<u64>,
        max_size: Option<u64>,
        location: Option<url::Url>,
        source_language: Option<String>,
        metadata: Option<HashMap<String, String>>,
        inserted_before: Option<chrono::DateTime<chrono::Utc>>,
        inserted_after: Option<chrono::DateTime<chrono::Utc>>,
        strings: Option<Vec<String>>,
        offset: u32,
        limit: u32,
        sort_field: Option<SortField>,
        sort_direction: Option<SortDirection>,
    ) -> Result<List<Persisted<Module>>> {
        let mut pagination: api::Pagination = Default::default();
        pagination.limit = limit;
        pagination.offset = offset;

        let location = if let Some(u) = location {
            Some(u.to_string())
        } else {
            None
        };

        let inserted_before = if let Some(t) = inserted_before {
            protobuf::MessageField::some(protobuf::well_known_types::timestamp::Timestamp {
                seconds: t.timestamp(),
                nanos: t.timestamp_subsec_nanos() as i32,
                special_fields: protobuf::SpecialFields::new(),
            })
        } else {
            protobuf::MessageField::none()
        };

        let inserted_after = if let Some(t) = inserted_after {
            protobuf::MessageField::some(protobuf::well_known_types::timestamp::Timestamp {
                seconds: t.timestamp(),
                nanos: t.timestamp_subsec_nanos() as i32,
                special_fields: protobuf::SpecialFields::new(),
            })
        } else {
            protobuf::MessageField::none()
        };

        let sort = match sort_field {
            Some(f) => MessageField::some(Sort {
                direction: EnumOrUnknown::new(
                    sort_direction
                        .unwrap_or(SortDirection::default())
                        .to_proto(),
                ),
                field: EnumOrUnknown::new(f.to_proto()),
                special_fields: SpecialFields::default(),
            }),
            _ => MessageField::none(),
        };

        let req = api::SearchModulesRequest {
            id: module_id,
            hash,
            function_name,
            module_name,
            imports: to_api::imports(imports.unwrap_or_default()),
            exports: to_api::exports(exports.unwrap_or_default()),
            min_size,
            max_size,
            location,
            sort,
            source_language: source_language
                .map(From::from)
                .map(to_api::source_language)
                .map(EnumOrUnknown::new),
            metadata: metadata.unwrap_or_default(),
            inserted_before,
            inserted_after,
            strings: strings.unwrap_or_default(),
            pagination: MessageField::some(pagination),
            ..Default::default()
        };

        let res: api::SearchModulesResponse =
            self.send(ModserverCommand::SearchModules(req)).await?;
        if res.error.is_some() {
            return Err(api_error(res.error, "search modules request failed"));
        }

        let modules = res.modules.into_iter().map(Into::into).collect();

        Ok(List::new(
            modules,
            res.total as u32,
            res.pagination.offset,
            res.pagination.limit,
        ))
    }

    /// Delete a module from the database. This is a non-reversable operation.
    async fn delete_modules(&self, module_ids: Vec<i64>) -> Result<HashMap<i64, String>> {
        let req = api::DeleteModulesRequest {
            module_ids,
            ..Default::default()
        };

        let res: api::DeleteModulesResponse =
            self.send(ModserverCommand::DeleteModules(req)).await?;
        if res.error.is_some() {
            return Err(api_error(res.error, "delete modules request failed"));
        }

        Ok(res.module_id_hash)
    }

    /// Audit the modules based on a provided checkfile and expected outcome.
    async fn audit_modules(
        &self,
        audit: Audit,
    ) -> Result<HashMap<i64, modsurfer_validation::Report>> {
        let mut pagination: api::Pagination = Default::default();
        pagination.limit = audit.page.limit;
        pagination.offset = audit.page.offset;

        let req = api::AuditModulesRequest {
            outcome: EnumOrUnknown::new(api::AuditOutcome::from(audit.outcome)),
            pagination: MessageField::some(pagination),
            checkfile: audit.checkfile,
            ..Default::default()
        };

        let res: api::AuditModulesResponse = self.send(ModserverCommand::AuditModules(req)).await?;
        if res.error.is_some() {
            return Err(api_error(res.error, "audit modules request failed"));
        }

        let mut id_reports: HashMap<i64, modsurfer_validation::Report> = Default::default();
        res.invalid_module_report
            .iter()
            .for_each(|(id, json_report)| {
                if let Ok(report) = serde_json::from_slice(json_report) {
                    let _ = id_reports.insert(*id, report);
                } else {
                    log::error!("failed to decode validation report for module {}", id);
                }
            });

        Ok(id_reports)
    }

    async fn diff_modules(
        &self,
        module1: i64,
        module2: i64,
        color_terminal: bool,
    ) -> Result<String> {
        let req = api::DiffRequest {
            module1,
            module2,
            color_terminal,
            ..Default::default()
        };

        let res: api::DiffResponse = self.send(ModserverCommand::DiffModules(req)).await?;
        if res.error.is_some() {
            return Err(api_error(res.error, "diff request failed"));
        }
        Ok(res.diff)
    }
}

impl Client {
    async fn send<T: protobuf::Message>(&self, cmd: ModserverCommand) -> Result<T> {
        match cmd {
            ModserverCommand::CreateModule(req) => {
                let resp = self
                    .inner
                    .put(&self.make_endpoint("/api/v1/module"))
                    .body(req.write_to_bytes()?)
                    .send()
                    .await?;
                let data = resp.bytes().await?;
                let val = protobuf::Message::parse_from_bytes(&data)?;

                return Ok(val);
            }
            ModserverCommand::GetModule(req) => {
                let resp = self
                    .inner
                    .post(&self.make_endpoint("/api/v1/module"))
                    .body(req.write_to_bytes()?)
                    .send()
                    .await?;
                let data = resp.bytes().await?;
                let val = protobuf::Message::parse_from_bytes(&data)?;

                return Ok(val);
            }
            ModserverCommand::ListModules(req) => {
                let resp = self
                    .inner
                    .post(&self.make_endpoint("/api/v1/modules"))
                    .body(req.write_to_bytes()?)
                    .send()
                    .await?;
                let data = resp.bytes().await?;
                let val = protobuf::Message::parse_from_bytes(&data)?;

                return Ok(val);
            }
            ModserverCommand::SearchModules(req) => {
                let resp = self
                    .inner
                    .post(&self.make_endpoint("/api/v1/search"))
                    .body(req.write_to_bytes()?)
                    .send()
                    .await?;
                let data = resp.bytes().await?;
                let val = protobuf::Message::parse_from_bytes(&data)?;

                return Ok(val);
            }
            ModserverCommand::DeleteModules(req) => {
                let resp = self
                    .inner
                    .delete(&self.make_endpoint("/api/v1/modules"))
                    .body(req.write_to_bytes()?)
                    .send()
                    .await?;
                let data = resp.bytes().await?;
                let val = protobuf::Message::parse_from_bytes(&data)?;

                return Ok(val);
            }
            ModserverCommand::AuditModules(req) => {
                let resp = self
                    .inner
                    .post(&self.make_endpoint("/api/v1/audit"))
                    .body(req.write_to_bytes()?)
                    .send()
                    .await?;
                let data = resp.bytes().await?;
                let val = protobuf::Message::parse_from_bytes(&data)?;

                return Ok(val);
            }
            ModserverCommand::DiffModules(req) => {
                let resp = self
                    .inner
                    .post(&self.make_endpoint("/api/v1/diff"))
                    .body(req.write_to_bytes()?)
                    .send()
                    .await?;
                let data = resp.bytes().await?;
                let val = protobuf::Message::parse_from_bytes(&data)?;
                return Ok(val);
            }
        }
    }

    fn make_endpoint(&self, route: &str) -> String {
        let base = self.base_url.trim_end_matches('/');
        let s = format!("{}{}", base, route);
        s
    }
}

fn api_error(
    error: protobuf::MessageField<modsurfer_convert::api::Error>,
    msg: &str,
) -> anyhow::Error {
    let e = error.get_or_default();

    return anyhow::anyhow!("{}: {} [{}]", msg, e.message, e.code);
}
