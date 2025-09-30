use crate::array::Array;
use crate::console::print::PrettyObjectOverride;
use crate::{ControlFlow, Error, MutObject, Object, ObjectHandle, Realm, Res, Value, ValueResult};
use regress::{Range, Regex};
use std::cell::{Cell, RefCell};
use std::collections::BTreeSet;
use yavashark_macro::{object, properties_new};
use yavashark_string::YSString;
use crate::value::{Constructor, Func, Obj, Symbol};

#[object()]
#[derive(Debug)]
pub struct RegExp {
    regex: Regex,
    flags: Flags,

    last_index: Cell<usize>,
    original_source: YSString,
    original_flags: YSString,
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
    pub has_indices: bool,
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

impl Flags {
    const ORDER: [char; 9] = ['d', 'g', 'i', 'm', 's', 'u', 'v', 'y', 'n'];

    const fn set_flag(&mut self, flag: char) {
        match flag {
            'i' => self.icase = true,
            'm' => self.multiline = true,
            's' => self.dot_all = true,
            'n' => self.no_opt = true,
            'u' => self.unicode = true,
            'v' => self.unicode_sets = true,
            'g' => self.global = true,
            'y' => self.sticky = true,
            'd' => self.has_indices = true,
            _ => {}
        }
    }

    fn try_from_str(flags: &str) -> Res<(Self, YSString)> {
        let mut parsed = Self::default();
        let mut seen = BTreeSet::new();

        for ch in flags.chars() {
            if !matches!(ch, 'd' | 'g' | 'i' | 'm' | 's' | 'u' | 'v' | 'y' | 'n') {
                return Err(Error::syn_error(format!(
                    "Invalid regular expression flag '{ch}'"
                )));
            }

            if !seen.insert(ch) {
                return Err(Error::syn_error(format!(
                    "Duplicate regular expression flag '{ch}'"
                )));
            }

            parsed.set_flag(ch);
        }

        if parsed.unicode && parsed.unicode_sets {
            return Err(Error::syn_error(
                "Flags 'u' and 'v' cannot be combined in the same regular expression"
                    .to_string(),
            ));
        }

        let mut canonical = String::with_capacity(seen.len());
        for flag in Self::ORDER.iter().copied() {
            if seen.contains(&flag) {
                canonical.push(flag);
            }
        }

        Ok((parsed, YSString::from(canonical)))
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
                object: MutObject::with_proto(realm.intrinsics.regexp.clone()),
            }),
            flags,
            original_source: source,
            original_flags: flags_str,
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
        let (flags, canonical_flags) = Flags::try_from_str(flags_str)?;

        let regex = Regex::from_unicode(source.chars().map(u32::from), flags)
            .map_err(|e| Error::syn_error(e.text))?;

        Ok(Self::new(
            realm,
            regex,
            flags,
            YSString::from_ref(source),
            canonical_flags,
        ))
    }

    fn set_last_index_value(
        &self,
        this: &Value,
        value: usize,
        _realm: &mut Realm,
    ) -> Res<()> {
        let obj = this.as_object()?;

        obj.define_property("lastIndex".into(), value.into())?;

        self.last_index.set(value);

        Ok(())
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
    pub fn new(_: &Object, func: ObjectHandle) -> crate::Res<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableRegExpConstructor {
                object: MutObject::with_proto(func.clone()),
            }),
        };

        this.initialize(func)?;

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

