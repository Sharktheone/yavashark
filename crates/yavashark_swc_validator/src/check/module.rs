use super::function::FunctionKind;
use super::{Checker, LEX, ScopeKind};
use crate::{Result, ValidationError};
use swc_common::Spanned;
use swc_ecma_ast::{
    Decl, DefaultDecl, ExportSpecifier, Expr, Ident, ImportSpecifier, Lit, ModuleDecl,
    ModuleExportName, ModuleItem, ObjectLit, Prop, PropName, PropOrSpread,
};

impl<'a> Checker<'a, '_> {
    pub(crate) fn validate_module(&mut self, items: &'a [ModuleItem]) -> Result<'a> {
        self.push_scope(ScopeKind::Module);

        for item in items {
            match item {
                ModuleItem::Stmt(s) => self.validate_statement(s)?,
                ModuleItem::ModuleDecl(d) => self.validate_module_decl(d)?,
            }
        }

        let mut exports = hashbrown::HashSet::new_in(self.arena);

        for item in items {
            let ModuleItem::ModuleDecl(d) = item else {
                continue;
            };

            match d {
                ModuleDecl::ExportDefaultDecl(_) | ModuleDecl::ExportDefaultExpr(_) => {
                    if !exports.insert(b"default".as_slice()) {
                        return Err(ValidationError::new("Duplicate export", d.span()));
                    }
                }
                ModuleDecl::ExportDecl(e) => {
                    let mut add = |i: &'a Ident| {
                        if exports.insert(i.sym.as_bytes()) {
                            Ok(())
                        } else {
                            Err(ValidationError::new("Duplicate export", i.span))
                        }
                    };

                    match &e.decl {
                        Decl::Var(v) => {
                            for d in &v.decls {
                                Self::each_name(&d.name, &mut add)?;
                            }
                        }
                        Decl::Fn(f) => add(&f.ident)?,
                        Decl::Class(c) => add(&c.ident)?,
                        _ => {}
                    }
                }
                ModuleDecl::ExportNamed(e) => {
                    for spec in &e.specifiers {
                        let name = match spec {
                            ExportSpecifier::Named(n) => n.exported.as_ref().unwrap_or(&n.orig),
                            ExportSpecifier::Namespace(n) => &n.name,
                            ExportSpecifier::Default(n) => {
                                if !exports.insert(n.exported.sym.as_bytes()) {
                                    return Err(ValidationError::new(
                                        "Duplicate export",
                                        n.exported.span,
                                    ));
                                }
                                continue;
                            }
                        };

                        if Self::export_str(name).is_none() {
                            return Err(ValidationError::new(
                                "Export name contains an unpaired surrogate",
                                name.span(),
                            ));
                        }

                        if !exports.insert(Self::export_key(name)) {
                            return Err(ValidationError::new("Duplicate export", name.span()));
                        }

                        if let ExportSpecifier::Named(n) = spec {
                            if Self::export_str(&n.orig).is_none() {
                                return Err(ValidationError::new(
                                    "Export name contains an unpaired surrogate",
                                    n.span,
                                ));
                            }

                            if e.src.is_none() {
                                if let ModuleExportName::Ident(i) = &n.orig {
                                    if !self.scopes[0].names.contains_key(i.sym.as_str()) {
                                        return Err(Self::binding_error(
                                            "Export of undeclared binding",
                                            &i.sym,
                                            i.span,
                                        ));
                                    }
                                } else {
                                    return Err(ValidationError::new(
                                        "Local export requires an identifier",
                                        n.span,
                                    ));
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub(super) fn validate_module_decl(&mut self, decl: &'a ModuleDecl) -> Result<'a> {
        match decl {
            ModuleDecl::Import(i) => {
                for s in &i.specifiers {
                    if let ImportSpecifier::Named(n) = s
                        && n.imported
                            .as_ref()
                            .is_some_and(|n| Self::export_str(n).is_none())
                    {
                        return Err(ValidationError::new(
                            "Import name contains an unpaired surrogate",
                            n.span,
                        ));
                    }

                    self.declare_binding(s.local(), LEX)?;
                }

                if let Some(w) = &i.with {
                    self.validate_attributes(w)?;
                }
            }
            ModuleDecl::ExportDecl(e) => self.validate_decl(&e.decl)?,
            ModuleDecl::ExportDefaultExpr(e) => self.validate_expr(&e.expr)?,
            ModuleDecl::ExportDefaultDecl(e) => match &e.decl {
                DefaultDecl::Fn(f) => {
                    if let Some(i) = &f.ident {
                        self.declare_binding(i, LEX)?;
                    }

                    self.validate_function(
                        &f.function,
                        f.ident.as_ref(),
                        FunctionKind::Declaration,
                    )?;
                }
                DefaultDecl::Class(c) => {
                    if let Some(i) = &c.ident {
                        self.declare_binding(i, LEX)?;
                    }

                    self.validate_class(&c.class, c.ident.as_ref())?;
                }
                DefaultDecl::TsInterfaceDecl(_) => {
                    return Err(ValidationError::new("Non-ECMAScript export", e.span));
                }
            },
            ModuleDecl::ExportNamed(e) => {
                if let Some(w) = &e.with {
                    self.validate_attributes(w)?;
                }
            }
            ModuleDecl::ExportAll(e) => {
                if let Some(w) = &e.with {
                    self.validate_attributes(w)?;
                }
            }
            _ => {
                return Err(ValidationError::new(
                    "Non-ECMAScript module declaration",
                    decl.span(),
                ));
            }
        }

        Ok(())
    }

    pub(super) fn validate_attributes(&self, attributes: &'a ObjectLit) -> Result<'a> {
        let mut names = hashbrown::HashSet::new_in(self.arena);

        for p in &attributes.props {
            if let PropOrSpread::Prop(p) = p {
                if let Prop::KeyValue(k) = &**p {
                    let name = match &k.key {
                        PropName::Ident(i) => i.sym.as_bytes(),
                        PropName::Str(s) => s.value.as_bytes(),
                        _ => {
                            return Err(ValidationError::new(
                                "Invalid import attribute key",
                                k.key.span(),
                            ));
                        }
                    };

                    if !names.insert(name) {
                        return Err(ValidationError::new(
                            "Duplicate import attribute",
                            k.key.span(),
                        ));
                    }

                    if !matches!(&*k.value, Expr::Lit(Lit::Str(_))) {
                        return Err(ValidationError::new(
                            "Import attribute must be a string",
                            k.value.span(),
                        ));
                    }
                } else {
                    return Err(ValidationError::new("Invalid import attribute", p.span()));
                }
            } else {
                return Err(ValidationError::new("Spread import attribute", p.span()));
            }
        }

        Ok(())
    }
}
