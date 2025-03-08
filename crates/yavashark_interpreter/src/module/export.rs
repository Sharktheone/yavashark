use swc_common::Spanned;
use swc_ecma_ast::{DefaultDecl, ExportAll, ExportDecl, ExportDefaultDecl, ExportDefaultExpr, NamedExport};
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
        todo!()
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