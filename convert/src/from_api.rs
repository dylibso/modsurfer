use crate::*;

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

pub fn val_types(v: Vec<protobuf::EnumOrUnknown<api::ValType>>) -> Vec<modsurfer::ValType> {
    v.into_iter()
        .map(|x| val_type(x.enum_value_or_default()))
        .collect()
}

pub fn val_type(v: api::ValType) -> modsurfer::ValType {
    match v {
        api::ValType::I32 => modsurfer::ValType::I32,
        api::ValType::I64 => modsurfer::ValType::I64,
        api::ValType::F32 => modsurfer::ValType::F32,
        api::ValType::F64 => modsurfer::ValType::F64,
        api::ValType::V128 => modsurfer::ValType::V128,
        api::ValType::FuncRef => modsurfer::ValType::FuncRef,
        api::ValType::ExternRef => modsurfer::ValType::ExternRef,
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
        },
    }
}

pub fn import(import: api::Import) -> modsurfer::Import {
    let name = import.func.name.to_string();
    let f = import.func.into_option().unwrap_or_default();
    modsurfer::Import {
        module_name: import.module_name,
        func: modsurfer::Function {
            name,
            ty: modsurfer::FunctionType {
                args: val_types(f.args),
                returns: val_types(f.returns),
            },
        },
    }
}

pub fn imports(imports: Vec<api::Import>) -> Vec<modsurfer::Import> {
    imports.into_iter().map(import).collect()
}

pub fn export(export: api::Export) -> modsurfer::Export {
    let name = export.func.name.to_string();
    let f = export.func.into_option().unwrap_or_default();
    modsurfer::Export {
        func: modsurfer::Function {
            name,
            ty: modsurfer::FunctionType {
                args: val_types(f.args),
                returns: val_types(f.returns),
            },
        },
    }
}

pub fn exports(exports: Vec<api::Export>) -> Vec<modsurfer::Export> {
    exports.into_iter().map(export).collect()
}

#[cfg(feature = "api")]
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
