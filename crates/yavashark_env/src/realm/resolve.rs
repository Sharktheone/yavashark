use crate::scope::Module;
use crate::{Error, Realm, Res};
use std::path::{Path, PathBuf};
use std::pin::Pin;

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

    pub fn get_module_async(
        &mut self,
        spec: &str,
        cur_path: &Path,
        cb: impl FnOnce(String, PathBuf, &mut Realm) -> Res<Module> + 'static,
    ) -> Res<ResolveModuleResult> {
        let path = resolve_path(spec, cur_path)?;

        if path == cur_path {
            return Err(Error::new("TODO: handle circular dependencies"));
        }

        if !self.env.modules.contains_key(&path) {
            let fut = async move {
                #[cfg(not(target_arch = "wasm32"))]
                let source = tokio::fs::read_to_string(&path)
                    .await
                    .map_err(|e| Error::new_error(e.to_string()))?;
                #[cfg(target_arch = "wasm32")]
                let source = {
                    // load in sync with stdlib
                    std::fs::read_to_string(&path)
                        .map_err(|e| Error::new_error(e.to_string()))?
                };

                Ok(ModuleFinalizer { source, path, cb: Box::new(cb) })
            };

            return Ok(ResolveModuleResult::Async(Box::pin(fut)));
        }

        Ok(ResolveModuleResult::Module(
            self.env
                .modules
                .get(&path)
                .ok_or(Error::new("failed to get module"))?,
        ))
    }
}

pub enum ResolveModuleResult<'a> {
    Module(&'a Module),
    Async(Pin<Box<dyn std::future::Future<Output = Res<ModuleFinalizer>>>>),
}

pub struct ModuleFinalizer {
    pub source: String,
    cb: Box<dyn FnOnce(String, PathBuf, &mut Realm) -> Res<Module>>,
    pub path: PathBuf,
}

impl ModuleFinalizer {
    pub fn finalize(
        self,
        realm: &mut Realm,
    ) -> Res<&Module> {
        let module = (self.cb)(self.source, self.path.clone(), realm)?;

        realm.env.modules.insert(self.path.clone(), module);

        realm
            .env
            .modules
            .get(&self.path)
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
