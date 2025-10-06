#![allow(clippy::needless_pass_by_value)]

use crate::value::{BoxedObj, DefinePropertyResult, MutObj, Obj, Property};
use std::cell::RefCell;
use yavashark_garbage::GcRef;

use crate::array::Array;
use crate::function::bound::BoundFunction;
use crate::realm::Realm;
use crate::{Error, InternalPropertyKey, MutObject, NativeConstructor, NativeFunction, ObjectHandle, ObjectOrNull, ObjectProperty, PropertyKey, Res, Value, ValueResult, Variable};

#[derive(Debug)]
struct MutableFunctionPrototype {
    pub object: MutObject,
    pub apply: ObjectProperty,
    pub bind: ObjectProperty,
    pub call: ObjectProperty,
    pub constructor: ObjectProperty,
    pub length: ObjectProperty,
    pub name: ObjectProperty,
    pub to_string: ObjectProperty,
}

#[derive(Debug)]
pub struct FunctionPrototype {
    inner: RefCell<MutableFunctionPrototype>,
}

impl FunctionPrototype {
    #[must_use]
    pub fn new(obj: ObjectHandle) -> Self {
        Self {
            inner: RefCell::new(MutableFunctionPrototype {
                object: MutObject::with_proto(obj),
                apply: Value::Undefined.into(),
                bind: Value::Undefined.into(),
                call: Value::Undefined.into(),
                constructor: Value::Undefined.into(),
                length: Value::Number(0.0).into(),
                name: Value::string("Function").into(),
                to_string: Value::Undefined.into(),
            }),
        }
    }

    pub fn initialize(&self, func: ObjectHandle, realm: &mut Realm) -> Res {
        let mut this = self.inner.try_borrow_mut()?;

        this.apply = NativeFunction::with_proto("apply", apply, func.clone(), realm).into();
        this.bind = NativeFunction::with_proto("bind", bind, func.clone(), realm).into();
        this.call = NativeFunction::with_proto("call", call, func.clone(), realm).into();
        this.constructor = NativeConstructor::special_with_proto(
            "Function".to_string(),
            constructor,
            func.clone(),
            func.clone(),
        )
        .into();
        this.to_string = NativeFunction::with_proto("toString", to_string, func.clone(), realm).into();

        this.constructor
            .value
            .as_object()?
            .define_property_attributes("prototype".into(), Variable::new_read_only(func.into()), realm)?;

        Ok(())
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

    buf.push_str(") { ");

    buf.push_str(&body.to_string(realm)?);

    buf.push_str(" }");

    buf.push_str("anonymous");

    let Some(eval) = realm.intrinsics.eval.clone() else {
        return Err(Error::new("eval is not defined"));
    };

    
    Ok(eval.call(vec![Value::String(buf.into())], Value::Undefined, realm)?.to_object()?)
        
}

fn to_string(_args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    if !this.is_callable() {
        return Err(Error::ty("toString called on non-function"));
    }

    Ok(this.to_string(realm)?.into())
}


#[allow(unused)]
impl Obj for FunctionPrototype {
    fn define_property(&self, name: InternalPropertyKey, value: crate::value::Value, realm: &mut Realm) -> Res<DefinePropertyResult> {
        todo!()
    }

    fn define_property_attributes(&self, name: InternalPropertyKey, value: crate::value::Variable, realm: &mut Realm) -> Res<DefinePropertyResult> {
        todo!()
    }

