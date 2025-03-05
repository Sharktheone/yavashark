use swc_common::Spanned;
use swc_ecma_ast::{AssignExpr, AssignOp, AssignTarget, AssignTargetPat, ExportDefaultExpr, Expr, MemberExpr, MemberProp, OptChainBase, OptChainExpr, ParenExpr, Pat, SimpleAssignTarget, SuperProp, SuperPropExpr};

use yavashark_env::scope::Scope;
use yavashark_env::{Error, Realm, Res, RuntimeResult, Value};
use yavashark_value::Obj;
use crate::Interpreter;

impl Interpreter {
    pub fn run_assign(realm: &mut Realm, stmt: &AssignExpr, scope: &mut Scope) -> RuntimeResult {
        let value = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;

        if stmt.op == AssignOp::Assign {
            return Ok(
                Self::assign_target(realm, &stmt.left, value.copy(), scope).map(|()| value)?
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
                SimpleAssignTarget::SuperProp(super_prop) => Self::assign_super(realm, super_prop, value, scope),
                SimpleAssignTarget::OptChain(opt) => Self::assign_opt_chain(realm, opt, value, scope),
                SimpleAssignTarget::Paren(paren) => Self::assign_expr(realm, &paren.expr, value, scope),
                
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
    
    pub fn assign_super(realm: &mut Realm, super_prop: &SuperPropExpr, value: Value, scope: &mut Scope) -> Res {
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
    
    pub fn assign_expr(
        realm: &mut Realm,
        expr: &Expr,
        value: Value,
        scope: &mut Scope,
    ) -> Res {
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
            },
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
                
                Self::run_pat(realm, &pat, scope, value)?;
            }
            AssignTargetPat::Object(expr) => {
                let pat = Pat::Object(expr.clone());
                
                Self::run_pat(realm, &pat, scope, value)?;
            }
            AssignTargetPat::Invalid(_) => return Err(Error::syn("Invalid left-hand side in assignment")),
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

                    scope.update(&name, value.copy())?;

                    Ok(value)
                }
                SimpleAssignTarget::Member(m) => Self::assign_member_op(realm, op, m, right, scope),
                SimpleAssignTarget::SuperProp(super_prop) => Self::assign_super_op(realm, op, super_prop, right, scope),
                SimpleAssignTarget::OptChain(opt) => Self::assign_opt_chain_op(realm, op, opt, right, scope),
                SimpleAssignTarget::Paren(paren) => Self::assign_expr_op(realm, op, &paren.expr, right, scope),
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
                MemberProp::Ident(i) => Value::String(i.sym.to_string()),
                MemberProp::PrivateName(p) => Value::String(p.name.to_string()),
                MemberProp::Computed(c) => Self::run_expr(realm, &c.expr, c.span, scope)?,
            };

            let left = obj
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
        right: Value,
        scope: &mut Scope,
    ) -> RuntimeResult {
        let this = scope.this()?;
        
        let obj = this.as_object()?;
        
        let proto = obj.prototype()?;
        let sup = proto.resolve(this, realm)?;
        
        match &super_prop.prop {
            SuperProp::Ident(i) => {
                let name = i.sym.to_string().into();
                
                let left = sup
                    .get_property(&name, realm)?;
                
                let value = Self::run_assign_op(op, left, right, realm)?;
                
                sup.define_property(name, value.copy());
                Ok(value)
            }
            SuperProp::Computed(p) => {
                let name = Self::run_expr(realm, &p.expr, super_prop.span, scope)?;
                
                let left = sup
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

                let this = this.unwrap_or(scope.this()?);

                let left = Self::run_call_on(
                    realm, &callee, this, &call.args, call.span, scope,
                )?;
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
            },
            Expr::SuperProp(super_prop) => Self::assign_super_op(realm, op, super_prop, left, scope)?,
            Expr::OptChain(opt) => Self::assign_opt_chain_op(realm, op, opt, left, scope)?,
            Expr::Paren(paren) => Self::assign_expr_op(realm, op, &paren.expr, left, scope)?,
            
            epxr => {
                Self::run_expr(realm, epxr, expr.span(), scope)?
            },
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
                
                Self::run_pat(realm, &pat, scope, left.copy())?;
            }
            AssignTargetPat::Object(expr) => {
                let pat = Pat::Object(expr.clone());
                
                Self::run_pat(realm, &pat, scope, left.copy())?;
            }
            AssignTargetPat::Invalid(_) => return Err(Error::syn("Invalid left-hand side in assignment").into()),
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
