use crate::Interpreter;
use std::path::Path;
use swc_ecma_ast::{ImportDecl, ImportSpecifier, ModuleExportName, ObjectLit};
use yavashark_env::scope::{Module, Scope};
use yavashark_env::{ControlFlow, Error, Realm, Res, RuntimeResult, Value};

impl Interpreter {
    pub fn run_import(realm: &mut Realm, stmt: &ImportDecl, scope: &mut Scope) -> RuntimeResult {
        let src = stmt.src.value.to_string();

        let module = Self::resolve_module(
            &src,
            stmt.with.as_deref(),
            &scope.get_current_path()?,
            realm,
        )?;

        let module = module.clone();

        for spec in &stmt.specifiers {
            match spec {
                ImportSpecifier::Named(named) => {
                    let name = match &named.imported {
                        Some(ModuleExportName::Ident(id)) => id.sym.to_string(),
                        Some(ModuleExportName::Str(str)) => str.value.to_string(),
                        None => named.local.to_string(),
                    };
                    let local = named.local.to_string();

                    let val = module
                        .exports
                        .resolve_property(name.clone(), realm)
                        .map_err(|_| {
                            Error::reference_error(format!("Export `{name}` not found in module"))
                        })?
                        .unwrap_or(Value::Undefined);

                    scope.declare_var(local, val, realm)?;
                }

                ImportSpecifier::Default(default) => {
                    let Some(val) = &module.default else {
                        return Err(Error::reference("Module has no default export").into());
                    };

                    scope.declare_var(default.local.to_string(), val.copy(), realm)?;
                }

                ImportSpecifier::Namespace(ns) => {
                    scope.declare_var(ns.local.to_string(), module.exports.clone().into(), realm)?;
                }
            }
        }

        Ok(Value::Undefined)
    }

    pub fn resolve_module<'a>(
        src: &str,
        with: Option<&ObjectLit>,
        path: &Path,
        realm: &'a mut Realm,
    ) -> Res<&'a Module> {
        //TODO: handle `with`
        realm.get_module(src, path, |source, path, realm| {
            Self::run_module_source(&source, path, realm)
        })
    }
}
