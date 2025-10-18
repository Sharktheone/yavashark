use std::iter;
use swc_common::Spanned;
use swc_ecma_ast::{
    AssignExpr, AssignOp, AssignTarget, AssignTargetPat, ExportDefaultExpr, Expr, MemberExpr,
    MemberProp, OptChainBase, OptChainExpr, ParenExpr, Pat, SimpleAssignTarget, SuperProp,
    SuperPropExpr,
};

use crate::Interpreter;
use yavashark_env::scope::Scope;
use yavashark_env::value::property_key::IntoPropertyKey;
use yavashark_env::value::{DefinePropertyResult, Obj, Property};
use yavashark_env::{
    Class, ClassInstance, Error, InternalPropertyKey, PrivateMember, Realm, Res, RuntimeResult,
    Value,
};
use yavashark_string::YSString;

impl Interpreter {
    pub fn run_assign(realm: &mut Realm, stmt: &AssignExpr, scope: &mut Scope) -> RuntimeResult {
        let value = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;

        if stmt.op == AssignOp::Assign {
            return Ok(Self::assign_target(realm, &stmt.left, value.copy(), scope).map(|()| value)?);
        }

        Self::assign_target_op(realm, stmt.op, &stmt.left, value, scope)
    }

    pub fn assign_target(
        realm: &mut Realm,
        target: &AssignTarget,
        value: Value,
        scope: &mut Scope,
    ) -> Res {
        match target {
            AssignTarget::Simple(t) => match t {
                SimpleAssignTarget::Ident(i) => {
                    let name = i.sym.to_string();
                    if scope.is_strict_mode()? && !scope.has_value(&name, realm)? {
                        return Err(Error::reference_error(format!("{name} is not defined")));
                    }
                    scope.update_or_define(name, value, realm)
                }
                SimpleAssignTarget::Member(m) => Self::assign_member(realm, m, value, scope),
                SimpleAssignTarget::SuperProp(super_prop) => {
                    Self::assign_super(realm, super_prop, value, scope)
                }
                SimpleAssignTarget::OptChain(opt) => {
                    Self::assign_opt_chain(realm, opt, value, scope)
                }
                SimpleAssignTarget::Paren(paren) => {
                    Self::assign_expr(realm, &paren.expr, value, scope)
                }

                _ => Err(Error::syn("Invalid left-hand side in assignment")),
            },
            AssignTarget::Pat(pat) => Self::assign_pat(realm, pat, value, scope),
        }
    }

    pub fn assign_member(
        realm: &mut Realm,
        m: &MemberExpr,
        value: Value,
        scope: &mut Scope,
    ) -> Res {
        let obj = Self::run_expr(realm, &m.obj, m.span, scope)?;
        Self::assign_member_on(realm, obj, &m.prop, value, scope)
    }

    pub fn assign_member_on(
        realm: &mut Realm,
        obj: Value,
        m: &MemberProp,
        value: Value,
        scope: &mut Scope,
    ) -> Res {
        if let Value::Object(obj) = obj {
            let name = match m {
                MemberProp::Ident(i) => Value::String(YSString::from_ref(&i.sym)),
                MemberProp::PrivateName(p) => {
                    let name = p.name.as_str();

                    if let Some(class) = obj.downcast::<ClassInstance>() {
                        let member = class
                            .get_private_prop(name, realm)?
                            .ok_or(Error::ty_error(format!("Private name {name} not found")))?;

                        let this_value = Value::Object(obj.clone());

                        Self::write_private_member_on_instance(
                            realm, &*class, name, member, value, this_value,
                        )?;

                        return Ok(());
                    }

                    if let Some(class) = obj.downcast::<Class>() {
                        let member = class
                            .get_private_prop(name)
                            .ok_or(Error::ty_error(format!("Private name {name} not found")))?;

                        let this_value = Value::Object(obj.clone());

                        Self::write_private_member_on_class(
                            realm, &*class, name, member, value, this_value,
                        )?;

                        return Ok(());
                    }

                    return Err(Error::ty_error(format!(
                        "Private name {name} can only be used in class"
                    )));
                }
                MemberProp::Computed(c) => Self::run_expr(realm, &c.expr, c.span, scope)?,
            };

            let key = name.into_internal_property_key(realm)?;

            match obj.define_property(key, value, realm)? {
                DefinePropertyResult::Handled => {},
                DefinePropertyResult::ReadOnly => {
                    if scope.is_strict_mode()? {
                        return Err(Error::ty("Cannot assign to read only property"));
                    }
                }
                DefinePropertyResult::Setter(_, _) => {}
            }
            Ok(())
        } else {
            Err(Error::ty_error(format!(
                "Invalid left-hand side in assignment: {obj}"
            )))
        }
    }

