use std::cell::RefCell;
use regex::{Regex, RegexBuilder};
use yavashark_macro::{object, properties_new};
use yavashark_value::{Constructor, Obj};
use crate::{ControlFlow, MutObject, Object, ObjectHandle, Realm, Value, ValueResult};

#[object]
#[derive(Debug)]
pub struct RegExp {
    regex: Regex,
}


impl RegExp {
    #[allow(clippy::new_ret_no_self)]
    #[must_use]
    pub fn new(realm: &Realm, regex: Regex) -> ObjectHandle {
        Self {
            regex,
            inner: RefCell::new(MutableRegExp {
                object: MutObject::with_proto(realm.intrinsics.regexp.clone().into()),
            }),
        }.into_object()
    }
}

#[object(constructor)]
#[derive(Debug)]
pub struct RegExpConstructor {}


#[properties_new(raw)]
impl RegExpConstructor {
    fn escape(value: String) -> String {
        regex::escape(&value)
    }
}

impl RegExpConstructor {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(_: &Object, func: &Value) -> crate::Result<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableRegExpConstructor {
                object: MutObject::with_proto(func.copy()),
            }),
        };
        
        this.initialize(func.copy())?;

        Ok(this.into_object())
    }
}

impl Constructor<Realm> for RegExpConstructor {
    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> ValueResult {
        let regex = args.first().map_or(Ok(String::new()), |v| v.to_string(realm))?;

        let flags = args.get(1).map_or(Ok(String::new()), |v| v.to_string(realm))?;



        let regex = RegexBuilder::new(&regex)
            .case_insensitive(flags.contains('i'))
            .multi_line(flags.contains('m'))
            .dot_matches_new_line(flags.contains('s'))
            .ignore_whitespace(flags.contains('x'))
            .unicode(flags.contains('u'))
            .build()
            .map_err(|e| ControlFlow::error(e.to_string()))?;



        let obj = RegExp::new(realm, regex);

        Ok(obj.into())
    }
}


#[properties_new(constructor(RegExpConstructor::new))]
impl RegExp {

}
