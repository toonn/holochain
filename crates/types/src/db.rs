use crate::agent::CellId;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct DatabasePath(PathBuf);

impl From<CellId> for DatabasePath {
    fn from(id: CellId) -> Self {
        let database_path = PathBuf::new().join(format!("{}", id));
        DatabasePath(database_path.into())
    }
}

impl From<&Path> for DatabasePath {
    fn from(path: &Path) -> Self {
        DatabasePath(path.into())
    }
}

impl AsRef<Path> for DatabasePath {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
    }
}