    pub fn assign_super(
        realm: &mut Realm,
        super_prop: &SuperPropExpr,
        value: Value,
        scope: &mut Scope,
    ) -> Res {
        let this = scope.this()?;

        let obj = this.as_object()?;

        let sup = obj.prototype(realm)?.to_object()?;

        match &super_prop.prop {
            SuperProp::Ident(i) => {
                let name = i.sym.to_string();

                sup.define_property(name.into(), value, realm)?;
            }
            SuperProp::Computed(p) => {
                let name = Self::run_expr(realm, &p.expr, super_prop.span, scope)?;

                sup.define_property(name.into_internal_property_key(realm)?, value, realm)?;
            }
        }

        Ok(())
    }

    pub fn assign_opt_chain(
        realm: &mut Realm,
        opt: &OptChainExpr,
        value: Value,
        scope: &mut Scope,
    ) -> Res {
        match &*opt.base {
            OptChainBase::Member(member) => {
                let obj = Self::run_expr(realm, &member.obj, member.span, scope)?;

                if (obj == Value::Undefined || obj == Value::Null) && opt.optional {
                    return Ok(());
                }

                Self::assign_member_on(realm, obj, &member.prop, value, scope)
            }
            OptChainBase::Call(call) => {
                let (callee, this) = Self::run_call_expr(realm, &call.callee, call.span, scope)?;

                if (callee == Value::Undefined || callee == Value::Null) && opt.optional {
                    return Ok(());
                }

                let this = this.unwrap_or(scope.fn_this()?);

                Self::run_call_on(realm, &callee, this, &call.args, call.span, scope)?;
                //TODO: maybe we should throw an error here?

                Ok(())
            }
        }
    }

    pub fn assign_expr(realm: &mut Realm, expr: &Expr, value: Value, scope: &mut Scope) -> Res {
        match expr {
            Expr::Member(m) => {
                let obj = Self::run_expr(realm, &m.obj, m.span, scope)?;
                Self::assign_member_on(realm, obj, &m.prop, value, scope)?;
            }
            Expr::SuperProp(super_prop) => {
                Self::assign_super(realm, super_prop, value, scope)?;
            }
            Expr::OptChain(opt) => {
                Self::assign_opt_chain(realm, opt, value, scope)?;
            }
            Expr::Paren(paren) => Self::assign_expr(realm, &paren.expr, value, scope)?,

            epxr => {
                Self::run_expr(realm, epxr, expr.span(), scope)?;
            }
        }

        Ok(())
    }

    fn assign_pat(
        realm: &mut Realm,
        pat: &AssignTargetPat,
        value: Value,
        scope: &mut Scope,
    ) -> Res {
        match pat {
            AssignTargetPat::Array(arr) => {
                let pat = Pat::Array(arr.clone());

                Self::run_pat(
                    realm,
                    &pat,
                    scope,
                    &mut iter::once(value),
                    &mut |scope, name, value, realm| scope.update_or_define(name, value, realm),
                )?;
            }
            AssignTargetPat::Object(expr) => {
                let pat = Pat::Object(expr.clone());

                Self::run_pat(
                    realm,
                    &pat,
                    scope,
                    &mut iter::once(value),
                    &mut |scope, name, value, realm| scope.update_or_define(name, value, realm),
                )?;
            }
            AssignTargetPat::Invalid(_) => {
                return Err(Error::syn("Invalid left-hand side in assignment"))
            }
        }

        Ok(())
    }

