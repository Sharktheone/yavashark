use swc_ecma_ast::{BlockStmt, Decl, Stmt, VarDeclKind};

use yavashark_env::scope::Scope;
use yavashark_env::{Realm, RuntimeResult, Value};

use crate::Interpreter;

fn needs_block_scope(stmts: &[Stmt]) -> bool {
    stmts.iter().any(|s| match s {
        Stmt::Decl(Decl::Var(v)) => !matches!(v.kind, VarDeclKind::Var),
        Stmt::Decl(_) => true,
        _ => false,
    })
}

impl Interpreter {
    pub fn run_block(realm: &mut Realm, stmt: &BlockStmt, scope: &mut Scope) -> RuntimeResult {
        if !needs_block_scope(&stmt.stmts) {
            return Self::run_statements(realm, &stmt.stmts, scope);
        }

        let scope = &mut Scope::with_parent(scope)?;

        Self::run_statements(realm, &stmt.stmts, scope)
    }
    pub fn run_block_this(
        realm: &mut Realm,
        stmt: &BlockStmt,
        scope: &mut Scope,
        this: Value,
    ) -> RuntimeResult {
        let scope = &mut Scope::with_parent_this(scope, this)?;

        Self::run_statements(realm, &stmt.stmts, scope)
    }
}
