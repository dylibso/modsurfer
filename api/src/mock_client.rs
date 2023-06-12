use std::{collections::HashMap, sync::Mutex};

use anyhow::Result;
use async_trait::async_trait;

use lazy_static::lazy_static;

#[cfg(not(feature = "mock-empty"))]
use modsurfer_convert::api::ListModulesResponse;

use modsurfer_module::{Export, Import, Module, SourceLanguage};
use url::Url;

use crate::{ApiClient, List, Persisted, SortDirection, SortField};

#[cfg(not(feature = "mock-empty"))]
lazy_static! {
    static ref MODULES: &'static [u8] = include_bytes!("../ListModulesResponse.pb");
    static ref PB_DATA: ListModulesResponse =
        protobuf::Message::parse_from_bytes(&MODULES).unwrap();
    static ref MOCK_CLIENT_DATA: Mutex<Vec<Persisted<Module>>> = Mutex::new(
        PB_DATA
            .modules
            .clone()
            .into_iter()
            .map(Into::into)
            .collect::<Vec<Persisted<Module>>>()
    );
}

#[cfg(feature = "mock-empty")]
lazy_static! {
    static ref MOCK_CLIENT_DATA: Mutex<Vec<Persisted<Module>>> = Mutex::new(vec![]);
}

#[derive(Clone, Default)]
pub struct Client;

impl Client {
    pub fn modules(&self) -> Vec<Persisted<Module>> {
        MOCK_CLIENT_DATA.lock().unwrap().to_vec()
    }

    pub fn module(&self, module_id: i64) -> Option<Persisted<Module>> {
        MOCK_CLIENT_DATA
            .lock()
            .unwrap()
            .iter()
            .find(|m| m.get_id() == module_id)
            .map(|m| m.clone())
    }
}

#[async_trait(?Send)]
impl ApiClient for Client {
    fn new(_base_url: &str) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self)
    }

    async fn get_module(&self, module_id: i64) -> Result<Persisted<Module>> {
        match self.module(module_id) {
            Some(p) => Ok(p),
            None => Err(anyhow::anyhow!("no module found")), // TODO: improve errors
        }
    }

    async fn list_modules(&self, offset: u32, limit: u32) -> Result<List<Persisted<Module>>> {
        let all = self.modules();
        let total = all.len() as u32;
        let modules = all
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect();
        let list = List::new(modules, total, offset, limit);
        Ok(list)
    }

    async fn create_module(
        &self,
        _wasm: impl AsRef<[u8]> + Send,
        metadata: Option<HashMap<String, String>>,
        _location: Option<Url>,
    ) -> Result<(i64, String)> {
        let mut module = Module::default();

        module.metadata = metadata;

        let id = (MOCK_CLIENT_DATA.lock().unwrap().len() + 1) as i64;
        let hash = module.hash.clone();
        MOCK_CLIENT_DATA
            .lock()
            .unwrap()
            .push(Persisted::from_module(id, module));

        Ok((id, hash))
    }

    async fn search_modules(
        &self,
        module_id: Option<i64>,
        hash: Option<String>,
        function_name: Option<String>,
        module_name: Option<String>,
        _imports: Option<Vec<Import>>,
        _exports: Option<Vec<Export>>,
        _min_size: Option<u64>,
        _max_size: Option<u64>,
        _location: Option<url::Url>,
        source_language: Option<String>,
        _metadata: Option<HashMap<String, String>>,
        _inserted_before: Option<chrono::DateTime<chrono::Utc>>,
        _inserted_after: Option<chrono::DateTime<chrono::Utc>>,
        strings: Option<Vec<String>>,
        offset: u32,
        limit: u32,
        _sort_field: Option<SortField>,
        _sort_direction: Option<SortDirection>,
    ) -> Result<List<Persisted<Module>>> {
        let modules = MOCK_CLIENT_DATA.lock().unwrap();

        if let Some(module_id) = module_id {
            if let Some(module) = modules.iter().find(|p| p.get_id() == module_id) {
                return Ok(List::new(vec![module.clone()], 1, offset, limit));
            }
        }

        if let Some(hash) = hash {
            if let Some(module) = modules.iter().find(|p| p.get_inner().hash == hash) {
                return Ok(List::new(vec![module.clone()], 1, offset, limit));
            }
        }

        let mut filtered = modules.clone();

        if let Some(function_name) = function_name {
            filtered = filtered
                .into_iter()
                .filter(|p| {
                    p.get_inner()
                        .imports
                        .iter()
                        .any(|i| i.func.name == function_name)
                        || p.get_inner()
                            .exports
                            .iter()
                            .any(|i| i.func.name == function_name)
                })
                .collect();
        }

        if let Some(module_name) = module_name {
            filtered = filtered
                .into_iter()
                .filter(|p| {
                    p.get_inner()
                        .imports
                        .iter()
                        .any(|i| i.module_name == module_name)
                })
                .collect();
        }

        if let Some(source_language) = source_language {
            let lang: SourceLanguage = source_language.into();
            filtered = filtered
                .into_iter()
                .filter(|p| p.get_inner().source_language == lang)
                .collect();
        }

        if let Some(strings) = strings {
            filtered = filtered
                .into_iter()
                .filter(|p| {
                    strings
                        .iter()
                        .any(|s| p.get_inner().strings.iter().any(|mod_s| mod_s.contains(s)))
                })
                .collect();
        }

        let total = filtered.len() as u32;
        Ok(List::new(filtered, total, offset, limit))
    }

    async fn diff_modules(
        &self,
        _module1: i64,
        _module2: i64,
        _color_terminal: bool,
        _with_context: bool,
    ) -> Result<String> {
        anyhow::bail!("Diff operation unimplemented.")
    }
}
