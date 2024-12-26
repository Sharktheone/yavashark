use crate::{
    AsAny, BoxedObj, Error, Obj, Object, ObjectProperty, Realm, Value, Variable,
};
use std::any::type_name;
use std::fmt::Debug;
use yavashark_garbage::collectable::OwningGcRefCellGuard;

impl<C: Realm> From<&str> for Value<C> {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl<C: Realm> From<String> for Value<C> {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl<C: Realm> From<&String> for Value<C> {
    fn from(s: &String) -> Self {
        Self::String(s.clone())
    }
}

impl<C: Realm> From<()> for Value<C> {
    fn from((): ()) -> Self {
        Self::Undefined
    }
}

impl<C: Realm> From<f64> for Value<C> {
    fn from(n: f64) -> Self {
        Self::Number(n)
    }
}

impl<C: Realm> From<bool> for Value<C> {
    fn from(b: bool) -> Self {
        Self::Boolean(b)
    }
}

impl<C: Realm> From<u8> for Value<C> {
    fn from(n: u8) -> Self {
        Self::Number(f64::from(n))
    }
}

impl<C: Realm> From<u16> for Value<C> {
    fn from(n: u16) -> Self {
        Self::Number(f64::from(n))
    }
}

impl<C: Realm> From<u32> for Value<C> {
    fn from(n: u32) -> Self {
        Self::Number(f64::from(n))
    }
}

impl<C: Realm> From<u64> for Value<C> {
    fn from(n: u64) -> Self {
        Self::Number(n as f64)
    }
}

impl<C: Realm> From<i8> for Value<C> {
    fn from(n: i8) -> Self {
        Self::Number(f64::from(n))
    }
}

impl<C: Realm> From<i16> for Value<C> {
    fn from(n: i16) -> Self {
        Self::Number(f64::from(n))
    }
}

impl<C: Realm> From<i32> for Value<C> {
    fn from(n: i32) -> Self {
        Self::Number(f64::from(n))
    }
}

impl<C: Realm> From<i64> for Value<C> {
    fn from(n: i64) -> Self {
        Self::Number(n as f64)
    }
}

impl<C: Realm> From<usize> for Value<C> {
    fn from(n: usize) -> Self {
        Self::Number(n as f64)
    }
}

impl<C: Realm> From<isize> for Value<C> {
    fn from(n: isize) -> Self {
        Self::Number(n as f64)
    }
}

impl<C: Realm> From<f32> for Value<C> {
    fn from(n: f32) -> Self {
        Self::Number(f64::from(n))
    }
}


impl<C: Realm> From<Value<C>> for Result<Value<C>, Error<C>> {
    fn from(value: Value<C>) -> Self {
        Ok(value)
    }
}

impl<O: Into<Object<C>>, C: Realm> From<O> for Value<C> {
    fn from(o: O) -> Self {
        Self::Object(o.into())
    }
}

pub trait FromValue<C: Realm>: Sized {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>>;
}

impl<C: Realm> FromValue<C> for String {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>> {
        match value {
            Value::String(s) => Ok(s),
            _ => Err(Error::ty_error(format!(
                "Expected a string, found {value:?}"
            ))),
        }
    }
}

impl<C: Realm> FromValue<C> for f64 {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>> {
        match value {
            Value::Number(n) => Ok(n),
            _ => Err(Error::ty_error(format!(
                "Expected a number, found {value:?}"
            ))),
        }
    }
}

impl<C: Realm> FromValue<C> for bool {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>> {
        match value {
            Value::Boolean(b) => Ok(b),
            _ => Err(Error::ty_error(format!(
                "Expected a boolean, found {value:?}"
            ))),
        }
    }
}

impl<C: Realm> FromValue<C> for Object<C> {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>> {
        match value {
            Value::Object(o) => Ok(o),
            _ => Err(Error::ty_error(format!(
                "Expected an object, found {value:?}"
            ))),
        }
    }
}

impl<C: Realm> FromValue<C> for () {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>> {
        match value {
            Value::Undefined => Ok(()),
            _ => Err(Error::ty_error(format!(
                "Expected undefined, found {value:?}"
            ))),
        }
    }
}

macro_rules! impl_from_value {
    ($($t:ty),*) => {
        $(
            impl<C: Realm> FromValue<C> for $t {
                fn from_value(value: Value<C>) -> Result<Self, Error<C>> {
                    match value {
                        Value::Number(n) => Ok(n as $t),
                        _ => Err(Error::ty_error(format!("Expected a number, found {:?}", value))),
                    }
                }
            }
        )*
    };
    () => {};
}

impl_from_value!(u8, u16, u32, u64, i8, i16, i32, i64, usize, isize, f32);

impl<C: Realm> FromValue<C> for Value<C> {
    fn from_value(value: Self) -> Result<Self, Error<C>> {
        Ok(value)
    }
}

impl<R: Realm, O: Obj<R>> FromValue<R> for OwningGcRefCellGuard<'_, BoxedObj<R>, O> {
    fn from_value(value: Value<R>) -> Result<Self, Error<R>> {
        let Value::Object(obj) = value else {
            return Err(Error::ty_error(format!(
                "Expected a number, found {value:?}"
            )));
        };

        obj.get_owned()?
            .maybe_map(|this| {
                let any = this.as_any();

                any.downcast_ref()
            })
            .map_err(|other| {
                Error::ty_error(format!(
                    "Expected {}, found {}",
                    type_name::<O>(),
                    other.class_name()
                ))
            })
    }
}

// impl<R: Realm, O: Obj<R>> FromValue<R> for GcMutRefCellGuard<'_, BoxedObj<R>, O> {
//     fn from_value(value: Value<R>) -> Result<Self, Error<R>> {
//         let Value::Object(obj) = value else {
//             return Err(Error::ty_error(format!(
//                 "Expected a number, found {:?}",
//                 value
//             )));
//         };
//
//         obj.get_mut()?
//             .maybe_map(|this| {
//                 let any = this.as_any_mut();
//
//                 any.downcast_mut()
//             })
//             .map_err(|other| {
//                 Error::ty_error(format!(
//                     "Expected {}, found {}",
//                     type_name::<O>(),
//                     other.class_name()
//                 ))
//             })
//     }
// }

#[derive(Eq, PartialEq, Clone, Debug)]
struct Re;

impl Realm for Re {}

#[derive(Debug)]
struct O1;

impl Obj<Re> for O1 {
    fn define_property(&mut self, name: Value<Re>, value: Value<Re>) {
        todo!()
    }

