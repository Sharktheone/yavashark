use crate::array::Array;
use crate::{
    Error, MutObject, Object, ObjectHandle, ObjectProperty, Realm, Res, Value, ValueResult,
};
use std::cell::{RefCell, RefMut};
use std::cmp;
use std::ops::{Deref, DerefMut};
use unicode_normalization::UnicodeNormalization;
use yavashark_macro::{object, properties_new};
use yavashark_value::{Constructor, CustomName, Func, MutObj, Obj};
use crate::utils::ArrayLike;

#[derive(Debug)]
pub struct StringObj {
    pub inner: RefCell<MutableStringObj>,
}

#[derive(Debug)]
pub struct MutableStringObj {
    pub object: MutObject,
    string: String,
}

impl Deref for MutableStringObj {
    type Target = MutObject;

    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl DerefMut for MutableStringObj {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.object
    }
}

impl yavashark_value::ObjectImpl<Realm> for StringObj {
    type Inner = MutableStringObj;

    fn get_wrapped_object(&self) -> impl DerefMut<Target = impl MutObj<Realm>> {
        RefMut::map(self.inner.borrow_mut(), |inner| &mut inner.object)
    }

    fn get_inner(&self) -> impl Deref<Target = Self::Inner> {
        self.inner.borrow()
    }

    fn get_inner_mut(&self) -> impl DerefMut<Target = Self::Inner> {
        self.inner.borrow_mut()
    }

    fn resolve_property(&self, name: &Value) -> Res<Option<ObjectProperty>> {
        if let Value::Number(n) = name {
            let index = *n as isize;

            return Ok(Some(self.at(index).into()));
        }

        self.get_wrapped_object().resolve_property(name)
    }

    fn get_property(&self, name: &Value) -> Res<Option<ObjectProperty>> {
        if let Value::Number(n) = name {
            let index = *n as isize;

            return Ok(Some(self.at(index).into()));
        }

        self.get_wrapped_object().get_property(name)
    }

    fn name(&self) -> String {
        "String".to_string()
    }

    fn primitive(&self) -> Option<yavashark_value::Value<Realm>> {
        Some(self.inner.borrow().string.clone().into())
    }
}

#[object(constructor, function, to_string, name)]
#[derive(Debug)]
pub struct StringConstructor {}

impl CustomName for StringConstructor {
    fn custom_name(&self) -> String {
        "String".to_string()
    }
}

