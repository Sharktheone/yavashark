use std::{fs, io};
use std::path::{Path, PathBuf};
use crate::Realm;

impl Realm {
    pub fn resolve_module(&mut self, specifier: &str, path: &Path) -> io::Result<(String, PathBuf)> {
        let path = path.join(specifier); //TODO: handle http/https, data urls, etc.

        Ok((fs::read_to_string(&path)?, path))
    }
}