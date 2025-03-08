use swc_ecma_ast::{ImportDecl, ImportSpecifier, ModuleExportName};
use yavashark_env::{ControlFlow, Error, Realm, RuntimeResult, Value};
use yavashark_env::scope::Scope;
use yavashark_value::IntoValue;
use crate::Interpreter;

impl Interpreter {

    pub fn run_import(realm: &mut Realm, stmt: &ImportDecl, scope: &mut Scope) -> RuntimeResult {
        let src = stmt.src.value.to_string();
        
        //TODO: handle `with`
        
        let (source, path) = realm.resolve_module(&src, &scope.get_current_path()?)
            .map_err(|e| ControlFlow::Error(Error::reference_error(e.to_string())))?;
        
        let module = Self::run_module_source(&source, path, realm)?;
        
        for spec in &stmt.specifiers {
            match spec {
                ImportSpecifier::Named(named) => {
                    let name = match &named.imported {
                        Some(ModuleExportName::Ident(id)) => id.sym.to_string(),
                        Some(ModuleExportName::Str(str)) => str.value.to_string(),
                        None => named.local.to_string(),
                    };
                    let local = named.local.to_string();
                    
                    let val = module.exports.get_property(&name.clone().into())
                        .map_err(|_| Error::reference_error(format!("Export `{}` not found in module", name)))?;
                    
                    scope.declare_var(local, val.value.copy())?;
                    
                }
                
                ImportSpecifier::Default(default) => {
                    let Some(val) = &module.default else {
                        return Err(Error::reference("Module has no default export").into());
                    };
                    
                    scope.declare_var(default.local.to_string(), val.copy())?
                    
                }
                
                ImportSpecifier::Namespace(ns) => {
                    scope.declare_var(ns.local.to_string(), module.exports.clone().into())?
                    
                }
            }
        }
        
        
        
        
        
        
        Ok(Value::Undefined)
    }

}