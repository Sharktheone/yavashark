use crate::{ControlFlow, MutObject, Object, ObjectHandle, Realm, Result, Value, ValueResult};
use regex::{Regex, RegexBuilder};
use std::cell::RefCell;
use yavashark_macro::{object, properties_new};
use yavashark_value::{Constructor, Func, IntoValue, Obj};
use crate::array::Array;

#[object(direct(last_index(lastIndex)))]
#[derive(Debug)]
pub struct RegExp {
    regex: Regex,
    global: bool,
}

impl RegExp {
    #[allow(clippy::new_ret_no_self)]
    #[must_use]
    pub fn new(realm: &Realm, regex: Regex, global: bool) -> ObjectHandle {
        Self {
            regex,
            inner: RefCell::new(MutableRegExp {
                object: MutObject::with_proto(realm.intrinsics.regexp.clone().into()),
                last_index: Value::from(0u8).into(),
            }),
            global,
        }
        .into_object()
    }

    pub fn new_from_str(realm: &Realm, regex: &str) -> Result<ObjectHandle> {
        let regex = Regex::new(regex).map_err(|e| ControlFlow::error(e.to_string()))?;

        Ok(Self::new(realm, regex, false))
    }

    pub fn new_from_str_with_flags(
        realm: &Realm,
        regex: &str,
        flags: &str,
    ) -> Result<ObjectHandle> {
        let regex = RegexBuilder::new(regex)
            .case_insensitive(flags.contains('i'))
            .multi_line(flags.contains('m'))
            .dot_matches_new_line(flags.contains('s'))
            .ignore_whitespace(flags.contains('x'))
            .unicode(flags.contains('u'))
            .build()
            .map_err(|e| ControlFlow::error(e.to_string()))?;

        let global = flags.contains('g');

        Ok(Self::new(realm, regex, global))
    }
}

#[object(constructor, function)]
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
        let regex = args
            .first()
            .map_or(Ok(String::new()), |v| v.to_string(realm))?;

        let flags = args
            .get(1)
            .map_or(Ok(String::new()), |v| v.to_string(realm))?;

        let obj = RegExp::new_from_str_with_flags(realm, &regex, &flags)?;

        Ok(obj.into())
    }
}

impl Func<Realm> for RegExpConstructor {
    fn call(&self, realm: &mut Realm, args: Vec<Value>, _this: Value) -> ValueResult {
        Constructor::construct(self, realm, args)
    }
}

#[properties_new(constructor(RegExpConstructor::new))]
impl RegExp {
    #[prop("exec")]
    fn exec(&self, value: String, #[realm] realm: &mut Realm) -> ValueResult {
        if self.global {
            let mut inner = self.inner.borrow_mut();
            
            let  mut last_index = inner.last_index.value.to_number(realm)? as usize;
            
            let value = value.get(last_index..).unwrap_or_default();
            
            let value = self
                .regex
                .find(value)
                .map(|m| {
                    last_index += m.end();
                    
                    m.as_str().to_string()
                });
            
            let Some(value) = value else {
                inner.last_index.value = Value::from(0u8);
                
                return Ok(Value::Null);
            };
            
            inner.last_index.value = Value::from(last_index);
            
            let array = Array::with_elements(realm, vec![value.into_value()])?;
            
            return Ok(array.into_value())
        }

        let value = self
            .regex
            .find(&value)
            .map_or_else(|| "".to_string(), |m| m.as_str().to_string());

        Ok(value.into_value())
    }

    #[prop("test")]
    fn test(&self, value: String) -> bool {
        self.regex.is_match(&value)
    }
}
