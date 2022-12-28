use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use modsurfer::{Export, Import, Module};
use modsurfer_convert::{api, to_api};
use protobuf::{self, EnumOrUnknown, Message, MessageField};
use reqwest;
use url::Url;

use super::{ApiClient, List, Persisted};

enum ModserverCommand {
    CreateModule(api::CreateModuleRequest),
    GetModule(api::GetModuleRequest),
    ListModules(api::ListModulesRequest),
    SearchModules(api::SearchModulesRequest),
}

#[derive(Clone)]
pub struct Client {
    inner: reqwest::Client,
    base_url: String,
}

#[async_trait(?Send)]
impl ApiClient for Client {
    fn new(base_url: &str) -> Result<Self> {
        let inner = reqwest::ClientBuilder::new()
            .build()
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        Ok(Client {
            inner,
            base_url: base_url.to_string(),
        })
    }

    async fn get_module(&self, module_id: i64) -> Result<Persisted<Module>> {
        let req = api::GetModuleRequest {
            module_id,
            ..Default::default()
        };
        let res: api::GetModuleResponse = self.send(ModserverCommand::GetModule(req)).await?;
        if res.error.is_some() {
            return Err(api_error(res.error, "get module request failed"));
        }

        Ok(res.module.unwrap_or_default().into())
    }

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
        }
    }

    fn make_endpoint(&self, route: &str) -> String {
        format!("{}{}", self.base_url, route)
    }
}

fn api_error(
    error: protobuf::MessageField<modsurfer_convert::api::Error>,
    msg: &str,
) -> anyhow::Error {
    let e = error.get_or_default();

    return anyhow::anyhow!("{}: {} [{}]", msg, e.message, e.code);
}
