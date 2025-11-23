use crate::array::Array;
use crate::conversion::{ActualString, Stringable};
use crate::utils::{ArrayLike, ProtoDefault};
use crate::value::property_key::InternalPropertyKey;
use crate::value::{Constructor, CustomName, Func, IntoValue, MutObj, Obj, Property};
use crate::{
    Error, MutObject, Object, ObjectHandle, PrimitiveValue, Realm, Res, Value, ValueResult,
};
use std::cell::{RefCell, RefMut};
use std::cmp;
use std::ops::{Deref, DerefMut};
use unicode_normalization::UnicodeNormalization;
use yavashark_macro::{object, properties_new};
use yavashark_string::{ToYSString, YSString};
use crate::builtins::RegExp;

#[derive(Debug)]
pub struct StringObj {
    pub inner: RefCell<MutableStringObj>,
}

impl ProtoDefault for StringObj {
    fn proto_default(realm: &mut Realm) -> Res<Self> {
        Self::with_string(realm, YSString::new())
    }

    fn null_proto_default() -> Self {
        Self {
            inner: RefCell::new(MutableStringObj {
                object: MutObject::null(),
                string: YSString::new(),
            }),
        }
    }
}

#[derive(Debug)]
pub struct MutableStringObj {
    pub object: MutObject,
    string: YSString,
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

impl crate::value::ObjectImpl for StringObj {
    type Inner = MutableStringObj;

    fn get_wrapped_object(&self) -> impl DerefMut<Target = impl MutObj> {
        RefMut::map(self.inner.borrow_mut(), |inner| &mut inner.object)
    }

    fn get_inner(&self) -> impl Deref<Target = Self::Inner> {
        self.inner.borrow()
    }

    fn get_inner_mut(&self) -> impl DerefMut<Target = Self::Inner> {
        self.inner.borrow_mut()
    }

    fn resolve_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        if let InternalPropertyKey::Index(n) = name {
            let index = n as isize;

            let inner = self.inner.borrow();

            let chr =
                Self::get_single_str(&inner.string, index).map_or(Value::Undefined, Into::into);

            return Ok(Some(chr.into()));
        }

        self.get_wrapped_object().resolve_property(name, realm)
    }

    fn get_own_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        if let InternalPropertyKey::Index(n) = name {
            let index = n as isize;

            let inner = self.inner.borrow();

            let chr =
                Self::get_single_str(&inner.string, index).map_or(Value::Undefined, Into::into);

            return Ok(Some(chr.into()));
        }

