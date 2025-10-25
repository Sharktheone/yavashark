#![allow(clippy::needless_pass_by_value)]

use crate::array::Array;
use crate::function::bound::BoundFunction;
use crate::inline_props::InlineObject;
use crate::partial_init::{Initializer, Partial};
use crate::realm::Realm;
use crate::{
    proto, Error, NativeConstructor, NativeFunction, ObjectHandle, Res, Value, ValueResult,
    Variable,
};
use std::cell::{Cell, RefCell};
use yavashark_macro::inline_props;

#[inline_props(enumerable = false, configurable)]
#[derive(Default, Debug)]
pub struct FunctionPrototype {
    pub apply: Partial<ObjectHandle, Apply>,
    pub bind: Partial<ObjectHandle, Bind>,
    pub call: Partial<ObjectHandle, Call>,
    pub constructor: Partial<ObjectHandle, FunctionConstructor>,
    pub length: usize,
    pub name: &'static str,
    #[prop("toString")]
    pub to_string: Partial<ObjectHandle, ToString>,
    #[prop("caller")]
    #[get]
    //TODO: this is still not fully correct
    pub caller_get: Partial<ObjectHandle, Caller>,
}

proto!(Apply, apply, "apply", 2);
proto!(Bind, bind, "bind", 1);
proto!(Call, call, "call", 1);
proto!(ToString, to_string, "toString", 0);

pub struct Caller;
impl Initializer<ObjectHandle> for Caller {
    fn initialize(realm: &mut Realm) -> Res<ObjectHandle> {
        Ok(realm
            .intrinsics
            .clone_public()
            .throw_type_error
            .get(realm)?
            .clone())
    }
}

impl FunctionPrototype {
    #[must_use]
    pub fn new(realm: &mut Realm) -> InlineObject<Self> {
        let proto = Self {
            name: RefCell::new("Function"),
            ..Self::default()
        };

        InlineObject::with_proto(proto, realm.intrinsics.obj.clone())
    }

    #[must_use]
    pub fn with_proto(obj: ObjectHandle) -> InlineObject<Self> {
        let proto = Self {
            name: RefCell::new("Function"),
            ..Self::default()
        };

        InlineObject::with_proto(proto, obj)
    }
}

pub struct GlobalFunctionConstructor;

impl Initializer<ObjectHandle> for GlobalFunctionConstructor {
    fn initialize(realm: &mut Realm) -> Res<ObjectHandle> {
        realm
            .intrinsics
            .func
            .clone()
            .get("constructor", realm)?
            .to_object()
    }
}

pub struct FunctionConstructor;

impl Initializer<ObjectHandle> for FunctionConstructor {
    fn initialize(realm: &mut Realm) -> Res<ObjectHandle> {
        let ctor = NativeConstructor::special("Function".to_string(), constructor, realm);

        ctor.define_property_attributes(
            "prototype".into(),
            Variable::new_read_only(realm.intrinsics.func.clone().into()),
            realm,
        )?;

        Ok(ctor)
    }
}

#[allow(unused)]
fn apply(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    let new_this = if let Some(new_this) = args.get(0) {
        if new_this.is_nullish() {
            realm.global.clone().into()
        } else {
            new_this.clone()
        }
    } else {
        realm.global.clone().into()
    };

    let args = if let Some(arr) = args.get(1) {
        let array = Array::from_array_like(realm, arr.copy())?;

        array.as_vec()?
    } else {
        vec![]
    };

    this.call(realm, args, new_this)
}

#[allow(unused)]
fn bind(mut args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    if args.is_empty() {
        return BoundFunction::new(this, Value::Undefined, vec![], realm);
    }

    BoundFunction::new(this, args.remove(0), args, realm)
}

#[allow(unused)]
fn call(mut args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    let new_this = if args.is_empty() {
        realm.global.clone().into()
    } else {
        args.remove(0)
    };

    this.call(realm, args, new_this)
}

