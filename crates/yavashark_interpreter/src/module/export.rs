use swc_ecma_ast::{ExportAll, ExportDecl, ExportDefaultDecl, ExportDefaultExpr, NamedExport};
use yavashark_env::{Realm, RuntimeResult};
use yavashark_env::scope::{ModuleScope, Scope};
use crate::Interpreter;

impl Interpreter {
    pub fn run_export_decl(relm: &mut Realm, stmt: &ExportDecl, scope: &mut ModuleScope) -> RuntimeResult {
        todo!()
    }

    pub fn run_export_named(relm: &mut Realm, stmt: &NamedExport, scope: &mut ModuleScope) -> RuntimeResult {
        todo!()
    }

    pub fn run_export_default_expr(realm: &mut Realm, stmt: &ExportDefaultExpr, scope: &mut ModuleScope) -> RuntimeResult {
        todo!()
    }
    
    pub fn run_export_default_decl(relm: &mut Realm, stmt: &ExportDefaultDecl, scope: &mut ModuleScope) -> RuntimeResult {
        todo!()
    }

    pub fn run_export_all(relm: &mut Realm, stmt: &ExportAll, scope: &mut ModuleScope) -> RuntimeResult {
        todo!()
    }
}