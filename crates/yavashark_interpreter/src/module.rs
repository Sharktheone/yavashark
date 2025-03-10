mod import;
mod export;

use std::path::{Path, PathBuf};
use swc_ecma_ast::{ExportAll, ExportDecl, ExportDefaultDecl, ExportDefaultExpr, ImportDecl, ModuleDecl, ModuleItem, NamedExport};
use yavashark_env::{Error, Realm, RuntimeResult, Value, Res};
use yavashark_env::scope::{Module, ModuleScope, Scope};
use crate::Interpreter;

impl Interpreter {
    pub fn run_module_items(
        realm: &mut Realm,
        script: &Vec<ModuleItem>,
        scope: &mut ModuleScope,
    ) -> RuntimeResult {
        let mut last_value = Value::Undefined;
        for stmt in script {
            
            match stmt {
                ModuleItem::ModuleDecl(decl) => {
                    Self::run_module_decl(realm, decl, scope)?;
                }
                ModuleItem::Stmt(stmt) => {
                    last_value = Self::run_statement(realm, stmt, &mut scope.scope)?;
                }
            }
        }

        Ok(last_value)
    }
    
    pub fn run_module_decl(
        realm: &mut Realm,
        decl: &ModuleDecl,
        scope: &mut ModuleScope,
    ) -> RuntimeResult {
        match decl {
            ModuleDecl::Import(import) => {
                Self::run_import(realm, import, &mut scope.scope)
            }
            ModuleDecl::ExportDecl(export) => {
                Self::run_export_decl(realm, export, scope)
            }
            ModuleDecl::ExportNamed(export) => {
                Self::run_export_named(realm, export, scope)
            }
            ModuleDecl::ExportDefaultDecl(export) => {
                Self::run_export_default_decl(realm, export, scope)
            }
            ModuleDecl::ExportDefaultExpr(export) => {
                Self::run_export_default_expr(realm, export, scope)
            }
            ModuleDecl::ExportAll(export) => {
                Self::run_export_all(realm, export, scope)
            }
            
            _ => Err(Error::syn("TypesScript not supported yet").into()),
        }
    }
    
    pub fn run_module_source(source: &str, path: PathBuf,  realm: &mut Realm) -> Res<Module> {
        let module = crate::parse::parse_module(source)
            .map_err(|e| Error::syn_error(format!("{e:?}")))?;
        
        let scope = Scope::global(realm, path);
        
        let mut scope = ModuleScope {
            scope,
            module: Module::default()
        };
        
        Self::run_module_in(&module.body, realm, &mut scope)?;
        
        Ok(scope.module)
    }
}