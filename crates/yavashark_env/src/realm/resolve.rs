use crate::scope::Module;
use crate::{Error, Realm, Res};
use std::path::{Path, PathBuf};

impl Realm {
    pub fn get_module(
        &mut self,
        spec: &str,
        cur_path: &Path,
        mut cb: impl FnMut(String, PathBuf, &mut Self) -> Res<Module>,
    ) -> Res<&Module> {
        let path = resolve_path(spec, cur_path)?;

        if path == cur_path {
            return Err(Error::new("TODO: handle circular dependencies"));
        }

        if !self.env.modules.contains_key(&path) {
            let source =
                std::fs::read_to_string(&path).map_err(|e| Error::new_error(e.to_string()))?;

            let module = cb(source, path.clone(), self)?;

            self.env.modules.insert(path.clone(), module);
        }

        self.env
            .modules
            .get(&path)
            .ok_or(Error::new("failed to get module"))
    }
}

pub fn resolve_path(spec: &str, path: &Path) -> Res<PathBuf> {
    Ok(if path.is_dir() {
        path.join(spec) //TODO: handle http/https, data urls, etc.
    } else {
        path.parent()
            .ok_or(Error::new("failed to resolve module path"))?
            .join(spec)
    })
}
