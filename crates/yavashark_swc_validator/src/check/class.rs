use super::function::FunctionKind;
use super::{Checker, Context, Map, ScopeKind};
use crate::{Result, ValidationError};
use swc_common::Spanned;
use swc_ecma_ast::{
    Class, ClassMember, Expr, Ident, Key, MethodKind, Param, ParamOrTsParamProp, Pat, PrivateName,
};

impl<'a> Checker<'a, '_> {
    pub(super) fn validate_class(
        &mut self,
        class: &'a Class,
        name: Option<&'a Ident>,
    ) -> Result<'a> {
        let previous_context = self.ctx;
        self.ctx.strict = true;

        if let Some(n) = name {
            self.validate_binding(n)?;
        }

        if let Some(e) = &class.super_class {
            self.validate_expr(e)?;
        }

        // Arena-backed rehashing retains old buffers until validation finishes.
        // Count only private members so large public classes reserve nothing.
        let private_count = class.body.iter().filter(|member| {
            matches!(member, ClassMember::PrivateProp(_) | ClassMember::PrivateMethod(_))
                || matches!(member, ClassMember::AutoAccessor(accessor) if matches!(accessor.key, Key::Private(_)))
        }).count();
        let mut private = Map::with_capacity_in(private_count, self.arena);
        let mut constructor = false;

        for member in &class.body {
            let entry = match member {
                ClassMember::Constructor(k) => {
                    if constructor {
                        return Err(ValidationError::new("Duplicate constructor", k.span));
                    }
                    constructor = true;
                    None
                }
                ClassMember::PrivateProp(p) => Some((&p.key, 1, p.is_static)),
                ClassMember::PrivateMethod(m) => Some((
                    &m.key,
                    match m.kind {
                        MethodKind::Getter => 2,
                        MethodKind::Setter => 4,
                        MethodKind::Method => 1,
                    },
                    m.is_static,
                )),
                ClassMember::AutoAccessor(a) => {
                    if let Key::Private(p) = &a.key {
                        Some((p, 1, a.is_static))
                    } else {
                        None
                    }
                }
                _ => None,
            };

            if let Some((p, kind, static_)) = entry {
                Self::identifier_name(&p.name).map_err(|m| ValidationError::new(m, p.span))?;

                if p.name == *"constructor" {
                    return Err(ValidationError::new("Private constructor name", p.span));
                }

                let bits = kind | if static_ { 8 } else { 0 };

                if let Some(prev) = private.get_mut(p.name.as_str()) {
                    if *prev & 1 != 0 || kind == 1 || *prev & kind != 0 || (*prev & 8) != (bits & 8)
                    {
                        return Err(ValidationError::new("Duplicate private name", p.span));
                    }
                    *prev |= bits;
                } else {
                    private.insert(p.name.as_str(), bits);
                }
            }
        }

        self.private.push(private);

        for member in &class.body {
            match member {
                ClassMember::Constructor(k) => {
                    let class_context = self.ctx;
                    self.ctx = Context {
                        strict: true,
                        module: previous_context.module,
                        function: true,
                        super_prop: true,
                        super_call: class.super_class.is_some(),
                        new_target: true,
                        label_base: self.labels.len(),
                        ..Context::default()
                    };
                    self.push_scope(ScopeKind::Function);
                    self.ctx.parameters = true;

                    for p in &k.params {
                        if let ParamOrTsParamProp::Param(p) = p {
                            self.validate_parameter(&p.pat, true)?;
                        } else {
                            return Err(ValidationError::new(
                                "TypeScript parameter property",
                                k.span,
                            ));
                        }
                    }

                    self.ctx.parameters = false;

                    if let Some(b) = &k.body {
                        if Self::strict_directive(&b.stmts)
                            && k.params.iter().any(|p| {
                                !matches!(
                                    p,
                                    ParamOrTsParamProp::Param(Param {
                                        pat: Pat::Ident(_),
                                        ..
                                    })
                                )
                            })
                        {
                            return Err(ValidationError::new(
                                "Strict directive with non-simple parameters",
                                k.span,
                            ));
                        }

                        self.validate_statements(&b.stmts)?;
                    }

                    self.scopes.pop();
                    self.ctx = class_context;
                }
                ClassMember::Method(m) => {
                    self.validate_key(&m.key)?;

                    if m.is_static && Self::key_is(&m.key, "prototype") {
                        return Err(ValidationError::new("Static prototype method", m.span));
                    }

                    Self::validate_method_arity(&m.function, m.kind)?;
                    self.validate_function(&m.function, None, FunctionKind::Method)?;
                }
                ClassMember::PrivateMethod(m) => {
                    Self::validate_method_arity(&m.function, m.kind)?;
                    self.validate_function(&m.function, None, FunctionKind::Method)?;
                }
                ClassMember::ClassProp(p) => {
                    self.validate_key(&p.key)?;

                    if Self::key_is(&p.key, "constructor")
                        || p.is_static && Self::key_is(&p.key, "prototype")
                    {
                        return Err(ValidationError::new("Invalid field name", p.span));
                    }

                    if let Some(e) = &p.value {
                        self.validate_initializer(e)?;
                    }
                }
                ClassMember::PrivateProp(p) => {
                    if let Some(e) = &p.value {
                        self.validate_initializer(e)?;
                    }
                }
                ClassMember::AutoAccessor(a) => {
                    if let Key::Public(k) = &a.key {
                        self.validate_key(k)?;
                    }

                    if let Some(e) = &a.value {
                        self.validate_initializer(e)?;
                    }
                }
                ClassMember::StaticBlock(b) => {
                    let before = self.ctx;
                    self.ctx = Context {
                        strict: true,
                        module: previous_context.module,
                        super_prop: true,
                        new_target: true,
                        no_arguments: true,
                        static_block: true,
                        label_base: self.labels.len(),
                        ..Context::default()
                    };
                    self.push_scope(ScopeKind::Function);
                    self.validate_statements(&b.body.stmts)?;
                    self.scopes.pop();
                    self.ctx = before;
                }
                ClassMember::Empty(_) => {}
                ClassMember::TsIndexSignature(_) => {
                    return Err(ValidationError::new(
                        "Non-ECMAScript class member",
                        member.span(),
                    ));
                }
            }
        }

        self.private.pop();
        self.ctx = previous_context;

        Ok(())
    }

    pub(super) fn validate_initializer(&mut self, expr: &'a Expr) -> Result<'a> {
        let previous_context = self.ctx;
        self.ctx = Context {
            strict: true,
            module: previous_context.module,
            super_prop: true,
            new_target: true,
            no_arguments: true,
            label_base: self.labels.len(),
            ..Context::default()
        };
        self.validate_expr(expr)?;
        self.ctx = previous_context;

        Ok(())
    }

    pub(super) fn validate_private_name(&self, private: &'a PrivateName) -> Result<'a> {
        if !self
            .private
            .iter()
            .rev()
            .any(|s| s.contains_key(private.name.as_str()))
        {
            return Err(Self::binding_error(
                "Undeclared private name",
                &private.name,
                private.span,
            ));
        }

        Ok(())
    }
}
