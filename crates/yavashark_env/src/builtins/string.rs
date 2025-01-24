use crate::{MutObject, Object, ObjectHandle, Realm, Value, ValueResult};
use std::cell::RefCell;
use yavashark_macro::{object, properties_new};
use yavashark_value::{Constructor, Func, Obj};

#[object]
#[derive(Debug)]
pub struct StringObj {
    #[mutable]
    #[primitive]
    string: String,
}




#[object(constructor, function)]
#[derive(Debug)]
pub struct StringConstructor {}

impl StringConstructor {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(_: &Object, func: &Value) -> crate::Result<ObjectHandle> {
        Ok(Self {
            inner: RefCell::new(MutableStringConstructor {
                object: MutObject::with_proto(func.copy()),
            }),
        }
        .into_object())
    }
}

#[properties_new(raw)]
impl StringConstructor {
    #[prop("fromCharCode")]
    fn from_char_code(#[variadic] args: &[Value], #[realm] realm: &mut Realm) -> String {
        args.iter()
            .map(|v| v.to_number(realm).unwrap_or_default() as u32)
            .filter_map(std::char::from_u32)
            .collect::<String>()
    }
    
    #[prop("fromCodePoint")]
    fn from_char_point(#[variadic] args: &[Value], #[realm] realm: &mut Realm) -> String {
        args.iter()
            .map(|v| v.to_number(realm).unwrap_or_default() as u32)
            .filter_map(std::char::from_u32)
            .collect::<String>()
    }
    
    //TODO: String.raw
}


impl Constructor<Realm> for StringConstructor {
    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> ValueResult {
        let str = match args.first() {
            Some(v) => v.to_string(realm)?,
            None => String::new(),
        };

        let obj = StringObj::with_string(realm, str)?;

        Ok(obj.into())
    }
}

impl Func<Realm> for StringConstructor {
    fn call(&self, realm: &mut Realm, args: Vec<Value>, _this: Value) -> ValueResult {
        let str = match args.first() {
            Some(v) => v.to_string(realm)?,
            None => String::new(),
        };

        Ok(str.into())
    }
}

impl StringObj {
    #[allow(clippy::new_ret_no_self, dead_code)]
    pub fn new(realm: &Realm) -> crate::Result<ObjectHandle> {
        Self::with_string(realm, String::new())
    }

    pub fn with_string(realm: &Realm, string: String) -> crate::Result<ObjectHandle> {
        let this = Self {
            inner: RefCell::new(MutableStringObj {
                object: MutObject::with_proto(realm.intrinsics.string.clone().into()),
                string,
            }),
        };

        Ok(this.into_object())
    }
}

#[properties_new(constructor(StringConstructor::new))]
impl StringObj {
    pub fn substr(&self, start: isize) -> ValueResult {
        let inner = self.inner.borrow();

        // negative numbers are counted from the end of the string
        let start = if start < 0 {
            (inner.string.len() as isize + start) as usize
        } else {
            start as usize
        };

        let string = inner.string.get(start..);

        Ok(string.unwrap_or_default().into())
    }
}
