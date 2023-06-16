use crate::*;

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use crate::types::Audit;

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