#[allow(unused)]
fn constructor(mut args: Vec<Value>, realm: &mut Realm) -> Res<ObjectHandle> {
    let Some(body) = args.pop() else {
        return Ok(NativeFunction::new("anonymous", |_, _, _| Ok(Value::Undefined), realm).into());
    };

    let mut buf = "function anonymous(".to_owned();

    for (i, arg) in args.iter().enumerate() {
        if i != 0 {
            buf.push(',');
        }

        buf.push_str(&arg.to_string(realm)?);
    }

    buf.push_str(") {\n ");

    buf.push_str(&body.to_string(realm)?);

    buf.push_str("\n }");

    buf.push_str("anonymous");

    let Some(eval) = realm.intrinsics.eval.clone() else {
        return Err(Error::new("eval is not defined"));
    };

    Ok(eval
        .call(vec![Value::String(buf.into())], Value::Undefined, realm)?
        .to_object()?)
}

fn to_string(_args: Vec<Value>, this: Value, _realm: &mut Realm) -> ValueResult {
    if !this.is_callable() {
        return Err(Error::ty("toString called on non-function"));
    }

    Ok("function () { } ".into())
}

// impl Obj for FunctionPrototype {
//     fn define_property(
//         &self,
//         name: InternalPropertyKey,
//         value: Value,
//         realm: &mut Realm,
//     ) -> Res<DefinePropertyResult> {
//         let mut this = self.inner.try_borrow_mut()?;
//
//         if let InternalPropertyKey::String(name) = &name {
//             match name.as_str() {
//                 "apply" => {
//                     this.apply = value.into();
//                     return Ok(DefinePropertyResult::Handled);
//                 }
//                 "bind" => {
//                     this.bind = value.into();
//                     return Ok(DefinePropertyResult::Handled);
//                 }
//                 "call" => {
//                     this.call = value.into();
//                     return Ok(DefinePropertyResult::Handled);
//                 }
//                 "constructor" => {
//                     this.constructor = value.into();
//                     return Ok(DefinePropertyResult::Handled);
//                 }
//                 "length" => {
//                     this.length = value.into();
//                     return Ok(DefinePropertyResult::Handled);
//                 }
//                 "name" => {
//                     this.name = value.into();
//                     return Ok(DefinePropertyResult::Handled);
//                 }
//                 "toString" => {
//                     this.to_string = value.into();
//                     return Ok(DefinePropertyResult::Handled);
//                 }
//                 "caller" => {
//                     return Err(Error::ty(
//                         "Cannot assign to read only property 'caller' of function 'function prototype'",
//                     ));
//                 }
//
//                 _ => {}
//             }
//         }
//
//         this.object.define_property(name, value, realm)
//     }
//
//     fn define_property_attributes(
//         &self,
//         name: InternalPropertyKey,
//         value: Variable,
//         realm: &mut Realm,
//     ) -> Res<DefinePropertyResult> {
//         let mut this = self.inner.try_borrow_mut()?;
//
//         if let InternalPropertyKey::String(name) = &name {
//             match name.as_str() {
//                 "apply" => {
//                     this.apply = value.into();
//                     return Ok(DefinePropertyResult::Handled);
//                 }
//                 "bind" => {
//                     this.bind = value.into();
//                     return Ok(DefinePropertyResult::Handled);
//                 }
//                 "call" => {
//                     this.call = value.into();
//                     return Ok(DefinePropertyResult::Handled);
//                 }
//                 "constructor" => {
//                     this.constructor = value.into();
//                     return Ok(DefinePropertyResult::Handled);
//                 }
//                 "length" => {
//                     this.length = value.into();
//                     return Ok(DefinePropertyResult::Handled);
//                 }
//                 "name" => {
//                     this.name = value.into();
//                     return Ok(DefinePropertyResult::Handled);
//                 }
//                 "toString" => {
//                     this.to_string = value.into();
//                     return Ok(DefinePropertyResult::Handled);
//                 }
//                 "caller" => {
//                     return Err(Error::ty(
//                         "Cannot assign to read only property 'caller' of function 'function prototype'",
//                     ));
//                 }
//                 _ => {}
//             }
//         }
//
//         this.object.define_property_attributes(name, value, realm)
//     }
//
//     fn resolve_property(
//         &self,
//         name: InternalPropertyKey,
//         realm: &mut Realm,
//     ) -> Res<Option<Property>> {
//         let this = self.inner.try_borrow()?;
//
//         if let InternalPropertyKey::String(ref name) = name {
//             match name.as_str() {
//                 "apply" => return Ok(Some(this.apply.clone().into())),
//                 "bind" => return Ok(Some(this.bind.clone().into())),
//                 "call" => return Ok(Some(this.call.clone().into())),
//                 "constructor" => return Ok(Some(this.constructor.clone().into())),
//                 "length" => return Ok(Some(this.length.clone().into())),
//                 "name" => return Ok(Some(this.name.clone().into())),
//                 "toString" => return Ok(Some(this.to_string.clone().into())),
//                 "caller" => {
//                     return Err(Error::ty(
//                         "Cannot assign to read only property 'caller' of function 'function prototype'",
//                     ));
//                 }
//                 _ => {}
//             }
//         }
//
//         this.object.resolve_property(name, realm)
//     }
//
//     fn get_own_property(
//         &self,
//         name: InternalPropertyKey,
//         realm: &mut Realm,
//     ) -> Res<Option<Property>> {
//         let this = self.inner.try_borrow()?;
//
//         if let InternalPropertyKey::String(ref name) = name {
//             match name.as_str() {
//                 "apply" => return Ok(Some(this.apply.copy().into())),
//                 "bind" => return Ok(Some(this.bind.copy().into())),
//                 "call" => return Ok(Some(this.call.copy().into())),
//                 "constructor" => return Ok(Some(this.constructor.copy().into())),
//                 "length" => return Ok(Some(this.length.copy().into())),
//                 "name" => return Ok(Some(this.name.copy().into())),
//                 "toString" => return Ok(Some(this.to_string.copy().into())),
//                 "caller" => {
//                     return Err(Error::ty(
//                         "Cannot assign to read only property 'caller' of function 'function prototype'",
//                     ));
//                 }
//                 _ => {}
//             }
//         }
//
//         this.object.get_own_property(name, realm)
//     }
//
//     fn define_getter(
//         &self,
//         name: InternalPropertyKey,
//         value: ObjectHandle,
//         realm: &mut Realm,
//     ) -> Res {
//         let mut this = self.inner.try_borrow_mut()?;
//
//         this.object.define_getter(name, value, realm)
//     }
//
//     fn define_setter(
//         &self,
//         name: InternalPropertyKey,
//         value: ObjectHandle,
//         realm: &mut Realm,
//     ) -> Res {
//         let mut this = self.inner.try_borrow_mut()?;
//         this.object.define_setter(name, value, realm)
//     }
//
//     fn delete_property(
//         &self,
//         name: InternalPropertyKey,
//         realm: &mut Realm,
//     ) -> Res<Option<Property>> {
//         let mut this = self.inner.try_borrow_mut()?;
//
//         if let InternalPropertyKey::String(ref name) = name {
//             match name.as_str() {
//                 "apply" => {
//                     let old = this.apply.value.copy();
//                     this.apply = Value::Undefined.into();
//                     return Ok(Some(old.into()));
//                 }
//                 "bind" => {
//                     let old = this.bind.value.copy();
//                     this.bind = Value::Undefined.into();
//                     return Ok(Some(old.into()));
//                 }
//                 "call" => {
//                     let old = this.call.value.copy();
//                     this.call = Value::Undefined.into();
//                     return Ok(Some(old.into()));
//                 }
//                 "constructor" => {
//                     let old = this.constructor.value.copy();
//                     this.constructor = Value::Undefined.into();
//                     return Ok(Some(old.into()));
//                 }
//                 "length" => {
//                     let old = this.length.value.copy();
//                     this.length = Value::Undefined.into();
//                     return Ok(Some(old.into()));
//                 }
//                 "name" => {
//                     let old = this.name.value.copy();
//                     this.name = Value::Undefined.into();
//                     return Ok(Some(old.into()));
//                 }
//                 "toString" => {
//                     let old = this.to_string.value.copy();
//                     this.to_string = Value::Undefined.into();
//                     return Ok(Some(old.into()));
//                 }
//                 "caller" => {
//                     return Err(Error::ty(
//                         "Cannot assign to read only property 'caller' of function 'function prototype'",
//                     ));
//                 }
//                 _ => {}
//             }
//         }
//
//         this.object.delete_property(name, realm)
//     }
//
//     fn contains_own_key(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool> {
//         if let InternalPropertyKey::String(ref name) = name {
//             match name.as_str() {
//                 "apply" | "bind" | "call" | "constructor" | "length" | "name" | "toString"
//                 | "caller" => return Ok(true),
//                 _ => {}
//             }
//         }
//
//         let mut this = self.inner.try_borrow_mut()?;
//
//         this.object.contains_own_key(name, realm)
//     }
//
//     fn contains_key(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool> {
//         if let InternalPropertyKey::String(ref name) = name {
//             match name.as_str() {
//                 "apply" | "bind" | "call" | "constructor" | "length" | "name" | "toString"
//                 | "caller" => return Ok(true),
//                 _ => {}
//             }
//         }
//
//         let mut this = self.inner.try_borrow_mut()?;
//
//         this.object.contains_key(name, realm)
//     }
//
//     fn properties(&self, realm: &mut Realm) -> Res<Vec<(PropertyKey, Value)>> {
//         let this = self.inner.try_borrow()?;
//
//         let mut props = this.object.properties(realm)?;
//         props.push((PropertyKey::String("apply".into()), this.apply.value.copy()));
//         props.push((PropertyKey::String("bind".into()), this.bind.value.copy()));
//         props.push((PropertyKey::String("call".into()), this.call.value.copy()));
//         props.push((
//             PropertyKey::String("constructor".into()),
//             this.constructor.value.copy(),
//         ));
//         props.push((
//             PropertyKey::String("length".into()),
//             this.length.value.copy(),
//         ));
//         props.push((PropertyKey::String("name".into()), this.name.value.copy()));
//         props.push((
//             PropertyKey::String("toString".into()),
//             this.to_string.value.copy(),
//         ));
//
//         Ok(props)
//     }
//
//     // fn to_string(&self, _realm: &mut Realm) -> Res<YSString, Error> {
//     //     Ok("function () { [Native code] } ".into())
//     // }
//     //
//     // fn to_string_internal(&self) -> Res<YSString> {
//     //     Ok("function () { [Native code <Function Prototype>] } ".into())
//     // }
//
//     fn keys(&self, realm: &mut Realm) -> Res<Vec<PropertyKey>> {
//         let this = self.inner.try_borrow()?;
//
//         let mut keys = this.object.keys(realm)?;
//         keys.push(PropertyKey::String("apply".into()));
//         keys.push(PropertyKey::String("bind".into()));
//         keys.push(PropertyKey::String("call".into()));
//         keys.push(PropertyKey::String("constructor".into()));
//         keys.push(PropertyKey::String("length".into()));
//         keys.push(PropertyKey::String("name".into()));
//         keys.push(PropertyKey::String("toString".into()));
//
//         Ok(keys)
//     }
//
//     fn values(&self, realm: &mut Realm) -> Res<Vec<Value>> {
//         let this = self.inner.try_borrow()?;
//
//         let mut values = this.object.values(realm)?;
//         values.push(this.apply.value.copy());
//         values.push(this.bind.value.copy());
//         values.push(this.call.value.copy());
//         values.push(this.constructor.value.copy());
//         values.push(this.length.value.copy());
//         values.push(this.name.value.copy());
//         values.push(this.to_string.value.copy());
//
//         Ok(values)
//     }
//
//     fn enumerable_properties(
//         &self,
//         realm: &mut Realm,
//     ) -> Res<Vec<(PropertyKey, crate::value::Value)>> {
//         let this = self.inner.try_borrow()?;
//
//         this.object.enumerable_properties(realm)
//     }
//
//     fn enumerable_keys(&self, realm: &mut Realm) -> Res<Vec<PropertyKey>> {
//         let this = self.inner.try_borrow()?;
//
//         this.object.enumerable_keys(realm)
//     }
//
//     fn enumerable_values(&self, realm: &mut Realm) -> Res<Vec<crate::value::Value>> {
//         let this = self.inner.try_borrow()?;
//
//         this.object.enumerable_values(realm)
//     }
//
//     fn clear_properties(&self, realm: &mut Realm) -> Res {
//         let mut this = self.inner.try_borrow_mut()?;
//
//         this.object.clear_properties(realm)?;
//         this.apply = Value::Undefined.into();
//         this.bind = Value::Undefined.into();
//         this.call = Value::Undefined.into();
//         this.constructor = Value::Undefined.into();
//         this.length = Value::Number(0.0).into();
//         this.name = Value::string("Function").into();
//         this.to_string = Value::Undefined.into();
//
//         Ok(())
//     }
//
//     fn get_array_or_done(&self, index: usize, realm: &mut Realm) -> Res<(bool, Option<Value>)> {
//         let mut this = self.inner.try_borrow_mut()?;
//
//         this.object.get_array_or_done(index, realm)
//     }
//
//     fn prototype(&self, realm: &mut Realm) -> Res<ObjectOrNull> {
//         let this = self.inner.try_borrow()?;
//
//         this.object.prototype(realm)
//     }
//
//     fn set_prototype(&self, prototype: ObjectOrNull, realm: &mut Realm) -> Res {
//         let mut this = self.inner.try_borrow_mut()?;
//
//         this.object.set_prototype(prototype, realm)
//     }
//
//     fn get_property_descriptor(
//         &self,
//         name: InternalPropertyKey,
//         realm: &mut Realm,
//     ) -> Res<Option<PropertyDescriptor>> {
//         if matches!(name, InternalPropertyKey::String(ref n) if n.as_str() == "caller") {
//             let throw_type_error = realm
//                 .intrinsics
//                 .clone_public()
//                 .throw_type_error
//                 .get(realm)?
//                 .clone();
//
//             return Ok(Some(PropertyDescriptor::Accessor {
//                 get: Some(throw_type_error.clone()),
//                 set: Some(throw_type_error),
//                 enumerable: false,
//                 configurable: true,
//             }));
//         }
//
//         let Some(prop) = self.get_own_property(name, realm)? else {
//             return Ok(None);
//         };
//
//         match prop {
//             Property::Value(v, a) => Ok(Some(PropertyDescriptor::Data {
//                 value: v,
//                 writable: a.is_writable(),
//                 enumerable: a.is_enumerable(),
//                 configurable: a.is_configurable(),
//             })),
//             Property::Getter(g, props) => Ok(Some(PropertyDescriptor::Accessor {
//                 get: Some(g),
//                 set: None,
//                 enumerable: props.is_enumerable(),
//                 configurable: props.is_configurable(),
//             })),
//         }
//     }
//
//     fn name(&self) -> String {
//         "FunctionPrototype".to_string()
//     }
//
//     fn gc_refs(&self) -> Vec<GcRef<BoxedObj>> {
//         let this = self.inner.borrow();
//
//         let mut refs = this.object.gc_refs();
//
//         if let Some(r) = this.apply.value.gc_ref() {
//             refs.push(r);
//         }
//
//         if let Some(r) = this.bind.value.gc_ref() {
//             refs.push(r);
//         }
//
//         if let Some(r) = this.call.value.gc_ref() {
//             refs.push(r);
//         }
//
//         if let Some(r) = this.constructor.value.gc_ref() {
//             refs.push(r);
//         }
//
//         if let Some(r) = this.length.value.gc_ref() {
//             refs.push(r);
//         }
//
//         if let Some(r) = this.name.value.gc_ref() {
//             refs.push(r);
//         }
//
//         if let Some(r) = this.to_string.value.gc_ref() {
//             refs.push(r);
//         }
//
//         refs
//     }
// }
