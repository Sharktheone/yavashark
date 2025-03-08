use swc_common::Spanned;
use swc_ecma_ast::{DefaultDecl, ExportAll, ExportDecl, ExportDefaultDecl, ExportDefaultExpr, ExportSpecifier, ImportSpecifier, ModuleExportName, NamedExport};
use swc_ecma_parser::token::Keyword::Export;
use yavashark_env::{Error, Realm, RuntimeResult, Value, Variable};
use yavashark_env::scope::{ModuleScope, Scope};
use yavashark_value::IntoValue;
use crate::Interpreter;
use crate::statement::decl::{var, DeclRet};

impl Interpreter {
    pub fn run_export_decl(realm: &mut Realm, stmt: &ExportDecl, scope: &mut ModuleScope) -> RuntimeResult {
        let val = Self::run_decl_ret(realm, &stmt.decl, &mut scope.scope)?;
        
        match val {
            DeclRet::Single(name, value) => {
                scope.scope.declare_var(name.clone(), value.copy());
                scope.module.exports.define_property(name.into(), value.into()); //TODO: if the value changes, the export should change too
            },
            DeclRet::Var(vars) => {
                for var in vars {
                    match var {
                        var::Variable::Var(name, value) => {
                            scope.scope.declare_global_var(name.clone(), value.copy());
                            scope.module.exports.define_property(name.into(), value.into()); //TODO: if the value changes, the export should change too
                        }
                        var::Variable::Let(name, value) => {
                            scope.scope.declare_var(name.clone(), value.copy());
                            scope.module.exports.define_property(name.into(), value.into()); //TODO: if the value changes, the export should change too
                        }
                        var::Variable::Const(name, value) => {
                            scope.scope.declare_read_only_var(name.clone(), value.copy());
                            scope.module.exports.define_property(name.into(), value.into()); //TODO: if the value changes, the export should change too
                        }
                    }
                }
            },
        }
        
        
        Ok(Value::Undefined)
    }

    pub fn run_export_named(realm: &mut Realm, stmt: &NamedExport, scope: &mut ModuleScope) -> RuntimeResult {
        let Some(src) = &stmt.src else {
            return Err(Error::syn("Export source is required").into());
        };
        
        
        let module = Self::resolve_module(&src.value, stmt.with.as_deref(), &scope.scope.get_current_path()?, realm)?;
        
        for spec in &stmt.specifiers {
            match spec {
                ExportSpecifier::Named(named) => {
                    let name = match &named.orig {
                        ModuleExportName::Ident(id) => id.sym.to_string(),
                        ModuleExportName::Str(str) => str.value.to_string(),
                    };
                    let export = match &named.exported {
                        Some(ModuleExportName::Ident(id)) => id.sym.to_string(),
                        Some(ModuleExportName::Str(str)) => str.value.to_string(),
                        None => name.clone(),
                    };
                    
                    
                    
                    let val = module
                        .exports
                        .get_property(&name.clone().into())
                        .map_err(|_| {
                            Error::reference_error(format!("Export `{name}` not found in module"))
                        })?;
                    
                    scope.scope.declare_var(export.clone(), val.value.copy())?;
                    scope.module.exports.define_property(export.into(), val.value.copy().into()); //TODO: if the value changes, the export should change too
                }
                
                ExportSpecifier::Default(default) => {
                    let Some(val) = &module.default else {
                        return Err(Error::reference("Module has no default export").into());
                    };
                    
                    let name = default.exported.to_string();
                    
                    scope.scope.declare_var(name.clone(), val.copy())?;
                    scope.module.exports.define_property(name.into(), val.copy().into()); //TODO: if the value changes, the export should change too
                }
                
                ExportSpecifier::Namespace(ns) => {
                    let name = match &ns.name {
                        ModuleExportName::Ident(id) => id.sym.to_string(),
                        ModuleExportName::Str(str) => str.value.to_string(),
                    };
                    
                    scope.scope.declare_var(name.clone(), module.exports.clone().into())?;
                    scope.module.exports.define_property(name.into(), module.exports.clone().into()); //TODO: if the value changes, the export should change too
                }
            }
        }
        
        
        Ok(Value::Undefined)
    }

    pub fn run_export_default_expr(realm: &mut Realm, stmt: &ExportDefaultExpr, scope: &mut ModuleScope) -> RuntimeResult {
        let val = Self::run_expr(realm, &stmt.expr, stmt.expr.span(), &mut scope.scope)?;
        
        scope.module.default = Some(val);
        
        Ok(Value::Undefined)
    }
    
    pub fn run_export_default_decl(realm: &mut Realm, stmt: &ExportDefaultDecl, scope: &mut ModuleScope) -> RuntimeResult {
        match &stmt.decl {
            DefaultDecl::Class(c) => {
                let class = Self::run_class(realm, &c, &mut scope.scope)?;
                
                scope.module.default = Some(class);
            }
            
            DefaultDecl::Fn(f) => {
                let func = Self::run_fn(realm, &f, &mut scope.scope)?;
                
                scope.module.default = Some(func);
            }
            
            _ => return Err(Error::syn("TypeScript is not supported").into()),
        }
        
        Ok(Value::Undefined)
    }

    pub fn run_export_all(realm: &mut Realm, stmt: &ExportAll, scope: &mut ModuleScope) -> RuntimeResult {
        todo!()
    }
}