    pub fn assign_target_op(
        realm: &mut Realm,
        op: AssignOp,
        target: &AssignTarget,
        right: Value,
        scope: &mut Scope,
    ) -> RuntimeResult {
        match target {
            AssignTarget::Simple(t) => match t {
                SimpleAssignTarget::Ident(i) => {
                    let name = i.sym.to_string();

                    let left = scope
                        .resolve(&name, realm)?
                        .ok_or_else(|| Error::reference_error(format!("{name} is not defined")))?;

                    let value = Self::run_assign_op(op, left, right, realm)?;

                    scope.update(&name, value.copy(), realm)?;

                    Ok(value)
                }
                SimpleAssignTarget::Member(m) => Self::assign_member_op(realm, op, m, right, scope),
                SimpleAssignTarget::SuperProp(super_prop) => {
                    Self::assign_super_op(realm, op, super_prop, right, scope)
                }
                SimpleAssignTarget::OptChain(opt) => {
                    Self::assign_opt_chain_op(realm, op, opt, right, scope)
                }
                SimpleAssignTarget::Paren(paren) => {
                    Self::assign_expr_op(realm, op, &paren.expr, right, scope)
                }
                _ => Err(Error::syn("Invalid left-hand side in assignment").into()),
            },
            AssignTarget::Pat(pat) => Self::assign_pat_op(realm, op, pat, right, scope),
        }
    }

    pub fn assign_member_op(
        realm: &mut Realm,
        op: AssignOp,
        m: &MemberExpr,
        right: Value,
        scope: &mut Scope,
    ) -> RuntimeResult {
        let obj = Self::run_expr(realm, &m.obj, m.span, scope)?;

        Self::assign_member_op_on(realm, obj, op, &m.prop, right, scope)
    }

    pub fn assign_member_op_on(
        realm: &mut Realm,
        obj: Value,
        op: AssignOp,
        m: &MemberProp,
        right: Value,
        scope: &mut Scope,
    ) -> RuntimeResult {
        if let Value::Object(obj) = obj {
            let name = match m {
                MemberProp::Ident(i) => Value::String(YSString::from_ref(&i.sym)),
                MemberProp::PrivateName(p) => {
                    let name = p.name.as_str();

                    if let Some(class) = obj.downcast::<ClassInstance>() {
                        let member = class
                            .get_private_prop(name, realm)?
                            .ok_or(Error::ty_error(format!("Private name {name} not found")))?;

                        if matches!(member, PrivateMember::Method(_)) {
                            return Err(Error::ty_error(format!(
                                "Cannot assign to private method {name}"
                            ))
                            .into());
                        }

                        let this_value = Value::Object(obj.clone());

                        let left =
                            Self::resolve_private_member(realm, member.clone(), this_value.copy())?
                                .0;

                        let value = Self::run_assign_op(op, left, right, realm)?;

                        Self::write_private_member_on_instance(
                            realm,
                            &*class,
                            name,
                            member,
                            value.copy(),
                            this_value,
                        )?;

                        return Ok(value);
                    }

                    if let Some(class) = obj.downcast::<Class>() {
                        let member = class
                            .get_private_prop(name)
                            .ok_or(Error::ty_error(format!("Private name {name} not found")))?;

                        if matches!(member, PrivateMember::Method(_)) {
                            return Err(Error::ty_error(format!(
                                "Cannot assign to private method {name}"
                            ))
                            .into());
                        }

                        let this_value = Value::Object(obj.clone());

                        let left =
                            Self::resolve_private_member(realm, member.clone(), this_value.copy())?
                                .0;

                        let value = Self::run_assign_op(op, left, right, realm)?;

                        Self::write_private_member_on_class(
                            realm,
                            &*class,
                            name,
                            member,
                            value.copy(),
                            this_value,
                        )?;

                        return Ok(value);
                    }

                    return Err(Error::ty_error(format!(
                        "Private name {name} can only be used in class"
                    ))
                    .into());
                }
                MemberProp::Computed(c) => Self::run_expr(realm, &c.expr, c.span, scope)?,
            };

            let name = name.into_internal_property_key(realm)?;

            let (left, writable) = (if let Some(v) = obj
                .resolve_property_no_get_set(name.clone(), realm)? {
                match v {
                    Property::Value(v, a) => (v, a.is_writable()),
                    Property::Getter(get, _) => (get.call(Vec::new(), scope.fn_this()?, realm)?, false),
                }
            } else {
                if scope.is_strict_mode()? {
                    return Err(Error::ty_error(format!(
                        "Property {name:?} does not exist on object",
                    )).into());
                }

                (Value::Undefined, true)
            });

            if !writable && !matches!(op, AssignOp::OrAssign | AssignOp::AndAssign | AssignOp::NullishAssign) && scope.is_strict_mode()? {
                return Err(Error::ty("Cannot assign to read only property").into());
            }

            let value = Self::run_assign_op(op, left, right, realm)?;

            obj.define_property(name, value.copy(), realm);
            Ok(value)
        } else {
            Err(Error::ty_error(format!("Invalid left-hand side in assignment: {obj}")).into())
        }
    }

