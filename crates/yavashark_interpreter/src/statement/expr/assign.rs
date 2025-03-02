use swc_ecma_ast::{AssignExpr, AssignOp, AssignTarget, MemberExpr, MemberProp, OptChainBase, OptChainExpr, SimpleAssignTarget, SuperProp, SuperPropExpr};

use yavashark_env::scope::Scope;
use yavashark_env::{Error, Realm, Res, RuntimeResult, Value};
use yavashark_value::Obj;
use crate::Interpreter;

impl Interpreter {
    pub fn run_assign(realm: &mut Realm, stmt: &AssignExpr, scope: &mut Scope) -> RuntimeResult {
        let value = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;

        if stmt.op == AssignOp::Assign {
            return Ok(
                Self::assign_target(realm, &stmt.left, value, scope).map(|()| Value::Undefined)?
            );
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
                    scope.update_or_define(name, value)
                }
                SimpleAssignTarget::Member(m) => Self::assign_member(realm, m, value, scope),
                SimpleAssignTarget::SuperProp(super_prop) => Self::assign_super(super_prop, value, scope, realm),
                SimpleAssignTarget::OptChain(opt) => Self::assign_opt_chain(realm, opt, value, scope),
                
                _ => todo!("assign targets"),
            },
            AssignTarget::Pat(_) => {
                todo!("Pattern assignment")
            }
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
                MemberProp::Ident(i) => Value::String(i.sym.to_string()),
                MemberProp::PrivateName(p) => Value::String(p.name.to_string()),
                MemberProp::Computed(c) => Self::run_expr(realm, &c.expr, c.span, scope)?,
            };

            obj.define_property(name, value);
            Ok(())
        } else {
            Err(Error::ty_error(format!(
                "Invalid left-hand side in assignment: {obj}"
            )))
        }
    }
    
    pub fn assign_super(super_prop: &SuperPropExpr, value: Value, scope: &mut Scope, realm: &mut Realm) -> Res {
        let this = scope.this()?;
        
        let obj = this.as_object()?;
        
        let proto = obj.prototype()?;
        let sup = proto.resolve(this, realm)?;
        
        
        match &super_prop.prop {
            SuperProp::Ident(i) => {
                let name = i.sym.to_string();
                
                sup.define_property(name.into(), value)
            }
            SuperProp::Computed(p) => {
                let name = Self::run_expr(realm, &p.expr, super_prop.span, scope)?;
                
                sup.define_property(name, value)
            }
        }
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

                let this = this.unwrap_or(scope.this()?);

                Self::run_call_on(
                    realm, &callee, this, &call.args, call.span, scope,
                )?; 
                //TODO: maybe we should throw an error here?
                
                Ok(())
            }
        }
    }

    pub fn assign_target_op(
        realm: &mut Realm,
        op: AssignOp,
        target: &AssignTarget,
        left: Value,
        scope: &mut Scope,
    ) -> RuntimeResult {
        match target {
            AssignTarget::Simple(t) => match t {
                SimpleAssignTarget::Ident(i) => {
                    let name = i.sym.to_string();

                    let right = scope
                        .resolve(&name, realm)?
                        .ok_or_else(|| Error::reference_error(format!("{name} is not defined")))?;

                    let value = Self::run_assign_op(op, left, right, realm)?;

                    scope.update(&name, value.copy())?;

                    Ok(value)
                }
                SimpleAssignTarget::Member(m) => Self::assign_member_op(realm, op, m, left, scope),
                SimpleAssignTarget::SuperProp(super_prop) => Self::assign_super_op(realm, op, super_prop, left, scope),
                SimpleAssignTarget::OptChain(opt) => Self::assign_opt_chain_op(realm, op, opt, left, scope),
                _ => todo!("assign targets"),
            },
            AssignTarget::Pat(_) => {
                todo!("Pattern assignment")
            }
        }
    }

    pub fn assign_member_op(
        realm: &mut Realm,
        op: AssignOp,
        m: &MemberExpr,
        left: Value,
        scope: &mut Scope,
    ) -> RuntimeResult {
        let obj = Self::run_expr(realm, &m.obj, m.span, scope)?;
        
        Self::assign_member_op_on(realm, obj, op, &m.prop, left, scope)
    }
    
    pub fn assign_member_op_on(
        realm: &mut Realm,
        obj: Value,
        op: AssignOp,
        m: &MemberProp,
        left: Value,
        scope: &mut Scope,
    ) -> RuntimeResult {
        if let Value::Object(obj) = obj {
            let name = match m {
                MemberProp::Ident(i) => Value::String(i.sym.to_string()),
                MemberProp::PrivateName(p) => Value::String(p.name.to_string()),
                MemberProp::Computed(c) => Self::run_expr(realm, &c.expr, c.span, scope)?,
            };

            let right = obj
                .resolve_property(&name, realm)?
                .unwrap_or(Value::Undefined);

            let value = Self::run_assign_op(op, left, right, realm)?;

            obj.define_property(name, value.copy());
            Ok(value)
        } else {
            Err(Error::ty_error(format!("Invalid left-hand side in assignment: {obj}")).into())
        }
    }
    
    pub fn assign_super_op(
        realm: &mut Realm,
        op: AssignOp,
        super_prop: &SuperPropExpr,
        left: Value,
        scope: &mut Scope,
    ) -> RuntimeResult {
        let this = scope.this()?;
        
        let obj = this.as_object()?;
        
        let proto = obj.prototype()?;
        let sup = proto.resolve(this, realm)?;
        
        match &super_prop.prop {
            SuperProp::Ident(i) => {
                let name = i.sym.to_string().into();
                
                let right = sup
                    .get_property(&name, realm)?;
                
                let value = Self::run_assign_op(op, left, right, realm)?;
                
                sup.define_property(name, value.copy());
                Ok(value)
            }
            SuperProp::Computed(p) => {
                let name = Self::run_expr(realm, &p.expr, super_prop.span, scope)?;
                
                let right = sup
                    .get_property(&name, realm)?;
                
                let value = Self::run_assign_op(op, left, right, realm)?;
                
                sup.define_property(name, value.copy());
                Ok(value)
            }
        }
    }
    
    pub fn assign_opt_chain_op(
        realm: &mut Realm,
        op: AssignOp,
        opt: &OptChainExpr,
        left: Value,
        scope: &mut Scope,
    ) -> RuntimeResult {
        match &*opt.base {
            OptChainBase::Member(member) => {
                let obj = Self::run_expr(realm, &member.obj, member.span, scope)?;

                if (obj == Value::Undefined || obj == Value::Null) && opt.optional {
                    return Ok(left);
                }

                Self::assign_member_op_on(realm, obj, op, &member.prop, left, scope)
            }
            OptChainBase::Call(call) => {
                let (callee, this) = Self::run_call_expr(realm, &call.callee, call.span, scope)?;

                if (callee == Value::Undefined || callee == Value::Null) && opt.optional {
                    return Ok(left);
                }

                let this = this.unwrap_or(scope.this()?);

                let right = Self::run_call_on(
                    realm, &callee, this, &call.args, call.span, scope,
                )?;
                //TODO: maybe we should throw an error here?
                
                let value = Self::run_assign_op(op, left, right, realm)?;
                
                Ok(value)
            }
        }
    }

    pub fn run_assign_op(
        op: AssignOp,
        left: Value,
        right: Value,
        realm: &mut Realm,
    ) -> RuntimeResult {
        Ok(match op {
            AssignOp::Assign => right,
            AssignOp::AddAssign => left + right,
            AssignOp::SubAssign => left - right,
            AssignOp::MulAssign => left * right,
            AssignOp::DivAssign => left / right,
            AssignOp::ModAssign => left % right,
            AssignOp::LShiftAssign => left << right,
            AssignOp::RShiftAssign => left >> right,
            AssignOp::ZeroFillRShiftAssign => left.zero_fill_rshift(&right),
            AssignOp::BitOrAssign => left | right,
            AssignOp::BitXorAssign => left ^ right,
            AssignOp::BitAndAssign => left & right,
            AssignOp::ExpAssign => left.pow(&right, realm)?,
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