impl Constructor for RegExpConstructor {
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

impl Func for RegExpConstructor {
    fn call(&self, realm: &mut Realm, args: Vec<Value>, _this: Value) -> ValueResult {
        Constructor::construct(self, realm, args)
    }
}

#[properties_new(constructor(RegExpConstructor::new))]
impl RegExp {
    #[prop("exec")]
    pub fn exec(
        &self,
        #[this] this: &Value,
        inp: YSString,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        let input = inp.as_str();
        let input_len = input.len();

        let last_index = if self.flags.global || self.flags.sticky {
            self.last_index.get()
        } else {
            0
        };

        if last_index > input_len {
            if self.flags.global || self.flags.sticky {
                self.set_last_index_value(this, 0, realm)?;
            }
            return Ok(Value::Undefined);
        }

        let Some(matched) = self.regex.find_from(input, last_index).next() else {
            if self.flags.global || self.flags.sticky {
                self.set_last_index_value(this, 0, realm)?;
            }
            return Ok(Value::Undefined);
        };

        if self.flags.sticky && matched.start() != last_index {
            self.set_last_index_value(this, 0, realm)?;
            return Ok(Value::Undefined);
        }

        let match_start = matched.start();
        let match_end = matched.end();

        if self.flags.global || self.flags.sticky {
            self.set_last_index_value(this, match_end, realm)?;
        } else {
            self.set_last_index_value(this, 0, realm)?;
        }

        let result = Array::from_realm(realm);
        let matched_slice = input.get(match_start..match_end).unwrap_or("");
        result.insert_array(YSString::from_ref(matched_slice).into(), 0)?;
        result.define_property("index".into(), match_start.into())?;
        result.define_property("input".into(), inp.clone().into())?;

        let indices_array = if self.flags.has_indices {
            let arr = Array::from_realm(realm);
            let range = Array::with_elements(
                realm,
                vec![match_start.into(), match_end.into()],
            )?;
            arr.insert_array(range.into_value(), 0)?;
            Some(arr)
        } else {
            None
        };

        let mut named_groups = matched
            .named_groups()
            .map(|(name, range)| (name.to_string(), range))
            .collect::<Vec<(String, Option<Range>)>>();
        named_groups.sort_by(|(a, _), (b, _)| a.cmp(b));

        let (groups_value, indices_groups_value) = if named_groups.is_empty() {
            (Value::Undefined, Value::Undefined)
        } else {
            let groups_obj = Object::null();
            let indices_groups_obj = indices_array
                .as_ref()
                .map(|_| Object::null());

            for (name, range_opt) in &named_groups {
                let name = YSString::from_ref(name);

                let capture_value = range_opt
                    .clone()
                    .and_then(|range| input.get(range))
                    .map_or(Value::Undefined, |slice| YSString::from_ref(slice).into());

                groups_obj.define_property(name.clone().into(), capture_value)?;

                if let Some(indices_obj) = indices_groups_obj.as_ref() {
                    let indices_value = if let Some(range) = range_opt {
                        Array::with_elements(
                            realm,
                            vec![
                                range.start.into(),
                                range.end.into(),
                            ],
                        )?
                        .into_value()
                    } else {
                        Value::Undefined
                    };
                    indices_obj.define_property(name.into(), indices_value)?;
                }
            }

            let indices_groups_value = indices_groups_obj
                .map_or(Value::Undefined, Into::into);

            (groups_obj.into(), indices_groups_value)
        };

        result.define_property("groups".into(), groups_value)?;

        for index in 1..=matched.captures.len() {
            let capture = matched.group(index);

            let capture_value = capture
                .clone()
                .and_then(|range| input.get(range))
                .map_or(Value::Undefined, |slice| YSString::from_ref(slice).into());

            result.insert_array(capture_value, index)?;

            if let (Some(indices_arr), Some(range)) = (indices_array.as_ref(), capture.clone()) {
                let indices_value = Array::with_elements(
                    realm,
                    vec![
                        range.start.into(),
                        range.end.into(),
                    ],
                )?
                .into_value();
                indices_arr.insert_array(indices_value, index)?;
            } else if let Some(indices_arr) = indices_array.as_ref() {
                indices_arr.insert_array(Value::Undefined, index)?;
            }
        }

        if let Some(indices_arr) = indices_array {
            indices_arr.define_property("groups".into(), indices_groups_value)?;
            result.define_property("indices".into(), indices_arr.into_value())?;
        }

        Ok(result.into_value())
    }

    #[prop("test")]
    pub fn test(
        &self,
        #[this] this: &Value,
        value: YSString,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        let result = self.exec(this, value, realm)?;
        Ok(Value::Boolean(!matches!(result, Value::Undefined)))
    }

