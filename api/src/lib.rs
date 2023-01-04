#[cfg(not(feature = "mock"))]
mod client;

mod interop;

#[cfg(feature = "mock")]
mod mock_client;

use std::collections::HashMap;

#[cfg(feature = "mock")]
pub use mock_client::Client;

#[cfg(not(feature = "mock"))]
pub use client::Client;

pub use interop::{List, Persisted};

pub use anyhow::Result;
use async_trait::async_trait;
use modsurfer_module::{Export, Import, Module};

#[async_trait(?Send)]
pub trait ApiClient {
    fn new(base_url: &str) -> Result<Self>
    where
        Self: Sized;
    async fn get_module(&self, module_id: i64) -> Result<Persisted<Module>>;
    async fn list_modules(&self, offset: u32, limit: u32) -> Result<List<Persisted<Module>>>;
    async fn create_module(
        &self,
        wasm: impl AsRef<[u8]> + Send,
        metadata: Option<HashMap<String, String>>,
        location: Option<url::Url>,
    ) -> Result<(i64, String)>;
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
    ) -> Result<List<Persisted<Module>>>;
}