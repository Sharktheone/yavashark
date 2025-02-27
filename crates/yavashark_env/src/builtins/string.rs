use crate::array::Array;
use crate::{Error, MutObject, Object, ObjectHandle, Realm, Value, ValueResult};
use std::cell::RefCell;
use std::cmp;
use unicode_normalization::UnicodeNormalization;
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
    pub fn anchor(&self, name: &str) -> ValueResult {
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
    pub fn concat(&self, args: &[Value], #[realm] realm: &mut Realm) -> ValueResult {
        let inner = self.inner.borrow();
        let mut string = inner.string.clone();

        for arg in args {
            string.push_str(&arg.to_string(realm)?);
        }

        Ok(string.into())
    }

    #[prop("endsWith")]
    pub fn ends_with(&self, search: &str) -> Value {
        let inner = self.inner.borrow();

        inner.string.ends_with(&search).into()
    }

    #[prop("fixed")]
    pub fn fixed(&self) -> ValueResult {
        Ok(format!("<tt>{}</tt>", self.inner.borrow().string).into())
    }

    #[prop("fontcolor")]
    pub fn font_color(&self, color: &str) -> ValueResult {
        Ok(format!(
            "<font color=\"{}\">{}</font>",
            color,
            self.inner.borrow().string
        )
        .into())
    }

    #[prop("fontsize")]
    pub fn font_size(&self, size: &str) -> ValueResult {
        Ok(format!(
            "<font size=\"{}\">{}</font>",
            size,
            self.inner.borrow().string
        )
        .into())
    }

    #[prop("includes")]
    pub fn includes(&self, search: &str) -> bool {
        let inner = self.inner.borrow();

        inner.string.contains(search)
    }

    #[prop("indexOf")]
    pub fn index_of(&self, search: &str, from: Option<isize>) -> isize {
        let from = from.unwrap_or(0);
        let inner = self.inner.borrow();

        let from = if from < 0 {
            (inner.string.len() as isize + from) as usize
        } else {
            from as usize
        };

        inner
            .string
            .get(from..)
            .and_then(|s| s.find(search))
            .map_or(-1, |i| i as isize + from as isize)
    }

    #[prop("isWellFormed")]
    pub fn is_well_formed(&self) -> bool {
        // check if we have any lone surrogates => between 0xD800-0xDFFF or 0xDC00-0xDFFF
        self.inner
            .borrow()
            .string
            .chars()
            .all(|c| !is_lone_surrogate(c))
    }

    #[prop("italics")]
    pub fn italics(&self) -> ValueResult {
        Ok(format!("<i>{}</i>", self.inner.borrow().string).into())
    }

    #[prop("lastIndexOf")]
    pub fn last_index_of(&self, search: &str, from: Option<isize>) -> isize {
        let inner = self.inner.borrow();

        let from = from.unwrap_or(-1);

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
    pub fn link(&self, url: &str) -> ValueResult {
        Ok(format!("<a href=\"{}\">{}</a>", url, self.inner.borrow().string).into())
    }

    // #[prop("localeCompare")]
    // pub fn locale_compare(&self, other: String) -> isize {
    //TODO: localization
    // }

    // #[prop("match")]
    // pub fn match_(&self, pattern: &RegExp, #[realm] realm: &mut Realm) -> ValueResult {
    //     //TODO: Symbol.search
    //     let inner = self.inner.borrow();
    //
    //     pattern.exec(&inner.string, realm)
    // }

    // #[prop("matchAll")]
    // pub fn match_all(&self, pattern: &RegExp, #[realm] realm: &mut Realm) -> ValueResult {
    //     //TODO
    // }

    pub fn normalize(&self, form: &str) -> ValueResult {
        let inner = self.inner.borrow();

        let form = match form {
            "NFC" => inner.string.nfc().to_string(),
            "NFD" => inner.string.nfd().to_string(),
            "NFKC" => inner.string.nfkc().to_string(),
            "NFKD" => inner.string.nfkd().to_string(),
            _ => return Err(Error::range("Invalid normalization form")),
        };

        Ok(form.into())
    }

    #[prop("padEnd")]
    pub fn pad_end(&self, target_length: usize, pad_string: &Option<String>) -> ValueResult {
        let inner = self.inner.borrow();

        let pad_string = pad_string.as_deref().unwrap_or(" ");

        let pad_len = target_length.saturating_sub(inner.string.len());

        let pad = pad_string.repeat(pad_len);

        Ok(format!("{}{}", inner.string, pad).into())
    }

    #[prop("padStart")]
    pub fn pad_start(&self, target_length: usize, pad_string: &Option<String>) -> ValueResult {
        let inner = self.inner.borrow();

        let pad_string = pad_string.as_deref().unwrap_or(" ");

        let pad_len = target_length.saturating_sub(inner.string.len());

        let pad = pad_string.repeat(pad_len);

        Ok(format!("{}{}", pad, inner.string).into())
    }

    pub fn repeat(&self, count: usize) -> ValueResult {
        let inner = self.inner.borrow();

        Ok(inner.string.repeat(count).into())
    }

    pub fn replace(&self, search: &str, replace: &str) -> ValueResult {
        let inner = self.inner.borrow();

        Ok(inner.string.replace(search, replace).into())
    }

    // pub fn search(&self, pattern: &RegExp, #[realm] realm: &mut Realm) -> ValueResult {
    //     //TODO: Symbol.search
    //
    //     let inner = self.inner().try_borrow()?;
    //
    //     Ok(pattern.regex.find(&inner.string)
    //         .map(|m| m.start() as isize)
    //         .unwrap_or(-1)
    //         .into())
    // }

    pub fn slice(&self, start: isize, end: Option<isize>) -> ValueResult {
        let inner = self.inner.borrow();

        // negative numbers are counted from the end of the string
        let start = if start < 0 {
            (inner.string.len() as isize + start) as usize
        } else {
            start as usize
        };

        let end = end.map_or(inner.string.len(), |end| {
            if end < 0 {
                (inner.string.len() as isize + end) as usize
            } else {
                end as usize
            }
        });

        let end = cmp::min(end, inner.string.len());

        let string = inner.string.get(start..end);

        Ok(string.unwrap_or_default().into())
    }

    pub fn small(&self) -> ValueResult {
        Ok(format!("<small>{}</small>", self.inner.borrow().string).into())
    }

    pub fn split(
        &self,
        separator: &str,
        limit: Option<usize>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        let inner = self.inner.borrow();

        let limit = limit.unwrap_or(usize::MAX);

        let parts = inner.string.splitn(limit, separator);

        let mut array = Vec::new();

        for part in parts {
            array.push(part.into());
        }

        Ok(Array::with_elements(realm, array)?.into_value())
    }

    #[prop("startsWith")]
    pub fn starts_with(&self, search: &str) -> bool {
        let inner = self.inner.borrow();

        inner.string.starts_with(search)
    }

    pub fn strike(&self) -> ValueResult {
        Ok(format!("<strike>{}</strike>", self.inner.borrow().string).into())
    }

    pub fn sub(&self) -> ValueResult {
        Ok(format!("<sub>{}</sub>", self.inner.borrow().string).into())
    }

    pub fn substr(&self, start: isize, len: Option<isize>) -> ValueResult {
        let inner = self.inner.borrow();

        // negative numbers are counted from the end of the string
        let start = if start < 0 {
            (inner.string.len() as isize + start) as usize
        } else {
            start as usize
        };

        let end = len.map_or(inner.string.len(), |len| start + len as usize);

        let end = cmp::min(end, inner.string.len());

        let string = inner.string.get(start..end);

        Ok(string.unwrap_or_default().into())
    }

    pub fn substring(&self, start: isize, end: Option<isize>) -> ValueResult {
        let inner = self.inner.borrow();

        // negative numbers are counted from the end of the string
        let start = if start < 0 {
            (inner.string.len() as isize + start) as usize
        } else {
            start as usize
        };

        let end = end.map_or(inner.string.len(), |end| {
            if end < 0 {
                (inner.string.len() as isize + end) as usize
            } else {
                end as usize
            }
        });

        let end = cmp::min(end, inner.string.len());

        let string = inner.string.get(start..end);

        Ok(string.unwrap_or_default().into())
    }

    pub fn sup(&self) -> ValueResult {
        Ok(format!("<sup>{}</sup>", self.inner.borrow().string).into())
    }

    #[prop("toLowerCase")]
    pub fn to_lower_case(&self) -> ValueResult {
        let inner = self.inner.borrow();

        Ok(inner.string.to_lowercase().into())
    }

    #[prop("toString")]
    pub fn to_string(&self) -> Value {
        self.inner.borrow().string.clone().into()
    }

    #[prop("toUpperCase")]
    pub fn to_upper_case(&self) -> ValueResult {
        let inner = self.inner.borrow();

        Ok(inner.string.to_uppercase().into())
    }

    #[prop("toWellFormed")]
    pub fn to_well_formed(&self) -> ValueResult {
        let inner = self.inner.borrow();

        let well_formed = inner
            .string
            .chars()
            .map(|c| if is_lone_surrogate(c) { '\u{FFFD}' } else { c })
            .collect::<String>();

        Ok(well_formed.into())
    }

    pub fn trim(&self) -> ValueResult {
        let inner = self.inner.borrow();

        Ok(inner.string.trim().into())
    }

    #[prop("trimEnd")]
    pub fn trim_end(&self) -> ValueResult {
        let inner = self.inner.borrow();

        Ok(inner.string.trim_end().into())
    }

    #[prop("trimStart")]
    pub fn trim_start(&self) -> ValueResult {
        let inner = self.inner.borrow();

        Ok(inner.string.trim_start().into())
    }

    #[prop("valueOf")]
    pub fn value_of(&self) -> Value {
        self.inner.borrow().string.clone().into()
    }
}

fn is_lone_surrogate(c: char) -> bool {
    let c = c as u32;
    (0xD800..=0xDFFF).contains(&c) || (0xDC00..=0xDFFF).contains(&c)
}