    pub fn assign_super_op(
        realm: &mut Realm,
        op: AssignOp,
        super_prop: &SuperPropExpr,
        right: Value,
        scope: &mut Scope,
    ) -> RuntimeResult {
        let this = scope.this()?;

        let obj = this.as_object()?;

        let sup = obj.prototype(realm)?.to_object()?;

        match &super_prop.prop {
            SuperProp::Ident(i) => {
                let name: InternalPropertyKey = i.sym.to_string().into();

                let left = sup
                    .resolve_property(name.clone(), realm)?
                    .unwrap_or(Value::Undefined);

                let value = Self::run_assign_op(op, left, right, realm)?;

                sup.define_property(name, value.copy(), realm);
                Ok(value)
            }
            SuperProp::Computed(p) => {
                let name = Self::run_expr(realm, &p.expr, super_prop.span, scope)?;

                let name = name.into_internal_property_key(realm)?;

                let left = sup
                    .resolve_property(name.clone(), realm)?
                    .unwrap_or(Value::Undefined);

                let value = Self::run_assign_op(op, left, right, realm)?;

                sup.define_property(name, value.copy(), realm);
                Ok(value)
            }
        }
    }

    pub fn assign_opt_chain_op(
        realm: &mut Realm,
        op: AssignOp,
        opt: &OptChainExpr,
        right: Value,
        scope: &mut Scope,
    ) -> RuntimeResult {
        match &*opt.base {
            OptChainBase::Member(member) => {
                let obj = Self::run_expr(realm, &member.obj, member.span, scope)?;

                if (obj == Value::Undefined || obj == Value::Null) && opt.optional {
                    return Ok(right);
                }

                Self::assign_member_op_on(realm, obj, op, &member.prop, right, scope)
            }
            OptChainBase::Call(call) => {
                let (callee, this) = Self::run_call_expr(realm, &call.callee, call.span, scope)?;

                if (callee == Value::Undefined || callee == Value::Null) && opt.optional {
                    return Ok(right);
                }

                let this = this.unwrap_or(scope.fn_this()?);

                let left = Self::run_call_on(realm, &callee, this, &call.args, call.span, scope)?;
                //TODO: maybe we should throw an error here?

                let value = Self::run_assign_op(op, left, right, realm)?;

                Ok(value)
            }
        }
    }

    fn assign_expr_op(
        realm: &mut Realm,
        op: AssignOp,
        expr: &Expr,
        left: Value,
        scope: &mut Scope,
    ) -> RuntimeResult {
        Ok(match expr {
            Expr::Member(m) => {
                let obj = Self::run_expr(realm, &m.obj, m.span, scope)?;
                Self::assign_member_op_on(realm, obj, op, &m.prop, left, scope)?
            }
            Expr::SuperProp(super_prop) => {
                Self::assign_super_op(realm, op, super_prop, left, scope)?
            }
            Expr::OptChain(opt) => Self::assign_opt_chain_op(realm, op, opt, left, scope)?,
            Expr::Paren(paren) => Self::assign_expr_op(realm, op, &paren.expr, left, scope)?,

            epxr => Self::run_expr(realm, epxr, expr.span(), scope)?,
        })
    }

