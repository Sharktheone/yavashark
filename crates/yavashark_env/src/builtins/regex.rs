use crate::array::Array;
use crate::{ControlFlow, Error, MutObject, Object, ObjectHandle, Realm, Res, Value, ValueResult};
use regress::{Range, Regex};
use std::cell::{Cell, RefCell};
use yavashark_macro::{object, properties_new};
use yavashark_string::YSString;
use yavashark_value::{Constructor, Func, Obj};

#[object()]
#[derive(Debug)]
pub struct RegExp {
    regex: Regex,
    flags: Flags,

    last_index: Cell<usize>,
    source: YSString,
    flags_str: YSString,
}

#[derive(Debug, Copy, Clone, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct Flags {
    pub icase: bool,
    pub multiline: bool,
    pub dot_all: bool,
    pub no_opt: bool,
    pub unicode: bool,
    pub unicode_sets: bool,
    pub global: bool,
    pub sticky: bool,
    pub has_indecies: bool,
}

impl From<Flags> for regress::Flags {
    fn from(value: Flags) -> Self {
        Self {
            icase: value.icase,
            multiline: value.multiline,
            dot_all: value.dot_all,
            no_opt: value.no_opt,
            unicode: value.unicode,
            unicode_sets: value.unicode_sets,
        }
    }
}

impl From<&str> for Flags {
    fn from(flags: &str) -> Self {
        let mut flags_obj = Flags::default();

        for c in flags.chars() {
            match c {
                'i' => flags_obj.icase = true,
                'm' => flags_obj.multiline = true,
                's' => flags_obj.dot_all = true,
                'n' => flags_obj.no_opt = true,
                'u' => flags_obj.unicode = true,
                'v' => flags_obj.unicode_sets = true,
                'g' => flags_obj.global = true,
                'y' => flags_obj.sticky = true,
                'd' => flags_obj.has_indecies = true,
                _ => {}
            }
        }

        flags_obj
    }
}

impl RegExp {
    #[allow(clippy::new_ret_no_self)]
    #[must_use]
    pub fn new(
        realm: &Realm,
        regex: Regex,
        flags: Flags,
        source: YSString,
        flags_str: YSString,
    ) -> ObjectHandle {
        Self {
            regex,
            inner: RefCell::new(MutableRegExp {
                object: MutObject::with_proto(realm.intrinsics.regexp.clone().into()),
            }),
            flags,
            source,
            flags_str,
            last_index: Cell::new(0),
        }
        .into_object()
    }

    pub fn new_from_str(realm: &Realm, source: &str) -> Res<ObjectHandle> {
        let regex = Regex::new(source).map_err(|e| ControlFlow::error(e.to_string()))?;

        Ok(Self::new(
            realm,
            regex,
            Flags::default(),
            YSString::from_ref(source),
            YSString::new(),
        ))
    }

    pub fn new_from_str_with_flags(
        realm: &Realm,
        source: &str,
        flags_str: &str,
    ) -> Res<ObjectHandle> {
        let flags = Flags::from(flags_str);

        let regex = Regex::from_unicode(source.chars().map(u32::from), flags)
            .map_err(|e| Error::syn_error(e.text))?;

        Ok(Self::new(
            realm,
            regex,
            flags,
            YSString::from_ref(source),
            YSString::from_ref(flags_str),
        ))
    }
}

#[object(constructor, function, to_string)]
#[derive(Debug)]
pub struct RegExpConstructor {}

#[properties_new(raw)]
impl RegExpConstructor {
    fn escape(value: &str) -> String {
        escape(value)
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
    pub fn exec(&self, value: YSString, #[realm] realm: &mut Realm) -> ValueResult {
        if !self.flags.global && !self.flags.sticky {
            self.last_index.set(0);
        }

        let input = value.as_str();

        if self.last_index.get() > input.len() {
            self.last_index.set(0);
        }

        let Some(m) = self.regex.find_from(input, self.last_index.get()).next() else {
            if self.flags.global || self.flags.sticky {
                self.last_index.set(0);
            }

            return Ok(Value::Undefined);
        };

        if self.flags.sticky && m.start() != self.last_index.get() {
            self.last_index.set(0);

            return Ok(Value::Undefined);
        }

        self.last_index.set(m.start());

        if self.flags.global || self.flags.sticky {
            self.last_index.set(m.end());
        }

        let a = Array::with_len(realm, m.captures.len() + 1)?;

        a.define_property("index".into(), self.last_index.get().into())?;
        a.define_property("input".into(), value.clone().into())?;

        let matches = Array::with_elements(realm, vec![m.start().into(), m.end().into()])?;

        let indices = Array::with_len(realm, m.captures.len() + 1)?;

        indices.push(matches.into_value())?;

        let mut named_groups = m.named_groups().collect::<Vec<(&str, Option<Range>)>>();
        named_groups.sort_by(|(a, _), (b, _)| a.cmp(b));

        let (groups, group_names) = if named_groups.is_empty() {
            (Value::Undefined, Value::Undefined)
        } else {
            todo!();
        };

        indices.define_property("groups".into(), groups)?;
        indices.define_property("groupNames".into(), group_names)?;

        for i in 1..=m.captures.len() {
            let capture = m.group(i);

            let captured = capture
                .clone()
                .and_then(|c| input.get(c))
                .map_or(Value::Undefined, |s| YSString::from_ref(s).into());

            a.push(captured)?;

            if self.flags.has_indecies {
                let range = capture
                    .map(|range| {
                        Array::with_elements(realm, vec![range.start.into(), range.end.into()])
                    })
                    .transpose()?
                    .map_or(Value::Undefined, Obj::into_value);

                indices.push(range)?;
            }
        }

        if self.flags.has_indecies {
            a.define_property("indices".into(), indices.into_value())?;
        }

        Ok(a.into_value())
    }

    #[prop("test")]
    pub fn test(&self, value: &str) -> bool {
        self.regex
            .find_from(value, self.last_index.get())
            .next()
            .is_some_and(|m| {
                if self.flags.sticky && m.start() != self.last_index.get() {
                    return false;
                }

                if self.flags.global || self.flags.sticky {
                    self.last_index.set(m.end());
                }

                true
            })
    }

    #[prop("toString")]
    pub fn js_to_string(&self) -> YSString {
        let mut source = self.source.clone();

        source.push_str(self.flags_str.clone());

        source
    }

    #[get("global")]
    pub const fn global(&self) -> bool {
        self.flags.global
    }

    #[get("lastIndex")]
    pub fn last_index(&self) -> usize {
        self.last_index.get()
    }
}

#[must_use]
pub fn escape(text: &str) -> String {
    let mut buf = String::with_capacity(text.len());

    for c in text.chars() {
        if is_meta_character(c) {
            buf.push('\\');
        }
        buf.push(c);
    }

    buf
}

#[must_use]
pub const fn is_meta_character(c: char) -> bool {
    matches!(
        c,
        '\\' | '.'
            | '+'
            | '*'
            | '?'
            | '('
            | ')'
            | '|'
            | '['
            | ']'
            | '{'
            | '}'
            | '^'
            | '$'
            | '#'
            | '&'
            | '-'
            | '~'
    )
}