        self.get_wrapped_object().get_own_property(name, realm)
    }

    fn get_array_or_done(
        &self,
        index: usize,
        _: &mut Realm,
    ) -> Result<(bool, Option<Value>), Error> {
        let inner = self.inner.borrow();

        if index >= inner.string.len() {
            return Ok((false, None));
        }

        let c = inner.string.chars().nth(index).unwrap_or_default();

        let value = c.to_string().into();

        Ok((true, Some(value)))
    }

    fn primitive(&self, _: &mut Realm) -> Res<Option<PrimitiveValue>> {
        Ok(Some(self.inner.borrow().string.clone().into()))
    }

    fn name(&self) -> String {
        "String".to_string()
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
    pub fn new(_: &Object, func: ObjectHandle, realm: &mut Realm) -> crate::Res<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableStringConstructor {
                object: MutObject::with_proto(func.clone()),
            }),
        };

        this.define_property("name".into(), "String".into(), realm)?;

        this.initialize(realm)?;

        Ok(this.into_object())
    }

    pub fn override_to_string(&self, _: &mut Realm) -> Res<YSString> {
        Ok("function String() { [native code] }".into())
    }

    pub fn override_to_string_internal(&self) -> Res<YSString> {
        Ok("function String() { [native code] }".into())
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
        if literal_count == 0 {
            //length from ArrayLike.len can't be under 0
            return Ok(Value::String(YSString::new()));
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

impl Constructor for StringConstructor {
    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> Res<ObjectHandle> {
        let str = match args.first() {
            Some(v) => v.to_string(realm)?,
            None => YSString::new(),
        };

        let obj = StringObj::with_string(realm, str)?;

        Ok(Obj::into_object(obj))
    }
}

impl Func for StringConstructor {
    fn call(&self, realm: &mut Realm, args: Vec<Value>, _this: Value) -> ValueResult {
        let str = match args.first() {
            Some(v) => v.to_string(realm)?,
            None => YSString::new(),
        };

        Ok(str.into())
    }
}

impl StringObj {
    #[allow(clippy::new_ret_no_self, dead_code)]
    pub fn new(realm: &mut Realm) -> Res<ObjectHandle> {
        Ok(Obj::into_object(Self::with_string(realm, YSString::new())?))
    }

    pub fn with_string(realm: &mut Realm, string: YSString) -> Res<Self> {
        Ok(Self {
            inner: RefCell::new(MutableStringObj {
                object: MutObject::with_proto(
                    realm.intrinsics.clone_public().string.get(realm)?.clone(),
                ),
                string,
            }),
        })
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

    pub fn get_single_str(str: &str, index: isize) -> Option<String> {
        let len = str.len() as isize;

        let start = if index < 0 {
            (len + index) as usize
        } else {
            index as usize
        };

        let end = start + 1;

        let string = str.get(start..end);

        string.map(ToString::to_string)
    }
}

#[properties_new(
    intrinsic_name(string),
    default_null(string),
    constructor(StringConstructor::new)
)]
impl StringObj {
    #[get("length")]
    fn get_length(&self) -> usize {
        self.inner.borrow().string.len()
    }

    pub fn anchor(#[this] string: &Stringable, name: &str) -> ValueResult {
        Ok(format!("<a name=\"{}\">{}</a>", name.replace('"', "&quot;"), string,).into())
    }

    pub fn at(#[this] str: &Stringable, index: isize) -> Value {
        Self::get_single_str(str, index).map_or(Value::Undefined, Into::into)
    }

    pub fn big(#[this] str: &Stringable) -> ValueResult {
        Ok(format!("<big>{str}</big>").into())
    }

    pub fn blink(#[this] str: &Stringable) -> ValueResult {
        Ok(format!("<blink>{str}</blink>").into())
    }

    pub fn bold(#[this] str: &Stringable) -> ValueResult {
        Ok(format!("<b>{str}</b>").into())
    }

    #[prop("charAt")]
    pub fn char_at(#[this] str: &Stringable, index: isize) -> Value {
        Self::get_single_str(str, index).map_or(Value::Undefined, Into::into)
    }

    #[prop("charCodeAt")]
    #[must_use]
    pub fn char_code_at(#[this] str: &Stringable, index: isize) -> Value {
        Self::get_single_str(str, index)
            .map(|s| s.chars().next().map(|c| c as u32).unwrap_or_default())
            .unwrap_or_default()
            .into()
    }

    #[prop("codePointAt")]
    #[must_use]
    pub fn code_point_at(#[this] str: &Stringable, index: isize) -> Value {
        Self::get_single_str(str, index)
            .map(|s| s.chars().next().map(|c| c as u32).unwrap_or_default())
            .unwrap_or_default()
            .into()
    }

    #[prop("concat")]
    pub fn concat(
        #[this] mut string: String,
        args: &[Value],
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        for arg in args {
            string.push_str(&arg.to_string(realm)?);
        }

        Ok(string.into())
    }

    #[prop("endsWith")]
    #[must_use]
    pub fn ends_with(#[this] str: &Stringable, search: &str) -> Value {
        str.ends_with(&search).into()
    }

    #[prop("fixed")]
    pub fn fixed(#[this] str: &Stringable) -> ValueResult {
        Ok(format!("<tt>{str}</tt>").into())
    }

    #[prop("fontcolor")]
    pub fn font_color(#[this] str: &Stringable, color: &str) -> ValueResult {
        Ok(format!("<font color=\"{color}\">{str}</font>",).into())
    }

    #[prop("fontsize")]
    pub fn font_size(#[this] str: &Stringable, size: &str) -> ValueResult {
        Ok(format!("<font size=\"{size}\">{str}</font>",).into())
    }

    #[prop("includes")]
    #[must_use]
    pub fn includes(#[this] str: &Stringable, search: &str) -> bool {
        str.contains(search)
    }

    #[prop("indexOf")]
    #[must_use]
    pub fn index_of(#[this] str: &Stringable, search: &str, from: Option<isize>) -> isize {
        let from = from.unwrap_or(0);

        let from = if from < 0 {
            (str.len() as isize + from) as usize
        } else {
            from as usize
        };

        str.get(from..)
            .and_then(|s| s.find(search))
            .map_or(-1, |i| i as isize + from as isize)
    }

    #[prop("isWellFormed")]
    #[must_use]
    pub fn is_well_formed(#[this] str: &Stringable) -> bool {
        // check if we have any lone surrogates => between 0xD800-0xDFFF or 0xDC00-0xDFFF
        str.chars().all(|c| !is_lone_surrogate(c))
    }

    #[prop("italics")]
    pub fn italics(#[this] str: &Stringable) -> ValueResult {
        Ok(format!("<i>{str}</i>").into())
    }

    #[prop("lastIndexOf")]
    #[must_use]
    pub fn last_index_of(#[this] str: &Stringable, search: &str, from: Option<isize>) -> isize {
        let from = from.unwrap_or(-1);

        let from = if from < 0 {
            (str.len() as isize + from) as usize
        } else {
            from as usize
        };

        str[..from].rfind(&search).map_or(-1, |i| i as isize)
    }

    #[prop("link")]
    pub fn link(#[this] str: &Stringable, url: &str) -> ValueResult {
        Ok(format!("<a href=\"{url}\">{str}</a>").into())
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
    #[prop("match")]
    pub fn match_(#[this] str: &Stringable, pattern: &RegExp, #[realm] realm: &mut Realm) -> ValueResult {
        pattern.exec(&Object::null().into(), str.to_ys_string(), realm)
    }

    // #[prop("matchAll")]
    // pub fn match_all(&self, pattern: &RegExp, #[realm] realm: &mut Realm) -> ValueResult {
    //     //TODO
    // }

    pub fn normalize(#[this] str: &Stringable, form: &str) -> ValueResult {
        let form = match form {
            "NFC" => str.nfc().to_string(),
            "NFD" => str.nfd().to_string(),
            "NFKC" => str.nfkc().to_string(),
            "NFKD" => str.nfkd().to_string(),
            _ => return Err(Error::range("Invalid normalization form")),
        };

        Ok(form.into())
    }

    #[prop("padEnd")]
    pub fn pad_end(
        #[this] str: &Stringable,
        target_length: usize,
        pad_string: &Option<String>,
    ) -> ValueResult {
        let pad_string = pad_string.as_deref().unwrap_or(" ");

        let pad_len = target_length.saturating_sub(str.len());

        let pad = pad_string.repeat(pad_len);

        Ok(format!("{str}{pad}").into())
    }

    #[prop("padStart")]
    pub fn pad_start(
        #[this] str: &Stringable,
        target_length: usize,
        pad_string: &Option<String>,
    ) -> ValueResult {
        let pad_string = pad_string.as_deref().unwrap_or(" ");

        let pad_len = target_length.saturating_sub(str.len());

        let pad = pad_string.repeat(pad_len);

        Ok(format!("{pad}{str}").into())
    }

    pub fn repeat(#[this] str: &Stringable, count: usize) -> ValueResult {
        Ok(str.repeat(count).into())
    }

    pub fn replace(#[this] str: &Stringable, search: &str, replace: &str) -> ValueResult {
        Ok(str.replacen(search, replace, 1).into())
    }

    #[prop("replaceAll")]
    pub fn replace_all(#[this] str: &Stringable, search: &str, replace: &str) -> ValueResult {
        Ok(str.replace(search, replace).into())
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

    pub fn slice(#[this] str: &Stringable, start: isize, end: Option<isize>) -> ValueResult {
        // negative numbers are counted from the end of the string
        let start = if start < 0 {
            (str.len() as isize + start) as usize
        } else {
            start as usize
        };

        let end = end.map_or(str.len(), |end| {
            if end < 0 {
                (str.len() as isize + end) as usize
            } else {
                end as usize
            }
        });

        let end = cmp::min(end, str.len());

        let string = str.get(start..end);

        Ok(YSString::from_ref(string.unwrap_or_default()).into())
    }

    pub fn small(#[this] str: &Stringable) -> ValueResult {
        Ok(format!("<small>{str}</small>").into())
    }

    pub fn split(
        #[this] str: &Stringable,
        separator: &str,
        limit: Option<usize>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        let limit = limit.unwrap_or(usize::MAX);

        let parts = str.splitn(limit, separator);

        let mut array = Vec::new();

        for part in parts {
            array.push(YSString::from_ref(part).into());
        }

        Ok(Array::with_elements(realm, array)?.into_value())
    }

    #[prop("startsWith")]
    #[must_use]
    pub fn starts_with(#[this] str: &Stringable, search: &str) -> bool {
        str.starts_with(search)
    }

    pub fn strike(#[this] str: &Stringable) -> ValueResult {
        Ok(format!("<strike>{str}</strike>").into())
    }

    pub fn sub(#[this] str: &Stringable) -> ValueResult {
        Ok(format!("<sub>{str}</sub>").into())
    }

    pub fn substr(#[this] str: &Stringable, start: isize, len: Option<isize>) -> ValueResult {
        // negative numbers are counted from the end of the string
        let start = if start < 0 {
            (str.len() as isize + start) as usize
        } else {
            start as usize
        };

        let end = len.map_or(str.len(), |len| start + len as usize);

        let end = cmp::min(end, str.len());

        let string = str.get(start..end);

        Ok(YSString::from_ref(string.unwrap_or_default()).into())
    }

    pub fn substring(#[this] str: &Stringable, start: isize, end: Option<isize>) -> ValueResult {
        // negative numbers are counted from the end of the string
        let start = if start < 0 {
            (str.len() as isize + start) as usize
        } else {
            start as usize
        };

        let end = end.map_or(str.len(), |end| {
            if end < 0 {
                (str.len() as isize + end) as usize
            } else {
                end as usize
            }
        });

        let end = cmp::min(end, str.len());

        let string = str.get(start..end);

        Ok(YSString::from_ref(string.unwrap_or_default()).into())
    }

    pub fn sup(#[this] str: &Stringable) -> ValueResult {
        Ok(format!("<sup>{str}</sup>").into())
    }


    #[prop("toLocaleLowerCase")]
    pub fn _to_locale_lower_case(#[this] str: &Stringable) -> ValueResult {
        Ok(str.to_lowercase().into())
    }

    #[prop("toLocaleUpperCase")]
    pub fn _to_locale_upper_case(#[this] str: &Stringable) -> ValueResult {
        Ok(str.to_uppercase().into())
    }

    #[prop("toLowerCase")]
    pub fn _to_lower_case(#[this] str: &Stringable) -> ValueResult {
        Ok(str.to_lowercase().into())
    }

    #[prop("toString")]
    #[must_use]
    pub fn _to_string(#[this] str: ActualString) -> Value {
        str.into()
    }

    #[prop("toUpperCase")]
    pub fn _to_upper_case(#[this] str: &Stringable) -> ValueResult {
        Ok(str.to_uppercase().into())
    }

    #[prop("toWellFormed")]
    pub fn _to_well_formed(#[this] str: &Stringable) -> ValueResult {
        let well_formed = str
            .chars()
            .map(|c| if is_lone_surrogate(c) { '\u{FFFD}' } else { c })
            .collect::<String>();

        Ok(well_formed.into())
    }

    pub fn trim(#[this] str: &Stringable) -> ValueResult {
        Ok(YSString::from_ref(str.trim()).into())
    }

    #[prop("trimEnd")]
    pub fn trim_end(#[this] str: &Stringable) -> ValueResult {
        Ok(YSString::from_ref(str.trim_end()).into())
    }

    #[prop("trimStart")]
    pub fn trim_start(#[this] str: &Stringable) -> ValueResult {
        Ok(YSString::from_ref(str.trim_start()).into())
    }

    #[prop("valueOf")]
    #[must_use]
    pub fn value_of(#[this] str: ActualString) -> Value {
        str.into()
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