    fn assign_pat_op(
        realm: &mut Realm,
        op: AssignOp,
        pat: &AssignTargetPat,
        left: Value,
        scope: &mut Scope,
    ) -> RuntimeResult {
        if op != AssignOp::Assign {
            return Err(Error::syn("Invalid left-hand side in assignment").into());
        }

        match pat {
            AssignTargetPat::Array(arr) => {
                let pat = Pat::Array(arr.clone());

                Self::run_pat(
                    realm,
                    &pat,
                    scope,
                    &mut iter::once(left.copy()),
                    &mut |scope, name, value, realm| {
                        scope.update_or_define(name, value, realm);
                        Ok(())
                    },
                )?;
            }
            AssignTargetPat::Object(expr) => {
                let pat = Pat::Object(expr.clone());

                Self::run_pat(
                    realm,
                    &pat,
                    scope,
                    &mut iter::once(left.copy()),
                    &mut |scope, name, value, realm| {
                        scope.update_or_define(name, value, realm);
                        Ok(())
                    },
                )?;
            }
            AssignTargetPat::Invalid(_) => {
                return Err(Error::syn("Invalid left-hand side in assignment").into())
            }
        }

        Ok(left)
    }

    pub fn run_assign_op(
        op: AssignOp,
        left: Value,
        right: Value,
        realm: &mut Realm,
    ) -> RuntimeResult {
        Ok(match op {
            AssignOp::Assign => right,
            AssignOp::AddAssign => left.add(&right, realm)?,
            AssignOp::SubAssign => left.sub(&right, realm)?,
            AssignOp::MulAssign => left.mul(&right, realm)?,
            AssignOp::DivAssign => left.div(&right, realm)?,
            AssignOp::ModAssign => left.rem(&right, realm)?,
            AssignOp::LShiftAssign => left.shl(&right, realm)?,
            AssignOp::RShiftAssign => left.shr(&right, realm)?,
            AssignOp::ZeroFillRShiftAssign => left.ushr(&right, realm)?,
            AssignOp::BitOrAssign => left.or(&right, realm)?,
            AssignOp::BitXorAssign => left.xor(&right, realm)?,
            AssignOp::BitAndAssign => left.and(&right, realm)?,
            AssignOp::ExpAssign => left.exp(&right, realm)?,
            AssignOp::AndAssign => left.log_and(right),
            AssignOp::OrAssign => left.log_or(right),
            AssignOp::NullishAssign => {
                if left.is_nullish() {
                    right
                } else {
                    left
                }
            }
        })
    }
}

impl Interpreter {
    fn write_private_member_on_instance(
        realm: &mut Realm,
        instance: &ClassInstance,
        name: &str,
        member: PrivateMember,
        value: Value,
        this_value: Value,
    ) -> Res {
        match member {
            PrivateMember::Field(_) => {
                instance.update_private_field(name, value);
                Ok(())
            }
            PrivateMember::Accessor {
                set: Some(setter), ..
            } => setter.call(realm, vec![value], this_value).map(|_| ()),
            PrivateMember::Accessor { set: None, .. } => Err(Error::ty_error(format!(
                "Private accessor #{name} does not have a setter"
            ))),
            PrivateMember::Method(_) => Err(Error::ty_error(format!(
                "Cannot assign to private method {name}"
            ))),
        }
    }

    fn write_private_member_on_class(
        realm: &mut Realm,
        class: &Class,
        name: &str,
        member: PrivateMember,
        value: Value,
        this_value: Value,
    ) -> Res {
        match member {
            PrivateMember::Field(_) => {
                class.update_private_field(name, value);
                Ok(())
            }
            PrivateMember::Accessor {
                set: Some(setter), ..
            } => setter.call(realm, vec![value], this_value).map(|_| ()),
            PrivateMember::Accessor { set: None, .. } => Err(Error::ty_error(format!(
                "Private accessor #{name} does not have a setter"
            ))),
            PrivateMember::Method(_) => Err(Error::ty_error(format!(
                "Cannot assign to private method {name}"
            ))),
        }
    }
}
