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
        let mut this = Self {
            inner: RefCell::new(MutableStringConstructor {
                object: MutObject::with_proto(func.copy()),
            }),
        };

        this.initialize(func.copy())?;

        Ok(this.into_object())
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

    pub fn get(&self, index: isize, to: isize) -> Option<String> {
        let inner = self.inner.borrow();
        let len = inner.string.len() as isize;

        let start = if index < 0 {
            (len + index) as usize
        } else {
            index as usize
        };

        let end = if to < 0 {
            (len + to) as usize
        } else {
            to as usize
        };

        if start > end {
            return None;
        }

        let string = inner.string.get(start..end);

        string.map(ToString::to_string)
    }
}

#[properties_new(constructor(StringConstructor::new))]
impl StringObj {
    pub fn anchor(&self, name: String) -> ValueResult {
        Ok(format!("<a name=\"{}\">{}</a>", name, self.inner.borrow().string).into())
    }

    pub fn at(&self, index: isize) -> Value {
        self.get(index, index + 1)
            .map_or(Value::Undefined, Into::into)
    }

    pub fn big(&self) -> ValueResult {
        Ok(format!("<big>{}</big>", self.inner.borrow().string).into())
    }

    pub fn blink(&self) -> ValueResult {
        Ok(format!("<blink>{}</blink>", self.inner.borrow().string).into())
    }

    pub fn bold(&self) -> ValueResult {
        Ok(format!("<b>{}</b>", self.inner.borrow().string).into())
    }

    #[prop("charAt")]
    pub fn char_at(&self, index: isize) -> Value {
        self.get(index, index + 1)
            .map_or(Value::Undefined, Into::into)
    }

    #[prop("charCodeAt")]
    pub fn char_code_at(&self, index: isize) -> Value {
        self.get(index, index + 1)
            .map(|s| s.chars().next().map(|c| c as u32).unwrap_or_default())
            .unwrap_or_default()
            .into()
    }

    #[prop("codePointAt")]
    pub fn code_point_at(&self, index: isize) -> Value {
        self.get(index, index + 1)
            .map(|s| s.chars().next().map(|c| c as u32).unwrap_or_default())
            .unwrap_or_default()
            .into()
    }

    #[prop("concat")]
    pub fn concat(&self, #[variadic] args: &[Value], #[realm] realm: &mut Realm) -> ValueResult {
        let inner = self.inner.borrow();
        let mut string = inner.string.clone();

        for arg in args {
            string.push_str(&arg.to_string(realm)?);
        }

        Ok(string.into())
    }

    #[prop("endsWith")]
    pub fn ends_with(&self, search: String) -> Value {
        let inner = self.inner.borrow();

        inner.string.ends_with(&search).into()
    }

    #[prop("fixed")]
    pub fn fixed(&self) -> ValueResult {
        Ok(format!("<tt>{}</tt>", self.inner.borrow().string).into())
    }

    #[prop("fontcolor")]
    pub fn font_color(&self, color: String) -> ValueResult {
        Ok(format!(
            "<font color=\"{}\">{}</font>",
            color,
            self.inner.borrow().string
        )
        .into())
    }

    #[prop("fontsize")]
    pub fn font_size(&self, size: String) -> ValueResult {
        Ok(format!(
            "<font size=\"{}\">{}</font>",
            size,
            self.inner.borrow().string
        )
        .into())
    }

    #[prop("includes")]
    pub fn includes(&self, search: String) -> bool {
        let inner = self.inner.borrow();

        inner.string.contains(&search)
    }

    #[prop("indexOf")]
    pub fn index_of(&self, search: String, from: isize) -> isize {
        let inner = self.inner.borrow();

        let from = if from < 0 {
            (inner.string.len() as isize + from) as usize
        } else {
            from as usize
        };

        inner.string[from..]
            .find(&search)
            .map_or(-1, |i| i as isize)
    }

    #[prop("isWellFormed")]
    pub fn is_well_formed(&self) -> bool {
        // check if we have any lone surrogates => between 0xD800-0xDFFF or 0xDC00-0xDFFF
        self.inner.borrow().string.chars().all(|c| {
            let c = c as u32;
            !(0xD800..=0xDFFF).contains(&c) || (0xDC00..=0xDFFF).contains(&c)
        })
    }

    #[prop("italics")]
    pub fn italics(&self) -> ValueResult {
        Ok(format!("<i>{}</i>", self.inner.borrow().string).into())
    }

    #[prop("lastIndexOf")]
    pub fn last_index_of(&self, search: String, from: isize) -> isize {
        let inner = self.inner.borrow();

        let from = if from < 0 {
            (inner.string.len() as isize + from) as usize
        } else {
            from as usize
        };

        inner.string[..from]
            .rfind(&search)
            .map_or(-1, |i| i as isize)
    }

    #[prop("link")]
    pub fn link(&self, url: String) -> ValueResult {
        Ok(format!("<a href=\"{}\">{}</a>", url, self.inner.borrow().string).into())
    }

    // #[prop("localeCompare")]
    // pub fn locale_compare(&self, other: String) -> isize {
    //TODO: localization
    // }

    //TODO: match, matchAll

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