    fn define_variable(&mut self, name: Value<Re>, value: Variable<Re>) {
        todo!()
    }

    fn resolve_property(&self, name: &Value<Re>) -> Option<ObjectProperty<Re>> {
        todo!()
    }

    fn get_property(&self, name: &Value<Re>) -> Option<&Value<Re>> {
        todo!()
    }

    fn define_getter(&mut self, name: Value<Re>, value: Value<Re>) -> Result<(), Error<Re>> {
        todo!()
    }

    fn define_setter(&mut self, name: Value<Re>, value: Value<Re>) -> Result<(), Error<Re>> {
        todo!()
    }

    fn get_getter(&self, name: &Value<Re>) -> Option<Value<Re>> {
        todo!()
    }

    fn get_setter(&self, name: &Value<Re>) -> Option<Value<Re>> {
        todo!()
    }

    fn delete_property(&mut self, name: &Value<Re>) -> Option<Value<Re>> {
        todo!()
    }

    fn name(&self) -> String {
        todo!()
    }

    fn to_string(&self, realm: &mut Re) -> Result<String, Error<Re>> {
        todo!()
    }

    fn to_string_internal(&self) -> String {
        todo!()
    }

    fn properties(&self) -> Vec<(Value<Re>, Value<Re>)> {
        todo!()
    }

    fn keys(&self) -> Vec<Value<Re>> {
        todo!()
    }

    fn values(&self) -> Vec<Value<Re>> {
        todo!()
    }

    fn get_array_or_done(&self, index: usize) -> (bool, Option<Value<Re>>) {
        todo!()
    }

    fn clear_values(&mut self) {
        todo!()
    }
}

#[test]
fn conv() {
    let values: Vec<Value<Re>> = vec![];


    let v1 = FromValue::from_value(values[0].copy()).unwrap();
    let v2 = FromValue::from_value(values[0].copy()).unwrap();
    let v3: OwningGcRefCellGuard<_, O1> = FromValue::from_value(values[0].copy()).unwrap();
    // let mut v4: GcMutRefCellGuard<_> = FromValue::from_value(values[0].copy()).unwrap();

    test_func(v1, v2, &O1, &mut O1)
}

fn test_func(s: f32, a: i32, g: &O1, r: &mut O1) {}