    fn resolve_property(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<Option<Property>> {
        todo!()
    }

    fn get_own_property(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<Option<Property>> {
        todo!()
    }

    fn define_getter(&self, name: InternalPropertyKey, callback: ObjectHandle, realm: &mut Realm) -> Res {
        todo!()
    }

    fn define_setter(&self, name: InternalPropertyKey, callback: ObjectHandle, realm: &mut Realm) -> Res {
        todo!()
    }

    fn delete_property(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<Option<Property>> {
        todo!()
    }

    fn contains_own_key(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool> {
        todo!()
    }

    fn contains_key(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool> {
        todo!()
    }

    fn properties(&self, realm: &mut Realm) -> Res<Vec<(PropertyKey, crate::value::Value)>> {
        todo!()
    }

    fn keys(&self, realm: &mut Realm) -> Res<Vec<PropertyKey>> {
        todo!()
    }

    fn values(&self, realm: &mut Realm) -> Res<Vec<crate::value::Value>> {
        todo!()
    }

    fn enumerable_properties(&self, realm: &mut Realm) -> Res<Vec<(PropertyKey, crate::value::Value)>> {
        todo!()
    }

    fn enumerable_keys(&self, realm: &mut Realm) -> Res<Vec<PropertyKey>> {
        todo!()
    }

    fn enumerable_values(&self, realm: &mut Realm) -> Res<Vec<crate::value::Value>> {
        todo!()
    }

    fn clear_properties(&self, realm: &mut Realm) -> Res {
        todo!()
    }

    fn get_array_or_done(&self, idx: usize, realm: &mut Realm) -> Res<(bool, Option<crate::value::Value>)> {
        todo!()
    }

    fn prototype(&self, realm: &mut Realm) -> Res<ObjectOrNull> {
        todo!()
    }

    fn set_prototype(&self, prototype: ObjectOrNull, realm: &mut Realm) -> Res {
        todo!()
    }

    fn gc_refs(&self) -> Vec<GcRef<BoxedObj>> {
        todo!()
    }
}

// impl Obj for FunctionPrototype {
//     fn define_property(&self, name: Value, value: Value) -> Res {
//         let mut this = self.inner.try_borrow_mut()?;
//
//         if let Value::String(name) = &name {
//             match name.as_str() {
//                 "apply" => {
//                     this.apply = value.into();
//                     return Ok(());
//                 }
//                 "bind" => {
//                     this.bind = value.into();
//                     return Ok(());
//                 }
//                 "call" => {
//                     this.call = value.into();
//                     return Ok(());
//                 }
//                 "constructor" => {
//                     this.constructor = value.into();
//                     return Ok(());
//                 }
//                 "length" => {
//                     this.length = value.into();
//                     return Ok(());
//                 }
//                 "name" => {
//                     this.name = value.into();
//                     return Ok(());
//                 }
//                 "toString" => {
//                     this.to_string = value.into();
//                     return Ok(());
//                 }
//
//                 _ => {}
//             }
//         }
//
//         this.object.define_property(name, value)
//     }
//
//     fn define_variable(&self, name: Value, value: Variable) -> Res {
//         let mut this = self.inner.try_borrow_mut()?;
//
//         if let Value::String(name) = &name {
//             match name.as_str() {
//                 "apply" => {
//                     this.apply = value.into();
//                     return Ok(());
//                 }
//                 "bind" => {
//                     this.bind = value.into();
//                     return Ok(());
//                 }
//                 "call" => {
//                     this.call = value.into();
//                     return Ok(());
//                 }
//                 "constructor" => {
//                     this.constructor = value.into();
//                     return Ok(());
//                 }
//                 "length" => {
//                     this.length = value.into();
//                     return Ok(());
//                 }
//                 "name" => {
//                     this.name = value.into();
//                     return Ok(());
//                 }
//                 "toString" => {
//                     this.to_string = value.into();
//                     return Ok(());
//                 }
//                 _ => {}
//             }
//         }
//
//         this.object.define_variable(name, value)
//     }
//
//     fn resolve_property(&self, name: &Value) -> Res<Option<ObjectProperty>> {
//         let this = self.inner.try_borrow()?;
//
//         if let Value::String(name) = name {
//             match name.as_str() {
//                 "apply" => return Ok(Some(this.apply.clone())),
//                 "bind" => return Ok(Some(this.bind.clone())),
//                 "call" => return Ok(Some(this.call.clone())),
//                 "constructor" => return Ok(Some(this.constructor.clone())),
//                 "length" => return Ok(Some(this.length.clone())),
//                 "name" => return Ok(Some(this.name.clone())),
//                 "toString" => return Ok(Some(this.to_string.clone())),
//                 _ => {}
//             }
//         }
//
//         this.object.resolve_property(name)
//     }
//
//     fn get_property(&self, name: &Value) -> Res<Option<ObjectProperty>> {
//         let this = self.inner.try_borrow()?;
//
//         if let Value::String(name) = name {
//             match name.as_str() {
//                 "apply" => return Ok(Some(this.apply.copy())),
//                 "bind" => return Ok(Some(this.bind.copy())),
//                 "call" => return Ok(Some(this.call.copy())),
//                 "constructor" => return Ok(Some(this.constructor.copy())),
//                 "length" => return Ok(Some(this.length.copy())),
//                 "name" => return Ok(Some(this.name.copy())),
//                 "toString" => return Ok(Some(this.to_string.copy())),
//                 _ => {}
//             }
//         }
//
//         this.object.get_property(name).map(|v| v.map(|v| v.copy()))
//     }
//
//     fn define_getter(&self, name: Value, value: Value) -> Res {
//         let mut this = self.inner.try_borrow_mut()?;
//
//         this.object.define_getter(name, value)
//     }
//
//     fn define_setter(&self, name: Value, value: Value) -> Res {
//         let mut this = self.inner.try_borrow_mut()?;
//         this.object.define_setter(name, value)
//     }
//
//     fn delete_property(&self, name: &Value) -> Res<Option<Value>> {
//         let mut this = self.inner.try_borrow_mut()?;
//
//         if let Value::String(name) = name {
//             match name.as_str() {
//                 "apply" => {
//                     let old = this.apply.value.copy();
//                     this.apply = Value::Undefined.into();
//                     return Ok(Some(old));
//                 }
//                 "bind" => {
//                     let old = this.bind.value.copy();
//                     this.bind = Value::Undefined.into();
//                     return Ok(Some(old));
//                 }
//                 "call" => {
//                     let old = this.call.value.copy();
//                     this.call = Value::Undefined.into();
//                     return Ok(Some(old));
//                 }
//                 "constructor" => {
//                     let old = this.constructor.value.copy();
//                     this.constructor = Value::Undefined.into();
//                     return Ok(Some(old));
//                 }
//                 "length" => {
//                     let old = this.length.value.copy();
//                     this.length = Value::Undefined.into();
//                     return Ok(Some(old));
//                 }
//                 "name" => {
//                     let old = this.name.value.copy();
//                     this.name = Value::Undefined.into();
//                     return Ok(Some(old));
//                 }
//                 "toString" => {
//                     let old = this.to_string.value.copy();
//                     this.to_string = Value::Undefined.into();
//                     return Ok(Some(old));
//                 }
//                 _ => {}
//             }
//         }
//
//         this.object.delete_property(name)
//     }
//
//     fn contains_key(&self, name: &Value) -> Res<bool> {
//         if let Value::String(name) = name {
//             match name.as_str() {
//                 "apply" | "bind" | "call" | "constructor" | "length" | "name" | "toString" => {
//                     return Ok(true)
//                 }
//                 _ => {}
//             }
//         }
//
//         let this = self.inner.try_borrow()?;
//
//         this.object.contains_key(name)
//     }
//
//     fn name(&self) -> String {
//         "FunctionPrototype".to_string()
//     }
//
//     fn to_string(&self, _realm: &mut Realm) -> Res<YSString, Error> {
//         Ok("function () { [Native code] } ".into())
//     }
//
//     fn to_string_internal(&self) -> Res<YSString> {
//         Ok("function () { [Native code <Function Prototype>] } ".into())
//     }
//
//     fn properties(&self) -> Res<Vec<(Value, Value)>> {
//         let this = self.inner.try_borrow()?;
//
//         let mut props = this.object.properties()?;
//         props.push((Value::String("apply".into()), this.apply.value.copy()));
//         props.push((Value::String("bind".into()), this.bind.value.copy()));
//         props.push((Value::String("call".into()), this.call.value.copy()));
//         props.push((
//             Value::String("constructor".into()),
//             this.constructor.value.copy(),
//         ));
//         props.push((Value::String("length".into()), this.length.value.copy()));
//         props.push((Value::String("name".into()), this.name.value.copy()));
//         props.push((
//             Value::String("toString".into()),
//             this.to_string.value.copy(),
//         ));
//
//         Ok(props)
//     }
//
//     fn keys(&self) -> Res<Vec<Value>> {
//         let this = self.inner.try_borrow()?;
//
//         let mut keys = this.object.keys()?;
//         keys.push(Value::string("apply"));
//         keys.push(Value::string("bind"));
//         keys.push(Value::string("call"));
//         keys.push(Value::string("constructor"));
//         keys.push(Value::string("length"));
//         keys.push(Value::string("name"));
//         keys.push(Value::string("toString"));
//
//         Ok(keys)
//     }
//
//     fn values(&self) -> Res<Vec<Value>> {
//         let this = self.inner.try_borrow()?;
//
//         let mut values = this.object.values()?;
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
//     fn get_array_or_done(&self, index: usize) -> Res<(bool, Option<Value>)> {
//         let this = self.inner.try_borrow()?;
//
//         this.object.get_array_or_done(index)
//     }
//
//     fn clear_values(&self) -> Res {
//         let mut this = self.inner.try_borrow_mut()?;
//
//         this.object.clear_values()?;
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
//     fn prototype(&self) -> Res<ObjectProperty> {
//         let this = self.inner.try_borrow()?;
//
//         this.object.prototype()
//     }
// }
