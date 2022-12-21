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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sort {
    pub order: Order,
    pub field: SortField,
}

#[derive(Default)]
pub struct Search {
    pub page: Pagination,
    pub hash: Option<String>,
    pub location: Option<String>,
    pub imports: Vec<modsurfer::Import>,
    pub exports: Vec<modsurfer::Export>,
    pub function_name: Option<String>,
    pub module_name: Option<String>,
    pub source_language: Option<modsurfer::SourceLanguage>,
    pub inserted_after: Option<chrono::DateTime<chrono::Utc>>,
    pub inserted_before: Option<chrono::DateTime<chrono::Utc>>,
    pub strings: Option<Vec<String>>,
    pub sort: Option<Sort>,
}