    #[prop(Symbol::MATCH)]
    pub fn symbol_match(
        &self,
        #[this] this: &Value,
        input: YSString,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        if !self.flags.global {
            let exec_result = self.exec(this, input.clone(), realm)?;
            return if exec_result.is_undefined() {
                Ok(Value::Null)
            } else {
                Ok(exec_result)
            };
        }

        self.set_last_index_value(this, 0, realm)?;
        let matches = Array::from_realm(realm);
        let mut found = false;

        loop {
            let previous_last_index = self.last_index.get();
            let exec_result = self.exec(this, input.clone(), realm)?;

            if exec_result.is_undefined() {
                self.set_last_index_value(this, 0, realm)?;
                return if found {
                    Ok(matches.into_value())
                } else {
                    Ok(Value::Null)
                };
            }

            found = true;

            let Value::Object(result_obj) = exec_result else {
                self.set_last_index_value(this, 0, realm)?;
                return Err(
                    Error::ty_error("RegExp exec must return an object".to_string()).into(),
                );
            };

            let matched_value = result_obj.get("0", realm)?;
            matches.push(matched_value)?;

            if self.last_index.get() == previous_last_index {
                let next_index =
                    advance_string_index(&input, previous_last_index, self.flags.unicode);
                self.set_last_index_value(this, next_index, realm)?;
            }
        }
    }

    #[prop(Symbol::SEARCH)]
    pub fn symbol_search(
        &self,
        #[this] this: &Value,
        input: YSString,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        let previous_last_index = self.last_index.get();

        self.set_last_index_value(this, 0, realm)?;
        let exec_result = self.exec(this, input, realm)?;
        self.set_last_index_value(this, previous_last_index, realm)?;

        if exec_result.is_undefined() {
            return Ok(Value::Number(-1.0));
        }

        let Value::Object(result_obj) = exec_result else {
            return Err(
                Error::ty("RegExp exec must return an object"),
            );
        };

        let index_value = result_obj.get("index", realm)?;
        let index = index_value.to_number(realm)?;
        Ok(Value::Number(index))
    }

    #[prop("toString")]
    pub fn js_to_string(&self) -> YSString {
        let escaped = escape_pattern(&self.original_source);
        let mut result = String::new();
        result.push('/');
        result.push_str(escaped.as_str());
        result.push('/');
        result.push_str(self.original_flags.as_str());

        YSString::from_ref(&result)
    }

    #[get("global")]
    pub const fn global(&self) -> bool {
        self.flags.global
    }

    #[get("lastIndex")]
    pub const fn last_index(&self) -> usize {
        self.last_index.get()
    }

    #[set("lastIndex")]
    pub fn set_last_index(&self, index: usize) {
        self.last_index.set(index);
    }

    #[get("hasIndices")]
    pub const fn has_indices(&self) -> bool {
        self.flags.has_indices
    }

    #[get("ignoreCase")]
    pub const fn ignore_case(&self) -> bool {
        self.flags.icase
    }

    #[get("multiline")]
    pub const fn multiline(&self) -> bool {
        self.flags.multiline
    }

    #[get("dotAll")]
    pub const fn dot_all(&self) -> bool {
        self.flags.dot_all
    }

    #[get("unicode")]
    pub const fn unicode(&self) -> bool {
        self.flags.unicode
    }

    #[get("unicodeSets")]
    pub const fn unicode_sets(&self) -> bool {
        self.flags.unicode_sets
    }

    #[get("sticky")]
    pub const fn sticky(&self) -> bool {
        self.flags.sticky
    }

    #[get("flags")]
    pub fn flag_string(&self) -> YSString {
        self.original_flags.clone()
    }

    #[get("source")]
    pub fn source(&self) -> YSString {
        escape_pattern(&self.original_source)
    }
}

impl PrettyObjectOverride for RegExp {
    fn pretty_inline(
        &self,
        _obj: &crate::value::Object,
        _not: &mut Vec<usize>,
    ) -> Option<String> {
        let mut s = String::new();
        s.push('/');
        let escaped = escape_pattern(&self.original_source);
        s.push_str(escaped.as_str());
        s.push('/');
        s.push_str(self.original_flags.as_str());
        Some(s)
    }

    fn pretty_multiline(
        &self,
        _obj: &crate::value::Object,
        _not: &mut Vec<usize>,
    ) -> Option<String> {
        self.pretty_inline(_obj, _not)
    }
}

fn escape_pattern(source: &YSString) -> YSString {
    if source.is_empty() {
        return YSString::from_ref("(?:)");
    }

    let mut escaped = String::with_capacity(source.len());

    for ch in source.as_str().chars() {
        match ch {
            '/' => escaped.push_str("\\/"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\u{2028}' => escaped.push_str("\\u2028"),
            '\u{2029}' => escaped.push_str("\\u2029"),
            _ => escaped.push(ch),
        }
    }

    YSString::from(escaped)
}

const fn advance_string_index(_input: &YSString, index: usize, _unicode: bool) -> usize {
    index.saturating_add(1)
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
