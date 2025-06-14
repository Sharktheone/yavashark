use crate::array::Array;
use crate::{ControlFlow, MutObject, Object, ObjectHandle, Realm, Res, Value, ValueResult};
use regex::{Regex, RegexBuilder};
use std::cell::RefCell;
use yavashark_macro::{object, properties_new};
use yavashark_string::YSString;
use yavashark_value::{Constructor, Func, IntoValue, Obj};

#[object(direct(last_index(lastIndex), global))]
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
                global: global.into(),
            }),
            global,
        }
        .into_object()
    }

    pub fn new_from_str(realm: &Realm, regex: &str) -> Res<ObjectHandle> {
        let regex = Regex::new(regex).map_err(|e| ControlFlow::error(e.to_string()))?;

        Ok(Self::new(realm, regex, false))
    }

    pub fn new_from_str_with_flags(realm: &Realm, regex: &str, flags: &str) -> Res<ObjectHandle> {
        let regex = RegexBuilder::new(regex)
            .case_insensitive(flags.contains('i'))
            .multi_line(flags.contains('m'))
            .dot_matches_new_line(flags.contains('s'))
            .ignore_whitespace(flags.contains('x'))
            .unicode(flags.contains('u'))
            .build()
            .map_err(|e| ControlFlow::error_syntax(e.to_string()))?;

        let global = flags.contains('g');

        Ok(Self::new(realm, regex, global))
    }
}

#[object(constructor, function, to_string)]
#[derive(Debug)]
pub struct RegExpConstructor {}

#[properties_new(raw)]
impl RegExpConstructor {
    fn escape(value: &str) -> String {
        regex::escape(value)
    }
}

impl RegExpConstructor {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(_: &Object, func: &Value) -> crate::Res<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableRegExpConstructor {
                object: MutObject::with_proto(func.copy()),
            }),
        };

        this.initialize(func.copy())?;

        Ok(this.into_object())
    }

    #[allow(clippy::unused_self)]
    fn override_to_string_internal(&self) -> Res<YSString> {
        Ok("function RegExp() { [native code] }".into())
    }

    #[allow(clippy::unused_self)]
    fn override_to_string(&self, _: &mut Realm) -> Res<YSString> {
        Ok("function RegExp() { [native code] }".into())
    }
}

impl Constructor<Realm> for RegExpConstructor {
    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> ValueResult {
        let regex = args.first().map_or(Res::<String>::Ok(String::new()), |v| {
            Ok(v.to_string(realm)?.to_string())
        })?;

        let flags = args.get(1).map_or(Res::<String>::Ok(String::new()), |v| {
            Ok(v.to_string(realm)?.to_string())
        })?;

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
    pub fn exec(&self, value: &str, #[realm] realm: &mut Realm) -> ValueResult {
        if self.global {
            let mut inner = self.inner.borrow_mut();

            let mut last_index = inner.last_index.value.to_number(realm)? as usize;

            let value = value.get(last_index..).unwrap_or_default();

            let value = self.regex.find(value).map(|m| {
                last_index += m.end();

                m.as_str().to_string()
            });

            let Some(value) = value else {
                inner.last_index.value = Value::from(0u8);

                return Ok(Value::Null);
            };

            inner.last_index.value = Value::from(last_index);

            let array = Array::with_elements(realm, vec![value.into_value()])?;

            return Ok(array.into_value());
        }

        let value = self
            .regex
            .find(value)
            .map_or_else(String::new, |m| m.as_str().to_string());

        Ok(value.into_value())
    }

    #[prop("test")]
    pub fn test(&self, value: &str) -> bool {
        self.regex.is_match(value)
    }

    #[prop("toString")]
    pub fn js_to_string(&self) -> String {
        let str = self.regex.to_string();

        if str.is_empty() {
            return "/(?:)/".to_string();
        }

        format!("/{}/{}", str, if self.global { "g" } else { "" })
    }
}