impl StringConstructor {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(_: &Object, func: &Value) -> crate::Res<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableStringConstructor {
                object: MutObject::with_proto(func.copy()),
            }),
        };

        this.define_property("name".into(), "String".into())?;

        this.initialize(func.copy())?;

        Ok(this.into_object())
    }

    pub fn override_to_string(&self, _: &mut Realm) -> Res<String> {
        Ok("function String() { [native code] }".to_string())
    }

    pub fn override_to_string_internal(&self) -> Res<String> {
        Ok("function String() { [native code] }".to_string())
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
    
    fn raw(
        template: ObjectHandle,
        #[variadic] args: &[Value],
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        //Let substitutionCount be the number of elements in substitutions.
        let substitution_count = args.len();
        
        // 2. Let cooked be ? ToObject(template).
        let cooked = template;
        
        // 3. Let literals be ? ToObject(? Get(cooked, "raw")).
        let literals = cooked.get("raw", realm)?;
        let mut literals = ArrayLike::new(literals, realm)?;
        
        // 4. Let literalCount be ? LengthOfArrayLike(literals).
        let literal_count = literals.len();
        
        // 5. If literalCount ≤ 0, return the empty String.
        if literal_count == 0 { //length from ArrayLike.len can't be under 0
            return Ok(Value::String(String::new()));
        }
        
        // 6. Let R be the empty String.
        let mut r = String::new();
        
        // 7. Let nextIndex be 0.
        let mut next_index = 0;
        
        // 8. Repeat,
        loop {
            //a. Let nextLiteralVal be ? Get(literals, ! ToString(𝔽(nextIndex))).
            let next_literal_val = literals.next(realm)?.unwrap_or(Value::Undefined);
            
            //b. Let nextLiteral be ? ToString(nextLiteralVal).
            let next_literal = next_literal_val.to_string(realm)?;
            // c. Set R to the string-concatenation of R and nextLiteral.
            r.push_str(&next_literal);
            
            //d. If nextIndex + 1 = literalCount, return R.
            if next_index + 1 == literal_count {
                return Ok(r.into());
            }
            
            //e. If nextIndex < substitutionCount, then
            if next_index < substitution_count {
                //i. Let nextSubVal be substitutions[nextIndex].
                let next_sub_val = args.get(next_index).unwrap_or(&Value::Undefined);

                //ii. Let nextSub be ? ToString(nextSubVal).
                let next_sub = next_sub_val.to_string(realm)?;

                //iii. Set R to the string-concatenation of R and nextSub.
                r.push_str(&next_sub);
            }
            
            //f. Set nextIndex to nextIndex + 1.
            next_index += 1;
        }
    }
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
    pub fn new(realm: &Realm) -> crate::Res<ObjectHandle> {
        Self::with_string(realm, String::new())
    }

    pub fn with_string(realm: &Realm, string: String) -> crate::Res<ObjectHandle> {
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

    pub fn get_single(&self, index: isize) -> Option<String> {
        let inner = self.inner.borrow();
        let len = inner.string.len() as isize;

        let start = if index < 0 {
            (len + index) as usize
        } else {
            index as usize
        };

        let end = start + 1;

        let string = inner.string.get(start..end);

        string.map(ToString::to_string)
    }
}

#[properties_new(constructor(StringConstructor::new))]
impl StringObj {
    #[get("length")]
    fn get_length(&self) -> usize {
        self.inner.borrow().string.len()
    }

    pub fn anchor(&self, name: &str) -> ValueResult {
        Ok(format!(
            "<a name=\"{}\">{}</a>",
            name.replace('"', "&quot;"),
            self.inner.borrow().string
        )
        .into())
    }

    pub fn at(&self, index: isize) -> Value {
        self.get_single(index).map_or(Value::Undefined, Into::into)
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
        self.get_single(index).map_or(Value::Undefined, Into::into)
    }

    #[prop("charCodeAt")]
    pub fn char_code_at(&self, index: isize) -> Value {
        self.get_single(index)
            .map(|s| s.chars().next().map(|c| c as u32).unwrap_or_default())
            .unwrap_or_default()
            .into()
    }

    #[prop("codePointAt")]
    pub fn code_point_at(&self, index: isize) -> Value {
        self.get_single(index)
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

//
//
// #[must_use]
// pub fn htmlstr(s: &str) -> String {
//     let mut result = String::with_capacity(s.len());
//
//     for c in s.chars() {
//         if let Some(replacement) = replace_html(c) {
//             result.push_str(replacement);
//         } else {
//             result.push(c);
//         }
//     }
//
//     result
// }
//
// #[must_use]
// pub const fn replace_html(c: char) -> Option<&'static str> {
//     Some(match c {
//         '\t' => "&tab;",
//         '\n' => "&newline;",
//         ' ' => "&nbsp;",
//         '"' => "&quot;",
//         '&' => "&amp;",
//         '<' => "&lt;",
//         '=' => "",
//         '>' => "&gt;",
//         '\u{00A0}' => "&nbsp;",
//         '¡' => "&iexcl;",
//         '¢' => "&cendt;",
//         '£' => "&pound;",
//         '¤' => "&curren;",
//         '¥' => "&yen;",
//         '¦' => "&brvbar;",
//         '§' => "&sect;",
//         '¨' => "&uml;",
//         '©' => "&copy;",
//         'ª' => "&ordf;",
//         '«' => "&laquo;",
//         '¬' => "&not;",
//         '\u{00AD}' => "&shy;",
//         '®' => "&reg;",
//         '¯' => "&macr;",
//         '°' => "&deg;",
//         '±' => "&plusmn;",
//         '²' => "&sup2;",
//         '³' => "&sup3;",
//         '´' => "&acute;",
//         'µ' => "&micro;",
//         '¶' => "&para;",
//         '·' => "&dot;",
//         '¸' => "&cedil;",
//         '¹' => "&sup1;",
//         'º' => "&ordm;",
//         '»' => "&raquo;",
//         '¼' => "&frac14;",
//         '½' => "&frac12;",
//         '¾' => "&frac34;",
//         '¿' => "&iquest;",
//         'À' => "&agrave;",
//         'Á' => "&aacute;",
//         'Â' => "&acirc;",
//         'Ã' => "&atilde;",
//         'Ä' => "&auml;",
//         'Å' => "&aring;",
//         'Æ' => "&aelig;",
//         'Ç' => "&ccedil;",
//         'È' => "&egrave;",
//         'É' => "&eacute;",
//         'Ê' => "&ecirc;",
//         'Ë' => "&euml;",
//         'Ì' => "&igrave;",
//         'Í' => "&iacute;",
//         'Î' => "&icirc;",
//         'Ï' => "&iuml;",
//         'Ð' => "&eth;",
//         'Ñ' => "&ntilde;",
//         'Ò' => "&ograve;",
//         'Ó' => "&oacute;",
//         'Ô' => "&ocirc;",
//         'Õ' => "&otilde;",
//         'Ö' => "&ouml;",
//         '×' => "&times;",
//         'Ø' => "&oslash;",
//         'Ù' => "&ugrave;",
//         'Ú' => "&uacute;",
//         'Û' => "&ucirc;",
//         'Ü' => "&uuml;",
//         'Ý' => "&yacute;",
//         'Þ' => "&thorn;",
//         'ß' => "&szlig;",
//         'à' => "&agrave;",
//         'á' => "&aacute;",
//         'â' => "&acirc;",
//         'ã' => "&atilde;",
//         'ä' => "&auml;",
//         'å' => "&aring;",
//         'æ' => "&aelig;",
//         'ç' => "&ccedil;",
//         'è' => "&egrave;",
//         'é' => "&eacute;",
//         'ê' => "&ecirc;",
//         'ë' => "&euml;",
//         'ì' => "&igrave;",
//         'í' => "&iacute;",
//         'î' => "&icirc;",
//         'ï' => "&iuml;",
//         'ð' => "&eth;",
//         'ñ' => "&ntilde;",
//         'ò' => "&ograve;",
//         'ó' => "&oacute;",
//         'ô' => "&ocirc;",
//         'õ' => "&otilde;",
//         'ö' => "&ouml;",
//         '÷' => "&divide;",
//         'ø' => "&oslash;",
//         'ù' => "&ugrave;",
//         'ú' => "&uacute;",
//         'û' => "&ucirc;",
//         'ü' => "&uuml;",
//         'ý' => "&yacute;",
//         'þ' => "&thorn;",
//         'ÿ' => "&yuml;",
//         'Ā' => "&amacr;",
//         'ā' => "&amacr;",
//         'Ă' => "&abreve;",
//         'ă' => "&abreve;",
//         'Ą' => "&aogon;",
//         'ą' => "&aogon;",
//         'Ć' => "&cacute;",
//         'ć' => "&cacute;",
//         'Ĉ' => "&ccirc;",
//         'ĉ' => "&ccirc;",
//         'Ċ' => "&cdot;",
//         'ċ' => "&cdot;",
//         'Č' => "&ccaron;",
//         'č' => "&ccaron;",
//         'Ď' => "&dcaron;",
//         'ď' => "&dcaron;",
//         'Đ' => "&dstrok;",
//         'đ' => "&dstrok;",
//         'Ē' => "&emacr;",
//         'ē' => "&emacr;",
//         'Ĕ' => "&ebreve;",
//         'ĕ' => "&ebreve;",
//         'Ė' => "&edot;",
//         'ė' => "&edot;",
//         'Ę' => "&eogon;",
//         'ę' => "&eogon;",
//         'Ě' => "&ecaron;",
//         'ě' => "&ecaron;",
//         'Ĝ' => "&gcirc;",
//         'ĝ' => "&gcirc;",
//         'Ğ' => "&gbreve;",
//         'ğ' => "&gbreve;",
//         'Ġ' => "&gdot;",
//         'ġ' => "&gdot;",
//         'Ģ' => "&gcedil;",
//         'ģ' => "&gcedil;",
//         'Ĥ' => "&hcirc;",
//         'ĥ' => "&hcirc;",
//         'Ħ' => "&hstrok;",
//         'ħ' => "&hstrok;",
//         'Ĩ' => "&itilde;",
//         'ĩ' => "&itilde;",
//         'Ī' => "&imacr;",
//         'ī' => "&imacr;",
//         'Ĭ' => "&ibreve;",
//         'ĭ' => "&ibreve;",
//         'Į' => "&iogon;",
//         'į' => "&iogon;",
//         'İ' => "&idot;",
//         'ı' => "&imath; &inodot;",
//         'Ĳ' => "&ijlig;",
//         'ĳ' => "&ijlig;",
//         'Ĵ' => "&jcirc;",
//         'ĵ' => "&jcirc;",
//         'Ķ' => "&kcedil;",
//         'ķ' => "&kcedil;",
//         'ĸ' => "&kgreen;",
//         'Ĺ' => "&lacute;",
//         'ĺ' => "&lacute;",
//         'Ļ' => "&lcedil;",
//         'ļ' => "&lcedil;",
//         'Ľ' => "&lcaron;",
//         'ľ' => "&lcaron;",
//         'Ŀ' => "&lmidot;",
//         'ŀ' => "&lmidot;",
//         'Ł' => "&lstrok;",
//         'ł' => "&lstrok;",
//         'Ń' => "&nacute;",
//         'ń' => "&nacute;",
//         'Ņ' => "&ncedil;",
//         'ņ' => "&ncedil;",
//         'Ň' => "&ncaron;",
//         'ň' => "&ncaron;",
//         'ŉ' => "&napos;",
//         'Ŋ' => "&eng;",
//         'ŋ' => "&eng;",
//         'Ō' => "&omacr;",
//         'ō' => "&omacr;",
//         'Ŏ' => "&obreve;",
//         'ŏ' => "&obreve;",
//         'Ő' => "&odblac;",
//         'ő' => "&odblac;",
//         'Œ' => "&oelig;",
//         'œ' => "&oelig;",
//         'Ŕ' => "&racute;",
//         'ŕ' => "&racute;",
//         'Ŗ' => "&rcedil;",
//         'ŗ' => "&rcedil;",
//         'Ř' => "&rcaron;",
//         'ř' => "&rcaron;",
//         'Ś' => "&sacute;",
//         'ś' => "&sacute;",
//         'Ŝ' => "&scirc;",
//         'ŝ' => "&scirc;",
//         'Ş' => "&scedil;",
//         'ş' => "&scedil;",
//         'Š' => "&scaron;",
//         'š' => "&scaron;",
//         'Ţ' => "&tcedil;",
//         'ţ' => "&tcedil;",
//         'Ť' => "&tcaron;",
//         'ť' => "&tcaron;",
//         'Ŧ' => "&tstrok;",
//         'ŧ' => "&tstrok;",
//         'Ũ' => "&utilde;",
//         'ũ' => "&utilde;",
//         'Ū' => "&umacr;",
//         'ū' => "&umacr;",
//         'Ŭ' => "&ubreve;",
//         'ŭ' => "&ubreve;",
//         'Ů' => "&uring;",
//         'ů' => "&uring;",
//         'Ű' => "&udblac;",
//         'ű' => "&udblac;",
//         'Ų' => "&uogon;",
//         'ų' => "&uogon;",
//         'Ŵ' => "&wcirc;",
//         'ŵ' => "&wcirc;",
//         'Ŷ' => "&ycirc;",
//         'ŷ' => "&ycirc;",
//         'Ÿ' => "&yuml;",
//         'ƒ' => "&fnof;",
//         'ˆ' => "&circ;",
//         '˜' => "&tilde;",
//         'Α' => "&alpha;",
//         'Β' => "&beta;",
//         'Γ' => "&gamma;",
//         'Δ' => "&delta;",
//         'Ε' => "&epsilon;",
//         'Ζ' => "&zeta;",
//         'Η' => "&eta;",
//         'Θ' => "&theta;",
//         'Ι' => "&iota;",
//         'Κ' => "&kappa;",
//         'Λ' => "&lambda;",
//         'Μ' => "&mu;",
//         'Ν' => "&nu;",
//         'Ξ' => "&xi;",
//         'Ο' => "&omicron;",
//         'Π' => "&pi;",
//         'Ρ' => "&rho;",
//         'Σ' => "&sigma;",
//         'Τ' => "&tau;",
//         'Υ' => "&upsilon;",
//         'Φ' => "&phi;",
//         'Χ' => "&chi;",
//         'Ψ' => "&psi;",
//         'Ω' => "&omega;",
//         'α' => "&alpha;",
//         'β' => "&beta;",
//         'γ' => "&gamma;",
//         'δ' => "&delta;",
//         'ε' => "&epsilon;",
//         'ζ' => "&zeta;",
//         'η' => "&eta;",
//         'θ' => "&theta;",
//         'ι' => "&iota;",
//         'κ' => "&kappa;",
//         'λ' => "&lambda;",
//         'μ' => "&mu;",
//         'ν' => "&nu;",
//         'ξ' => "&xi;",
//         'ο' => "&omicron;",
//         'π' => "&pi;",
//         'ρ' => "&rho;",
//         'ς' => "&sigmaf;",
//         'σ' => "&sigma;",
//         'τ' => "&tau;",
//         'υ' => "&upsilon;",
//         'φ' => "&phi;",
//         'χ' => "&chi;",
//         'ψ' => "&psi;",
//         'ω' => "&omega;",
//         'ϑ' => "&thetasym;",
//         'ϒ' => "&upsih;",
//         'ϖ' => "&piv;",
//         '\u{2002}' => "&ensp;",
//         '\u{2003}' => "&emsp;",
//         '\u{2009}' => "&thinsp;",
//         '\u{200C}' => "&zwnj;",
//         '\u{200D}' => "&zwj;",
//         '\u{200E}' => "&lrm;",
//         '\u{200F}' => "&rlm;",
//         '–' => "&ndash;",
//         '—' => "&mdash;",
//         '‘' => "&lsquo;",
//         '’' => "&rsquo;",
//         '‚' => "&sbquo;",
//         '“' => "&ldquo;",
//         '”' => "&rdquo;",
//         '„' => "&bdquo;",
//         '†' => "&dagger;",
//         '‡' => "&dagger;",
//         '•' => "&bull;",
//         '…' => "&hellip;",
//         '‰' => "&permil;",
//         '′' => "&prime;",
//         '″' => "&prime;",
//         '‹' => "&lsaquo;",
//         '›' => "&rsaquo;",
//         '‾' => "&oline;",
//         '€' => "&euro;",
//         '™' => "&trade;",
//         '←' => "&larr;",
//         '↑' => "&uarr;",
//         '→' => "&rarr;",
//         '↓' => "&darr;",
//         '↔' => "&harr;",
//         '↵' => "&crarr;",
//         '∀' => "&forall;",
//         '∂' => "&part;",
//         '∃' => "&exist;",
//         '∅' => "&empty;",
//         '∇' => "&nabla;",
//         '∈' => "&isin;",
//         '∉' => "&notin;",
//         '∋' => "&ni;",
//         '∏' => "&prod;",
//         '∑' => "&sum;",
//         '−' => "&minus;",
//         '∗' => "&lowast;",
//         '√' => "&radic;",
//         '∝' => "&prop;",
//         '∞' => "&infin;",
//         '∠' => "&ang;",
//         '∧' => "&and;",
//         '∨' => "&or;",
//         '∩' => "&cap;",
//         '∪' => "&cup;",
//         '∫' => "&int;",
//         '∴' => "&there4;",
//         '∼' => "&sim;",
//         '≅' => "&cong;",
//         '≈' => "&asymp;",
//         '≠' => "&ne;",
//         '≡' => "&equiv;",
//         '≤' => "&le;",
//         '≥' => "&ge;",
//         '⊂' => "&sub;",
//         '⊃' => "&sup;",
//         '⊄' => "&nsub;",
//         '⊆' => "&sube;",
//         '⊇' => "&supe;",
//         '⊕' => "&oplus;",
//         '⊗' => "&otimes;",
//         '⊥' => "&perp;",
//         '⋅' => "&sdot;",
//         '⌈' => "&lceil;",
//         '⌉' => "&rceil;",
//         '⌊' => "&lfloor;",
//         '⌋' => "&rfloor;",
//         '◊' => "&loz;",
//         '♠' => "&spades;",
//         '♣' => "&clubs;",
//         '♥' => "&hearts;",
//         '♦' => "&diams;",
//
//         _ => return None
//
//     })
//
//
// }
