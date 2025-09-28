use crate::Validator;
use std::collections::HashSet;
use swc_ecma_ast::{BlockStmt, Decl, ObjectPatProp, Pat, Stmt, VarDeclKind};

fn collect_pat_idents<'a>(pat: &'a Pat, out: &mut Vec<&'a str>) {
    match pat {
        Pat::Ident(i) => out.push(&i.id.sym),
        Pat::Array(a) => {
            for e in a.elems.iter().flatten() {
                collect_pat_idents(e, out);
            }
        }
        Pat::Object(o) => {
            for p in &o.props {
                match p {
                    ObjectPatProp::KeyValue(kv) => collect_pat_idents(&kv.value, out),
                    ObjectPatProp::Assign(a) => out.push(&a.key.sym),
                    ObjectPatProp::Rest(r) => collect_pat_idents(&r.arg, out),
                }
            }
        }
        Pat::Rest(r) => collect_pat_idents(&r.arg, out),
        Pat::Assign(a) => collect_pat_idents(&a.left, out),
        Pat::Expr(_) | Pat::Invalid(_) => {}
    }
}

impl Validator {
    pub fn validate_block(block: &BlockStmt) -> Result<(), String> {
        let mut lexical: Vec<&str> = Vec::new();
        let mut var_names: Vec<&str> = Vec::new();

        for stmt in &block.stmts {
            if let Stmt::Decl(decl) = stmt {
                match decl {
                    Decl::Var(v) => {
                        for d in &v.decls {
                            let mut names = Vec::new();
                            collect_pat_idents(&d.name, &mut names);
                            if v.kind == VarDeclKind::Var {
                                var_names.extend(names);
                            } else {
                                lexical.extend(names);
                            }
                        }
                    }
                    Decl::Class(c) => {
                        lexical.push(&c.ident.sym);
                    }
                    Decl::Fn(f) => {
                        lexical.push(&f.ident.sym);
                    }
                    _ => {}
                }
            }
        }

        let mut seen = HashSet::new();
        for &name in &lexical {
            if name == "_" {
                continue;
            }
            if !seen.insert(name) {
                return Err(format!(
                    "Identifier '{name}' has already been declared in this block"
                ));
            }
        }

        if !lexical.is_empty() && !var_names.is_empty() {
            let set_lex: HashSet<&str> = lexical.iter().copied().collect();
            let mut reported: Option<String> = None;
            for &v in &var_names {
                if v == "_" {
                    continue;
                }
                if set_lex.contains(v) {
                    reported = Some(v.to_string());
                    break;
                }
            }
            if let Some(name) = reported {
                return Err(format!(
                    "Identifier '{name}' conflicts with lexical declaration in this block"
                ));
            }
        }

        Self::validate_statements(&block.stmts)
    }
}
