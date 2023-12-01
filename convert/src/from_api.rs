use crate::*;

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use crate::types::Audit;

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use chrono::offset::TimeZone;

use modsurfer_module::{Export, Function, FunctionType, Import, ValType};

pub fn source_language(src: api::SourceLanguage) -> SourceLanguage {
    match src {
        api::SourceLanguage::Unknown => SourceLanguage::Unknown,
        api::SourceLanguage::AssemblyScript => SourceLanguage::AssemblyScript,
        api::SourceLanguage::C => SourceLanguage::C,
        api::SourceLanguage::Cpp => SourceLanguage::Cpp,
        api::SourceLanguage::Go => SourceLanguage::Go,
        api::SourceLanguage::Rust => SourceLanguage::Rust,
    }
}

pub fn val_types(v: Vec<protobuf::EnumOrUnknown<api::ValType>>) -> Vec<ValType> {
    v.into_iter()
        .map(|x| val_type(x.enum_value_or_default()))
        .collect()
}

pub fn val_type(v: api::ValType) -> ValType {
    match v {
        api::ValType::I32 => ValType::I32,
        api::ValType::I64 => ValType::I64,
        api::ValType::F32 => ValType::F32,
        api::ValType::F64 => ValType::F64,
        api::ValType::V128 => ValType::V128,
        api::ValType::FuncRef => ValType::FuncRef,
        api::ValType::ExternRef => ValType::ExternRef,
        api::ValType::Bool => ValType::ExternRef,
        api::ValType::S8 => ValType::ExternRef,
        api::ValType::U8 => ValType::ExternRef,
        api::ValType::S16 => ValType::ExternRef,
        api::ValType::U16 => ValType::ExternRef,
        api::ValType::S32 => ValType::ExternRef,
        api::ValType::U32 => ValType::ExternRef,
        api::ValType::S64 => ValType::ExternRef,
        api::ValType::U64 => ValType::ExternRef,
        api::ValType::Float32 => ValType::ExternRef,
        api::ValType::Float64 => ValType::ExternRef,
        api::ValType::Char => ValType::ExternRef,
        api::ValType::String => ValType::ExternRef,
    }
}

pub fn sort(sort: api::Sort) -> Sort {
    Sort {
        order: match sort.direction.enum_value_or_default() {
            api::Direction::Asc => Order::Asc,
            api::Direction::Desc => Order::Desc,
        },
        field: match sort.field.enum_value_or_default() {
            api::Field::CreatedAt => SortField::CreatedAt,
            api::Field::ExportsCount => SortField::ExportsCount,
            api::Field::ImportsCount => SortField::ImportsCount,
            api::Field::Language => SortField::Language,
            api::Field::Name => SortField::Name,
            api::Field::Sha256 => SortField::Sha256,
            api::Field::Size => SortField::Size,
            api::Field::Complexity => SortField::Complexity,
        },
    }
}

pub fn import(import: api::Import) -> Import {
    let name = import.func.name.to_string();
    let f = import.func.into_option().unwrap_or_default();
    Import {
        module_name: import.module_name,
        func: Function {
            name,
            ty: FunctionType {
                params: val_types(f.params),
                results: val_types(f.results),
            },
        },
    }
}

pub fn imports(imports: Vec<api::Import>) -> Vec<Import> {
    imports.into_iter().map(import).collect()
}

pub fn export(export: api::Export) -> Export {
    let name = export.func.name.to_string();
    let f = export.func.into_option().unwrap_or_default();
    Export {
        func: Function {
            name,
            ty: FunctionType {
                params: val_types(f.params),
                results: val_types(f.results),
            },
        },
    }
}

pub fn exports(exports: Vec<api::Export>) -> Vec<Export> {
    exports.into_iter().map(export).collect()
}

pub fn module(module: &modsurfer_proto_v1::api::Module) -> modsurfer_module::Module {
    let modsurfer_module = &mut modsurfer_module::Module {
        hash: module.hash.clone(),
        imports: imports(module.imports.clone()),
        exports: exports(module.exports.clone()),
        size: module.size,
        location: module.location.clone(),
        source_language: source_language(module.source_language.enum_value_or_default()),
        metadata: Some(module.metadata.clone()),
        strings: module.strings.clone(),
        complexity: module.complexity,
        graph: module.graph.clone(),
        function_hashes: module.function_hashes.clone(),
        #[cfg(not(target_arch = "wasm32"))]
        inserted_at: chrono::DateTime::from(
            chrono::Utc.timestamp_nanos(module.inserted_at.nanos as i64),
        ),
        #[cfg(target_arch = "wasm32")]
        inserted_at: module.inserted_at.nanos as u64,
    };

    return modsurfer_module.clone();
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
pub fn search(mut req: api::SearchModulesRequest) -> Search {
    Search {
        page: req
            .pagination
            .as_ref()
            .map(Pagination::from)
            .unwrap_or_default(),
        hash: req.hash,
        location: req.location,
        function_name: req.function_name,
        module_name: req.module_name,
        inserted_after: req.inserted_after.as_ref().and_then(|x| {
            Some(chrono::DateTime::<chrono::Utc>::from_utc(
                chrono::NaiveDateTime::from_timestamp_opt(x.seconds, x.nanos as u32)?,
                chrono::Utc,
            ))
        }),
        inserted_before: req.inserted_before.as_ref().and_then(|x| {
            Some(chrono::DateTime::<chrono::Utc>::from_utc(
                chrono::NaiveDateTime::from_timestamp_opt(x.seconds, x.nanos as u32)?,
                chrono::Utc,
            ))
        }),
        strings: if req.strings.is_empty() {
            None
        } else {
            Some(req.strings)
        },
        sort: req.sort.take().map(sort),
        source_language: req
            .source_language
            .map(|x| source_language(x.enum_value_or_default())),
        imports: imports(req.imports),
        exports: exports(req.exports),
    }
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
pub fn audit(req: api::AuditModulesRequest) -> Audit {
    Audit {
        page: req
            .pagination
            .as_ref()
            .map(Pagination::from)
            .unwrap_or_default(),
        outcome: types::AuditOutcome::from(req.outcome.enum_value_or_default()),
        checkfile: req.checkfile,
    }
}
