use modsurfer_convert::api::{Direction, Field};

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
    pub fn to_proto(self) -> Direction {
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

    pub fn to_proto(self) -> Field {
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
