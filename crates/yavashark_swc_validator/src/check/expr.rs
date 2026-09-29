use super::Checker;
use super::function::FunctionKind;
use crate::{Result, ValidationError};
use swc_common::Spanned;
use swc_ecma_ast::{
    AssignTarget, AssignTargetPat, BinaryOp, Callee, Expr, Lit, MemberExpr, MemberProp,
    MetaPropKind, MethodKind, OptChainBase, Prop, PropName, PropOrSpread, SimpleAssignTarget,
    SuperProp, SuperPropExpr, UnaryOp,
};

impl<'a> Checker<'a, '_> {
    pub(super) fn validate_expr(&mut self, expr: &'a Expr) -> Result<'a> {
        self.depth += 1;
        let result = if self.depth.is_multiple_of(16) {
            stacker::maybe_grow(256 * 1024, 2 * 1024 * 1024, || {
                self.validate_expr_inner(expr)
            })
        } else {
            self.validate_expr_inner(expr)
        };
        self.depth -= 1;
        result
    }

    pub(super) fn validate_expr_inner(&mut self, expr: &'a Expr) -> Result<'a> {
        match expr {
            Expr::Ident(ident) => self.validate_ident(ident)?,
            Expr::Lit(lit) => match lit {
                Lit::Regex(r) => crate::regexp::Parser::validate(r, self.arena)?,
                Lit::Num(n) if self.ctx.strict => {
                    if n.raw.as_ref().is_some_and(|r| {
                        let b = r.as_bytes();
                        b.len() > 1 && b[0] == b'0' && b[1].is_ascii_digit()
                    }) {
                        return Err(ValidationError::new(
                            "Legacy numeric literal in strict code",
                            n.span,
                        ));
                    }
                }
                Lit::Str(s) => {
                    if s.raw
                        .as_ref()
                        .is_some_and(|r| !Self::valid_unicode_escapes(r.as_str()))
                    {
                        return Err(ValidationError::new(
                            "Invalid unicode string escape",
                            s.span,
                        ));
                    }

                    if self.ctx.strict
                        && s.raw
                            .as_ref()
                            .is_some_and(|r| Self::legacy_escape(r.as_str()))
                    {
                        return Err(ValidationError::new(
                            "Legacy string escape in strict code",
                            s.span,
                        ));
                    }
                }
                _ => {}
            },
            Expr::Array(array) => {
                for expr in array.elems.iter().flatten() {
                    self.validate_expr(&expr.expr)?;
                }
            }
            Expr::Object(object) => {
                let mut proto = false;

                for p in &object.props {
                    match p {
                        PropOrSpread::Spread(s) => self.validate_expr(&s.expr)?,
                        PropOrSpread::Prop(p) => match &**p {
                            Prop::Shorthand(i) => self.validate_ident(i)?,
                            Prop::Assign(a) => {
                                return Err(ValidationError::new(
                                    "Assignment property in object literal",
                                    a.span,
                                ));
                            }
                            Prop::KeyValue(k) => {
                                self.validate_key(&k.key)?;

                                if Self::key_is(&k.key, "__proto__") {
                                    if proto {
                                        return Err(ValidationError::new(
                                            "Duplicate prototype setter",
                                            k.key.span(),
                                        ));
                                    }
                                    proto = true;
                                }

                                self.validate_expr(&k.value)?;
                            }
                            Prop::Method(m) => {
                                self.validate_key(&m.key)?;
                                self.validate_function(&m.function, None, FunctionKind::Method)?;
                            }
                            Prop::Getter(g) => {
                                self.validate_key(&g.key)?;
                                Self::validate_method_arity(&g.function, MethodKind::Getter)?;
                                self.validate_function(&g.function, None, FunctionKind::Method)?;
                            }
                            Prop::Setter(s) => {
                                self.validate_key(&s.key)?;
                                Self::validate_method_arity(&s.function, MethodKind::Setter)?;
                                self.validate_function(&s.function, None, FunctionKind::Method)?;
                            }
                        },
                    }
                }
            }
            Expr::Fn(function) => self.validate_function(
                &function.function,
                function.ident.as_ref(),
                FunctionKind::Expression,
            )?,
            Expr::Arrow(arrow) => self.validate_arrow(arrow)?,
            Expr::Class(class) => self.validate_class(&class.class, class.ident.as_ref())?,
            Expr::Unary(unary) => {
                if unary.op == UnaryOp::Delete {
                    match Self::unparen(&unary.arg) {
                        Expr::Ident(_) if self.ctx.strict => {
                            return Err(ValidationError::new(
                                "Delete of identifier in strict code",
                                unary.span,
                            ));
                        }
                        Expr::Member(m) if matches!(m.prop, MemberProp::PrivateName(_)) => {
                            return Err(ValidationError::new(
                                "Delete of private member",
                                unary.span,
                            ));
                        }
                        _ => {}
                    }
                }

                self.validate_expr(&unary.arg)?;
            }
            Expr::Update(update) => self.validate_assignment_target(&update.arg)?,
            Expr::Bin(binary) => {
                if let Expr::PrivateName(p) = &*binary.left {
                    if binary.op != BinaryOp::In {
                        return Err(ValidationError::new(
                            "Private name requires in operator",
                            p.span,
                        ));
                    }

                    self.validate_private_name(p)?;
                } else {
                    self.validate_expr(&binary.left)?;
                }

                self.validate_expr(&binary.right)?;
            }
            Expr::Assign(assign) => {
                match &assign.left {
                    AssignTarget::Simple(t) => match t {
                        SimpleAssignTarget::Ident(i) => self.validate_binding(&i.id)?,
                        SimpleAssignTarget::Member(m) => self.validate_member(m)?,
                        SimpleAssignTarget::SuperProp(s) => self.validate_super_property(s)?,
                        SimpleAssignTarget::Paren(p) => self.validate_assignment_target(&p.expr)?,
                        _ => {
                            return Err(ValidationError::new(
                                "Invalid assignment target",
                                assign.span,
                            ));
                        }
                    },
                    AssignTarget::Pat(p) => match p {
                        AssignTargetPat::Array(p) => self.validate_array_assignment(p)?,
                        AssignTargetPat::Object(p) => self.validate_object_assignment(p)?,
                        AssignTargetPat::Invalid(p) => {
                            return Err(ValidationError::new("Invalid assignment pattern", p.span));
                        }
                    },
                }

                self.validate_expr(&assign.right)?;
            }
            Expr::Member(member) => self.validate_member(member)?,
            Expr::SuperProp(super_prop) => self.validate_super_property(super_prop)?,
            Expr::Cond(conditional) => {
                self.validate_expr(&conditional.test)?;
                self.validate_expr(&conditional.cons)?;
                self.validate_expr(&conditional.alt)?;
            }
            Expr::Call(call) => {
                match &call.callee {
                    Callee::Expr(expr) => self.validate_expr(expr)?,
                    Callee::Super(s) => {
                        if !self.ctx.super_call {
                            return Err(ValidationError::new(
                                "Super call outside derived constructor",
                                s.span,
                            ));
                        }
                    }
                    Callee::Import(i) => {
                        if !(1..=2).contains(&call.args.len())
                            || call.args.iter().any(|a| a.spread.is_some())
                        {
                            return Err(ValidationError::new("Invalid import arguments", i.span));
                        }
                    }
                }

                for a in &call.args {
                    self.validate_expr(&a.expr)?;
                }
            }
            Expr::New(new) => {
                if Self::import_chain(&new.callee) {
                    return Err(ValidationError::new(
                        "Import call cannot be a new expression",
                        new.span,
                    ));
                }

                self.validate_expr(&new.callee)?;

                if let Some(args) = &new.args {
                    for a in args {
                        self.validate_expr(&a.expr)?;
                    }
                }
            }
            Expr::Seq(sequence) => {
                for expr in &sequence.exprs {
                    self.validate_expr(expr)?;
                }
            }
            Expr::Tpl(template) => {
                for q in &template.quasis {
                    if q.cooked.is_none()
                        || Self::legacy_escape(&q.raw)
                        || !Self::valid_unicode_escapes(&q.raw)
                    {
                        return Err(ValidationError::new(
                            "Invalid untagged template escape",
                            q.span,
                        ));
                    }
                }

                for expr in &template.exprs {
                    self.validate_expr(expr)?;
                }
            }
            Expr::TaggedTpl(tagged) => {
                self.validate_expr(&tagged.tag)?;

                for expr in &tagged.tpl.exprs {
                    self.validate_expr(expr)?;
                }
            }
            Expr::Yield(yield_expr) => {
                if !self.ctx.generator || self.ctx.parameters {
                    return Err(ValidationError::new(
                        "Yield outside generator body",
                        yield_expr.span,
                    ));
                }

                if let Some(expr) = &yield_expr.arg {
                    self.validate_expr(expr)?;
                }
            }
            Expr::Await(await_expr) => {
                if (!self.ctx.asynchronous && (!self.ctx.module || self.ctx.function))
                    || self.ctx.parameters
                    || self.ctx.static_block
                {
                    return Err(ValidationError::new(
                        "Await outside async body",
                        await_expr.span,
                    ));
                }

                self.validate_expr(&await_expr.arg)?;
            }
            Expr::MetaProp(meta) => match meta.kind {
                MetaPropKind::NewTarget if !self.ctx.new_target => {
                    return Err(ValidationError::new(
                        "New.target outside function",
                        meta.span,
                    ));
                }
                MetaPropKind::ImportMeta if !self.ctx.module => {
                    return Err(ValidationError::new(
                        "Import.meta outside module",
                        meta.span,
                    ));
                }
                _ => {}
            },
            Expr::Paren(paren) => self.validate_expr(&paren.expr)?,
            Expr::OptChain(chain) => match &*chain.base {
                OptChainBase::Member(m) => self.validate_member(m)?,
                OptChainBase::Call(c) => {
                    self.validate_expr(&c.callee)?;

                    for a in &c.args {
                        self.validate_expr(&a.expr)?;
                    }
                }
            },
            Expr::PrivateName(private) => {
                return Err(ValidationError::new(
                    "Private name is not an expression",
                    private.span,
                ));
            }
            Expr::This(_) => {}
            _ => {
                return Err(ValidationError::new(
                    "Non-ECMAScript expression",
                    expr.span(),
                ));
            }
        }

        Ok(())
    }

    pub(super) fn validate_assignment_target(&mut self, expr: &'a Expr) -> Result<'a> {
        match expr {
            Expr::Ident(ident) => self.validate_binding(ident),
            Expr::Member(member) => self.validate_member(member),
            Expr::SuperProp(super_prop) => self.validate_super_property(super_prop),
            Expr::Paren(paren) => self.validate_assignment_target(&paren.expr),
            _ => Err(ValidationError::new(
                "Invalid assignment target",
                expr.span(),
            )),
        }
    }

    pub(super) fn validate_member(&mut self, member: &'a MemberExpr) -> Result<'a> {
        self.validate_expr(&member.obj)?;

        match &member.prop {
            MemberProp::Computed(c) => self.validate_expr(&c.expr)?,
            MemberProp::PrivateName(p) => self.validate_private_name(p)?,
            MemberProp::Ident(_) => {}
        }

        Ok(())
    }

    pub(super) fn validate_super_property(&mut self, super_prop: &'a SuperPropExpr) -> Result<'a> {
        if !self.ctx.super_prop {
            return Err(ValidationError::new(
                "Super property outside method",
                super_prop.span,
            ));
        }

        if let SuperProp::Computed(c) = &super_prop.prop {
            self.validate_expr(&c.expr)?;
        }

        Ok(())
    }

    pub(super) fn validate_key(&mut self, key: &'a PropName) -> Result<'a> {
        if let PropName::Ident(i) = key {
            Self::identifier_name(&i.sym).map_err(|m| ValidationError::new(m, i.span))?;
        }

        if let PropName::Computed(c) = key {
            self.validate_expr(&c.expr)?;
        }

        Ok(())
    }
}
