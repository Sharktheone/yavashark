use crate::Validator;
use std::collections::HashSet;
use swc_ecma_ast::{
    BlockStmt, Decl, ForHead, ObjectPatProp, Pat, Stmt, VarDeclKind, VarDeclOrExpr,
};

fn collect_pat_idents<'a>(pat: &'a Pat, out: &mut Vec<&'a str>) {
    match pat {
        Pat::Ident(i) => out.push(i.id.sym.as_str()),
        Pat::Array(a) => {
            for e in a.elems.iter().flatten() {
                collect_pat_idents(e, out);
            }
        }
        Pat::Object(o) => {
            for p in &o.props {
                match p {
                    ObjectPatProp::KeyValue(kv) => collect_pat_idents(&kv.value, out),
                    ObjectPatProp::Assign(a) => out.push(a.key.sym.as_str()),
                    ObjectPatProp::Rest(r) => collect_pat_idents(&r.arg, out),
                }
            }
        }
        Pat::Rest(r) => collect_pat_idents(&r.arg, out),
        Pat::Assign(a) => collect_pat_idents(&a.left, out),
        Pat::Expr(_) | Pat::Invalid(_) => {}
    }
}

fn collect_var_declared_names_from_block<'a>(block: &'a BlockStmt, out: &mut Vec<&'a str>) {
    for stmt in &block.stmts {
        collect_var_declared_names(stmt, out);
    }
}

pub fn collect_var_declared_names<'a>(stmt: &'a Stmt, out: &mut Vec<&'a str>) {
    match stmt {
        Stmt::Decl(Decl::Var(var_decl)) if var_decl.kind == VarDeclKind::Var => {
            for decl in &var_decl.decls {
                collect_pat_idents(&decl.name, out);
            }
        }
        Stmt::Block(block) => collect_var_declared_names_from_block(block, out),
        Stmt::Labeled(labeled) => collect_var_declared_names(&labeled.body, out),
        Stmt::If(if_stmt) => {
            collect_var_declared_names(&if_stmt.cons, out);
            if let Some(alt) = &if_stmt.alt {
                collect_var_declared_names(alt, out);
            }
        }
        Stmt::While(while_stmt) => collect_var_declared_names(&while_stmt.body, out),
        Stmt::DoWhile(do_while) => collect_var_declared_names(&do_while.body, out),
        Stmt::With(with_stmt) => collect_var_declared_names(&with_stmt.body, out),
        Stmt::For(for_stmt) => {
            if let Some(init) = &for_stmt.init {
                if let VarDeclOrExpr::VarDecl(var_decl) = init {
                    if var_decl.kind == VarDeclKind::Var {
                        for decl in &var_decl.decls {
                            collect_pat_idents(&decl.name, out);
                        }
                    }
                }
            }
            collect_var_declared_names(&for_stmt.body, out);
        }
        Stmt::ForIn(for_in) => {
            if let ForHead::VarDecl(var_decl) = &for_in.left {
                if var_decl.kind == VarDeclKind::Var {
                    for decl in &var_decl.decls {
                        collect_pat_idents(&decl.name, out);
                    }
                }
            }
            collect_var_declared_names(&for_in.body, out);
        }
        Stmt::ForOf(for_of) => {
            if let ForHead::VarDecl(var_decl) = &for_of.left {
                if var_decl.kind == VarDeclKind::Var {
                    for decl in &var_decl.decls {
                        collect_pat_idents(&decl.name, out);
                    }
                }
            }
            collect_var_declared_names(&for_of.body, out);
        }
        Stmt::Switch(switch_stmt) => {
            for case in &switch_stmt.cases {
                for cons in &case.cons {
                    collect_var_declared_names(cons, out);
                }
            }
        }
        Stmt::Try(try_stmt) => {
            collect_var_declared_names_from_block(&try_stmt.block, out);
            if let Some(handler) = &try_stmt.handler {
                collect_var_declared_names_from_block(&handler.body, out);
            }
            if let Some(finalizer) = &try_stmt.finalizer {
                collect_var_declared_names_from_block(finalizer, out);
            }
        }
        _ => {}
    }
}

pub fn collect_lexical_names<'a>(stmt: &'a Stmt, out: &mut Vec<&'a str>) {
    if let Stmt::Decl(decl) = stmt {
        match decl {
            Decl::Var(var_decl) if var_decl.kind != VarDeclKind::Var => {
                for declarator in &var_decl.decls {
                    collect_pat_idents(&declarator.name, out);
                }
            }
            Decl::Class(class_decl) => out.push(class_decl.ident.sym.as_str()),
            Decl::Fn(fn_decl) => out.push(fn_decl.ident.sym.as_str()),
            Decl::Using(using_decl) => {
                for declarator in &using_decl.decls {
                    collect_pat_idents(&declarator.name, out);
                }
            }
            _ => {}
        }
    }
}

impl<'a> Validator<'a> {
    pub fn validate_block(&mut self, block: &'a BlockStmt) -> Result<(), String> {
        self.validate_block_with_shadow(block, true)
    }

    pub fn validate_block_with_shadow(
        &mut self,
        block: &'a BlockStmt,
        allow_param_shadow: bool,
    ) -> Result<(), String> {
        let guard = self.enter_block_scope(allow_param_shadow);

        let result = (|| {
            let mut lexical = Vec::new();
            let mut var_names = Vec::new();

            for stmt in &block.stmts {
                collect_lexical_names(stmt, &mut lexical);
                collect_var_declared_names(stmt, &mut var_names);
            }

            let mut seen = HashSet::new();
            for name in &lexical {
                if *name == "_" {
                    continue;
                }
                if !seen.insert(*name) {
                    return Err(format!(
                        "Identifier '{name}' has already been declared in this block"
                    ));
                }
            }

            if !lexical.is_empty() && !var_names.is_empty() {
                let set_lex: HashSet<&str> = lexical
                    .iter()
                    .copied()
                    .filter(|name| *name != "_")
                    .collect();
                if let Some(name) = var_names
                    .iter()
                    .copied()
                    .filter(|name| *name != "_")
                    .find(|name| set_lex.contains(name))
                {
                    return Err(format!(
                        "Identifier '{name}' conflicts with lexical declaration in this block"
                    ));
                }
            }

            self.validate_statements(&block.stmts)
        })();

        guard.exit(self);
        result
    }
}
