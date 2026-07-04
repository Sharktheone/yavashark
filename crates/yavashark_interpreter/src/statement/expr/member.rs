use crate::Interpreter;
use swc_common::Span;
use swc_ecma_ast::{MemberExpr, MemberProp, ObjectLit};
use yavashark_env::builtins::{BigIntObj, BooleanObj, NumberObj, StringObj, SymbolObj};
use yavashark_env::scope::Scope;
use yavashark_env::value::Obj;
use yavashark_env::{Class, ClassInstance, ControlFlow, Error, InternalPropertyKey, PrivateMember, PropertyKey, Realm, RuntimeResult, Value};
use yavashark_env::value::property_key::IntoPropertyKey;
use yavashark_string::YSString;

impl Interpreter {
    pub fn run_member(realm: &mut Realm, stmt: &MemberExpr, scope: &mut Scope) -> RuntimeResult {
        let value = Self::run_expr(realm, &stmt.obj, stmt.span, scope)?;

        Self::run_member_on(realm, value, &stmt.prop, stmt.span, scope)
    }

    pub fn run_member_on(
        realm: &mut Realm,
        value: Value,
        prop: &MemberProp,
        span: Span,
        scope: &mut Scope,
    ) -> RuntimeResult {
        Ok(Self::run_member_on_primitives(realm, value, prop, span, scope)?.0)
    }

    pub fn run_member_on_primitives(
        realm: &mut Realm,
        value: Value,
        prop: &MemberProp,
        span: Span,
        scope: &mut Scope,
    ) -> Result<(Value, Option<Value>), ControlFlow> {
        let name = match &prop {
            MemberProp::Ident(i) => InternalPropertyKey::from(YSString::from_ref(i.sym.as_str())),
            MemberProp::Computed(e) => Self::run_expr(realm, &e.expr, span, scope)?.into_internal_property_key(realm)?,
            MemberProp::PrivateName(p) => {
                let name = p.name.as_str();
                let obj = value.as_object()?;

                if let Some(class) = obj.downcast::<ClassInstance>() {
                    let member = class.get_private_prop(name, realm)?.ok_or_else(|| {
                        ControlFlow::error_type(format!("Private name {name} not found"))
                    })?;

                    return Self::resolve_private_member(realm, member, value.copy());
                }

                if let Some(class) = obj.downcast::<Class>() {
                    let member = class.get_private_prop(name).ok_or_else(|| {
                        ControlFlow::error_type(format!("Private name {name} not found"))
                    })?;

                    return Self::resolve_private_member(realm, member, value.copy());
                }

                return Err(ControlFlow::error_type(format!(
                    "Private name {name} can only be used in class"
                )));
            }
        };

        let proto = match value {
            Value::Object(ref o) => return Ok((
                o.resolve_property(name, realm)?
                    .unwrap_or(Value::Undefined),
                Some(value),
            )),
            Value::Undefined => return Err(ControlFlow::error_type(format!(
                "Cannot read property '{name}' of undefined",
            ))),
            Value::Null => return Err(ControlFlow::error_type(format!(
                "Cannot read property '{name}' of null",
            ))),
            Value::String(ref s) => {
                if let InternalPropertyKey::Index(i) = name {
                    let v = StringObj::get_single_str(&*s.as_str_lossy(), i as isize)
                        .map_or(Value::Undefined, Into::into);

                    return Ok((v, Some(value)));
                }

                if matches!(&name, InternalPropertyKey::String(k) if k == "length") {
                    return Ok((Value::Number(s.len() as f64), Some(value)));
                }

                realm.intrinsics.clone_public().string.get(realm)?.clone()
            }
            Value::Number(_) => realm.intrinsics.clone_public().number.get(realm)?.clone(),
            Value::Boolean(_) => realm.intrinsics.clone_public().boolean.get(realm)?.clone(),
            Value::Symbol(_) => realm.intrinsics.clone_public().symbol.get(realm)?.clone(),
            Value::BigInt(_) => realm.intrinsics.clone_public().bigint.get(realm)?.clone(),
        };

        let v = proto
            .resolve_property_no_get_set(name, realm)?
            .map(|prop| prop.get(value.copy(), realm))
            .transpose()?
            .unwrap_or(Value::Undefined);

        Ok((v, Some(value)))
    }
}

impl Interpreter {
    pub(crate) fn resolve_private_member(
        realm: &mut Realm,
        member: PrivateMember,
        base: Value,
    ) -> Result<(Value, Option<Value>), ControlFlow> {
        match member {
            PrivateMember::Field(value) => Ok((value, None)),
            PrivateMember::Method(func) => Ok((func, Some(base))),
            PrivateMember::Accessor { get, .. } => {
                if let Some(getter) = get {
                    let result = getter
                        .call(realm, vec![], base.copy())
                        .map_err(ControlFlow::Error)?;

                    Ok((result, None))
                } else {
                    Ok((Value::Undefined, None))
                }
            }
        }
    }
}
