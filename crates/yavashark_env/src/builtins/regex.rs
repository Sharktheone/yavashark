use crate::array::Array;
use crate::builtins::iterator::{create_iter_result_object, Iterator};
use crate::builtins::NumberConstructor;
use crate::console::print::PrettyObjectOverride;
use crate::realm::Intrinsic;
use crate::value::{DefinePropertyResult, IntoValue, Obj, Symbol};
use crate::{ControlFlow, Error, MutObject, Object, ObjectHandle, Realm, Res, Value, ValueResult};
use regress::{Range, Regex};
use std::cell::{Cell, RefCell};
use std::collections::BTreeSet;
use yavashark_macro::{object, properties, props};
use yavashark_string::YSString;

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
                "Flags 'u' and 'v' cannot be combined in the same regular expression".to_string(),
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
    pub fn new(
        realm: &mut Realm,
        regex: Regex,
        flags: Flags,
        source: YSString,
        flags_str: YSString,
    ) -> Res<Self> {
        let obj = Self {
            regex,
            inner: RefCell::new(MutableRegExp {
                object: MutObject::with_proto(
                    realm.intrinsics.clone_public().regexp.get(realm)?.clone(),
                ),
            }),
            flags,
            original_source: source,
            original_flags: flags_str,
            last_index: Cell::new(0),
        };

        obj.define_property_attributes(
            "lastIndex".into(),
            crate::Variable::write(Value::from(0)),
            realm,
        )?;

        Ok(obj)
    }

    pub fn new_from_str(realm: &mut Realm, source: &str) -> Res<Self> {
        let regex = Regex::new(source).map_err(|e| ControlFlow::error(e.to_string()))?;

        Self::new(
            realm,
            regex,
            Flags::default(),
            YSString::from_ref(source),
            YSString::new(),
        )
    }

    pub fn new_from_str_with_flags(realm: &mut Realm, source: &str, flags_str: &str) -> Res<Self> {
        let (flags, canonical_flags) = Flags::try_from_str(flags_str)?;

        let regex = Regex::from_unicode(source.chars().map(u32::from), flags)
            .map_err(|e| Error::syn_error(e.text))?;

        Self::new(
            realm,
            regex,
            flags,
            YSString::from_ref(source),
            canonical_flags,
        )
    }

    fn set_last_index_value(&self, this: &Value, value: usize, realm: &mut Realm) -> Res<()> {
        let obj = this.as_object()?;

        match obj.define_property("lastIndex".into(), value.into(), realm)? {
            DefinePropertyResult::ReadOnly => {
                return Err(Error::ty(
                    "Cannot set property lastIndex which is non-writable",
                ));
            }
            DefinePropertyResult::Setter(setter, val) => {
                setter.call(vec![val], this.clone(), realm)?;
            }
            DefinePropertyResult::Handled => {}
        }

        self.last_index.set(value);

        Ok(())
    }
}

#[props(intrinsic_name = regexp)]
impl RegExp {
    #[constructor]
    #[call_constructor]
    fn construct(regex: Option<String>, flags: Option<String>, realm: &mut Realm) -> Res<Self> {
        let regex = regex.as_deref().unwrap_or_default();
        let flags = flags.as_deref().unwrap_or_default();

        RegExp::new_from_str_with_flags(realm, &regex, &flags)
    }

    fn escape(value: Value) -> Res<YSString> {
        // 1. If S is not a String, throw a TypeError exception.
        let Value::String(s) = value else {
            return Err(Error::ty("RegExp.escape requires a string argument"));
        };

        Ok(escape_for_regexp(&s.as_str_lossy()))
    }

    pub fn exec(
        &self,
        #[this] this: &Value,
        inp: YSString,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        let input = inp.as_str_lossy();
        let input_len = input.len();

        // Let lastIndex be ? ToLength(? Get(R, "lastIndex")).
        let this_obj = this.as_object()?;
        let last_index_val = this_obj.get("lastIndex", realm)?;
        let last_index_raw = last_index_val.to_number(realm)?;
        // ToLength: clamp to 0..2^53-1 as integer
        let last_index_full = if last_index_raw.is_nan() || last_index_raw < 0.0 {
            0usize
        } else {
            (last_index_raw as usize).min(NumberConstructor::MAX_SAFE_INTEGER_U as usize)
        };

        let last_index = if self.flags.global || self.flags.sticky {
            last_index_full
        } else {
            0
        };

        if last_index > input_len {
            if self.flags.global || self.flags.sticky {
                self.set_last_index_value(this, 0, realm)?;
            }
            return Ok(Value::Null);
        }

        let Some(matched) = self.regex.find_from(&input, last_index).next() else {
            if self.flags.global || self.flags.sticky {
                self.set_last_index_value(this, 0, realm)?;
            }
            return Ok(Value::Null);
        };

        if self.flags.sticky && matched.start() != last_index {
            self.set_last_index_value(this, 0, realm)?;
            return Ok(Value::Null);
        }

        let match_start = matched.start();
        let match_end = matched.end();

        if self.flags.global || self.flags.sticky {
            self.set_last_index_value(this, match_end, realm)?;
        }

        let result = Array::from_realm(realm)?;
        let matched_slice = input.get(match_start..match_end).unwrap_or("");
        result.insert_array(YSString::from_ref(matched_slice).into(), 0)?;
        result.define_property("index".into(), match_start.into(), realm)?;
        result.define_property("input".into(), inp.clone().into(), realm)?;

        let indices_array = if self.flags.has_indices {
            let arr = Array::from_realm(realm)?;
            let range = Array::with_elements(realm, vec![match_start.into(), match_end.into()])?;
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
            let indices_groups_obj = indices_array.as_ref().map(|_| Object::null());

            for (name, range_opt) in &named_groups {
                let name = YSString::from_ref(name);

                let capture_value = range_opt
                    .clone()
                    .and_then(|range| input.get(range))
                    .map_or(Value::Undefined, |slice| YSString::from_ref(slice).into());

                groups_obj.define_property(name.clone().into(), capture_value, realm)?;

                if let Some(indices_obj) = indices_groups_obj.as_ref() {
                    let indices_value = if let Some(range) = range_opt {
                        Array::with_elements(realm, vec![range.start.into(), range.end.into()])?
                            .into_value()
                    } else {
                        Value::Undefined
                    };
                    indices_obj.define_property(name.into(), indices_value, realm)?;
                }
            }

            let indices_groups_value = indices_groups_obj.map_or(Value::Undefined, Into::into);

            (groups_obj.into(), indices_groups_value)
        };

        result.define_property("groups".into(), groups_value, realm)?;

        for index in 1..=matched.captures.len() {
            let capture = matched.group(index);

            let capture_value = capture
                .clone()
                .and_then(|range| input.get(range))
                .map_or(Value::Undefined, |slice| YSString::from_ref(slice).into());

            result.insert_array(capture_value, index)?;

            if let (Some(indices_arr), Some(range)) = (indices_array.as_ref(), capture.clone()) {
                let indices_value =
                    Array::with_elements(realm, vec![range.start.into(), range.end.into()])?
                        .into_value();
                indices_arr.insert_array(indices_value, index)?;
            } else if let Some(indices_arr) = indices_array.as_ref() {
                indices_arr.insert_array(Value::Undefined, index)?;
            }
        }

        if let Some(indices_arr) = indices_array {
            indices_arr.define_property("groups".into(), indices_groups_value, realm)?;
            result.define_property("indices".into(), indices_arr.into_value(), realm)?;
        }

        Ok(result.into_value())
    }

    pub fn test(
        &self,
        #[this] this: &Value,
        value: YSString,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        let result = self.exec(this, value, realm)?;
        Ok(Value::Boolean(!result.is_nullish()))
    }

    pub fn compile(&self, _a: Value, _b: Value) {}

    #[prop(Symbol::MATCH)]
    pub fn symbol_match(
        &self,
        #[this] this: &Value,
        input: YSString,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        if !self.flags.global {
            return self.regexp_exec(this, input.clone(), realm);
        }

        self.set_last_index_value(this, 0, realm)?;
        let matches = Array::from_realm(realm)?;
        let mut found = false;

        loop {
            let previous_last_index = self.last_index.get();
            let exec_result = self.regexp_exec(this, input.clone(), realm)?;

            if exec_result.is_null() {
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

            let matched_value = result_obj.get(0usize, realm)?;
            matches.push(matched_value.clone())?;

            let matched_str = matched_value.to_string(realm)?;
            if matched_str.is_empty() {
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
        let exec_result = self.regexp_exec(this, input, realm)?;
        self.set_last_index_value(this, previous_last_index, realm)?;

        if exec_result.is_null() {
            return Ok(Value::Number(-1.0));
        }

        let Value::Object(result_obj) = exec_result else {
            return Err(Error::ty("RegExp exec must return an object"));
        };

        let index_value = result_obj.get("index", realm)?;
        let index = index_value.to_number(realm)?;
        Ok(Value::Number(index))
    }

    #[prop(Symbol::MATCH_ALL)]
    pub fn symbol_match_all(
        &self,
        #[this] this: &Value,
        string: YSString,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        // 22.2.6.8 RegExp.prototype [ @@matchAll ] ( string )

        // 1. Let R be the this value.
        // 2. If Type(R) is not Object, throw a TypeError exception.
        let this_obj = this.as_object()?;

        // 3. Let S be ? ToString(string).
        // (already have string as YSString)

        // 5. Let flags be ? ToString(? Get(R, "flags")).
        let flags = this_obj.get("flags", realm)?.to_string(realm)?;
        let flags_str = flags.as_str_lossy();

        // 6. Let matcher be ? Construct(C, « R, flags »).
        // Create a copy of the regexp
        let matcher = RegExp::new_from_str_with_flags(
            realm,
            &self.original_source.as_str_lossy(),
            &flags_str,
        )?;
        let matcher_handle: ObjectHandle = Obj::into_object(matcher);

        // 7. Let lastIndex be ? ToLength(? Get(R, "lastIndex")).
        let last_index = this_obj.get("lastIndex", realm)?.to_number(realm)? as usize;

        // 8. Perform ? Set(matcher, "lastIndex", lastIndex, true).
        matcher_handle.define_property("lastIndex".into(), last_index.into(), realm)?;

        // 9-10. global and unicode flags
        let global = flags_str.contains('g');
        let unicode = flags_str.contains('u') || flags_str.contains('v');

        // 13. Return CreateRegExpStringIterator(matcher, S, global, fullUnicode).
        let iterator = RegExpStringIterator::new(matcher_handle, string, global, unicode, realm)?;
        Ok(iterator.into_value())
    }

    #[prop(Symbol::REPLACE)]
    pub fn symbol_replace(
        &self,
        #[this] this: &Value,
        string: YSString,
        replace_value: Value,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        // Convert to owned String to avoid borrow issues with as_str() during exec calls
        let s: String = string.as_str_lossy().to_string();
        let length_s = s.len();

        let functional_replace = replace_value.is_callable();
        let replace_str = if !functional_replace {
            replace_value.to_string(realm)?
        } else {
            YSString::new()
        };

        // Per spec:
        // Let global be ! ToBoolean(? Get(rx, "global")).
        // Let fullUnicode be ! ToBoolean(? Get(rx, "unicode")) or ! ToBoolean(? Get(rx, "unicodeSets")).
        let this_obj = this.as_object()?;
        let global = this_obj.get("global", realm)?.is_truthy();
        let unicode = this_obj.get("unicode", realm)?.is_truthy();
        let unicode_sets = this_obj.get("unicodeSets", realm)?.is_truthy();
        let full_unicode = unicode || unicode_sets;

        if global {
            self.set_last_index_value(this, 0, realm)?;
        }

        // Collect all match results
        let mut results: Vec<crate::value::Object> = Vec::new();

        loop {
            let result = self.regexp_exec(this, string.clone(), realm)?;

            if result.is_null() {
                break;
            }

            let Value::Object(result_obj) = result else {
                return Err(Error::ty("RegExp exec must return an object"));
            };

            results.push(result_obj.clone());

            if !global {
                break;
            }

            // If empty match, advance lastIndex
            let match_str = result_obj.get(0usize, realm)?.to_string(realm)?;
            if match_str.is_empty() {
                let this_index = self.last_index.get();
                let next_index = advance_string_index(&string, this_index, full_unicode);
                self.set_last_index_value(this, next_index, realm)?;
            }
        }

        let mut accumulated_result = String::new();
        let mut next_source_position: usize = 0;

        for result_obj in results {
            // Get result length (number of captures + 1)
            let result_length = result_obj.get("length", realm)?.to_number(realm)? as usize;
            let n_captures = result_length.saturating_sub(1);

            // Get matched string (use numeric index, not string "0")
            let matched = result_obj.get(0usize, realm)?.to_string(realm)?;
            let match_length = matched.len();

            // Get position
            let position_val = result_obj.get("index", realm)?;
            let position = position_val.to_number(realm)? as usize;
            let position = position.min(length_s);

            // Collect captures
            let mut captures: Vec<Value> = Vec::with_capacity(n_captures);
            for n in 1..=n_captures {
                let cap_n = result_obj.get(n, realm)?;
                let cap_val = if cap_n.is_undefined() {
                    Value::Undefined
                } else {
                    cap_n.to_string(realm)?.into()
                };
                captures.push(cap_val);
            }

            // Get named captures
            let named_captures = result_obj.get("groups", realm)?;

            // Per spec: If namedCaptures is not undefined, set namedCaptures to ? ToObject(namedCaptures).
            // This will throw TypeError for null
            let named_captures = if !named_captures.is_undefined() {
                named_captures.to_object()?.into()
            } else {
                named_captures
            };

            let replacement_string = if functional_replace {
                // Build replacer arguments: matched, ...captures, position, S, [namedCaptures]
                let mut replacer_args: Vec<Value> = vec![matched.clone().into()];
                replacer_args.extend(captures.iter().cloned());
                replacer_args.push((position as f64).into());
                replacer_args.push(string.clone().into());
                if !named_captures.is_undefined() {
                    replacer_args.push(named_captures);
                }

                let replacement_value =
                    replace_value.call(realm, replacer_args, Value::Undefined)?;
                replacement_value.to_string(realm)?
            } else {
                // GetSubstitution
                get_substitution(
                    &matched,
                    &s,
                    position,
                    &captures,
                    &named_captures,
                    &replace_str,
                    realm,
                )?
            };

            if position >= next_source_position {
                accumulated_result.push_str(&s[next_source_position..position]);
                accumulated_result.push_str(&replacement_string.as_str_lossy());
                next_source_position = position + match_length;
            }
        }

        if next_source_position < length_s {
            accumulated_result.push_str(&s[next_source_position..]);
        }

        Ok(YSString::from(accumulated_result).into())
    }

    #[prop(Symbol::SPLIT)]
    pub fn symbol_split(
        &self,
        #[this] _this: &Value,
        string: YSString,
        limit: Value,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        // 21.2.5.13 RegExp.prototype [ @@split ] ( string, limit )
        let s = string.as_str_lossy();
        // size is the length in UTF-16 code units (not bytes)
        let size: usize = s.chars().map(|c| c.len_utf16()).sum();

        // 5. Let flags be ? ToString(? Get(rx, "flags")).
        let flags = self.original_flags.as_str_lossy();

        // 6-7. unicodeMatching
        let full_unicode = flags.contains('u') || flags.contains('v');

        // 8-9. If flags contains "y", let newFlags be flags. Else add "y".
        let new_flags = if flags.contains('y') {
            self.original_flags.to_string()
        } else {
            format!("{}y", flags)
        };

        // 10. Let splitter be ? Construct(C, « rx, newFlags »).
        // Create a new RegExp with the sticky flag
        let splitter = RegExp::new_from_str_with_flags(
            realm,
            &self.original_source.as_str_lossy(),
            &new_flags,
        )?;
        let splitter_value: Value = splitter.into_value();
        let splitter_obj = splitter_value.as_object()?;

        // 13. If limit is undefined, let lim be 2^32 - 1; else let lim be ? ToUint32(limit).
        let lim = if limit.is_undefined() {
            u32::MAX as usize
        } else {
            let n = limit.to_number(realm)?;
            if n.is_nan() || n.is_infinite() || n == 0.0 {
                0
            } else {
                // ToUint32: truncate then modulo 2^32
                let int = n.trunc() as i64;
                (int as u32) as usize
            }
        };

        // 14. If lim is 0, return A.
        if lim == 0 {
            return Ok(Array::from_realm(realm)?.into_value());
        }

        // 16. If size is 0, then
        if size == 0 {
            // a. Let z be ? RegExpExec(splitter, S).
            let exec_prop = splitter_obj.get("exec", realm)?;
            let result =
                exec_prop.call(realm, vec![string.clone().into()], splitter_value.clone())?;

            // b. If z is not null, return empty array
            if !result.is_null() {
                return Ok(Array::from_realm(realm)?.into_value());
            }

            // c. Return array with original string
            let arr = Array::from_realm(realm)?;
            arr.push(string.into())?;
            return Ok(arr.into_value());
        }

        // 17. Let p be 0.
        let mut p: usize = 0;
        // 18. Let q be p.
        let mut q: usize = 0;

        let mut result_vec: Vec<Value> = Vec::new();

        // 19. Repeat, while q < size
        while q < size {
            // a. Perform ? Set(splitter, "lastIndex", q, true).
            splitter_obj.define_property("lastIndex".into(), q.into(), realm)?;

            // b. Let z be ? RegExpExec(splitter, S).
            let exec_prop = splitter_obj.get("exec", realm)?;
            let exec_result =
                exec_prop.call(realm, vec![string.clone().into()], splitter_value.clone())?;

            // c. If z is null, set q to AdvanceStringIndex(S, q, unicodeMatching).
            if exec_result.is_null() {
                q = advance_string_index(&string, q, full_unicode);
                continue;
            }

            // d. Else,
            let result_obj = exec_result.as_object()?;

            // i. Let e be ? ToLength(? Get(splitter, "lastIndex")).
            let e_val = splitter_obj.get("lastIndex", realm)?;
            let e_raw = e_val.to_number(realm)?;
            let e = (e_raw as usize).min(size);

            // iii. If e = p, set q to AdvanceStringIndex(S, q, unicodeMatching).
            if e == p {
                q = advance_string_index(&string, q, full_unicode);
                continue;
            }

            // iv. Else,
            // 1. Let T be the substring of S from p to q.
            // Convert UTF-16 indices to byte offsets for slicing
            let p_byte = utf16_index_to_byte_offset(&s, p);
            let q_byte = utf16_index_to_byte_offset(&s, q);
            let substring = &s[p_byte..q_byte];
            result_vec.push(YSString::from_ref(substring).into());

            // 4. If lengthA = lim, return A.
            if result_vec.len() >= lim {
                return Ok(Array::with_elements(realm, result_vec)?.into_value());
            }

            // 5. Set p to e.
            p = e;

            // 6-9. Add captured groups
            let result_length = result_obj.get("length", realm)?.to_number(realm)? as usize;
            let n_captures = result_length.saturating_sub(1);

            for i in 1..=n_captures {
                let cap = result_obj.get(i.to_string(), realm)?;
                result_vec.push(cap);

                if result_vec.len() >= lim {
                    return Ok(Array::with_elements(realm, result_vec)?.into_value());
                }
            }

            // 10. Set q to p.
            q = p;
        }

        // 20. Let T be the substring of S from p to size.
        let p_byte = utf16_index_to_byte_offset(&s, p);
        let substring = &s[p_byte..];
        result_vec.push(YSString::from_ref(substring).into());

        // 21. Return A.
        Ok(Array::with_elements(realm, result_vec)?.into_value())
    }

    #[prop("toString")]
    pub fn js_to_string(&self) -> YSString {
        let escaped = escape_pattern(&self.original_source);
        let mut result = String::new();
        result.push('/');
        result.push_str(&escaped.as_str_lossy());
        result.push('/');
        result.push_str(&self.original_flags.as_str_lossy());

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

impl RegExp {
    /// RegExpExec abstract operation (21.2.5.2.1)
    /// Calls the user-defined exec method if present, otherwise uses the built-in exec.
    fn regexp_exec(&self, this: &Value, string: YSString, realm: &mut Realm) -> ValueResult {
        let this_obj = this.as_object()?;

        // Step 1-2: Let exec be ? Get(R, "exec"). If IsCallable(exec) is true...
        let exec_prop = this_obj.get("exec", realm)?;
        if exec_prop.is_callable() {
            // Call the user-defined exec method
            let result = exec_prop.call(realm, vec![string.into()], this.clone())?;

            // Step 2.d: If result is not null and not an object, throw TypeError
            if !result.is_null() && !result.is_object() {
                return Err(Error::ty("RegExp exec must return an object or null"));
            }

            return Ok(result);
        }

        // Step 3: Use built-in exec
        self.exec(this, string, realm)
    }
}

impl PrettyObjectOverride for RegExp {
    fn pretty_inline(
        &self,
        _obj: &crate::value::Object,
        _not: &mut Vec<usize>,
        _realm: &mut Realm,
    ) -> Option<String> {
        let mut s = String::new();
        s.push('/');
        let escaped = escape_pattern(&self.original_source);
        s.push_str(&escaped.as_str_lossy());
        s.push('/');
        s.push_str(&self.original_flags.as_str_lossy());
        Some(s)
    }

    fn pretty_multiline(
        &self,
        obj: &crate::value::Object,
        not: &mut Vec<usize>,
        realm: &mut Realm,
    ) -> Option<String> {
        self.pretty_inline(obj, not, realm)
    }
}

fn escape_pattern(source: &YSString) -> YSString {
    if source.is_empty() {
        return YSString::from_ref("(?:)");
    }

    let mut escaped = String::with_capacity(source.len());

    for ch in source.as_str_lossy().chars() {
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

fn advance_string_index(input: &YSString, index: usize, unicode: bool) -> usize {
    let s = input.as_str_lossy();
    let len = s.len();

    if index >= len {
        return index.saturating_add(1);
    }

    if !unicode {
        // Even in non-unicode mode, we need to advance to a valid UTF-8 boundary
        // to avoid slicing in the middle of a multi-byte character
        let mut next = index + 1;
        while next < len && !s.is_char_boundary(next) {
            next += 1;
        }
        return next;
    }

    // In Unicode mode, we need to advance past surrogate pairs
    // We need to find the character at byte position `index`
    // and advance by the appropriate amount

    // First, verify we're at a char boundary
    if !s.is_char_boundary(index) {
        // We're in the middle of a character, find the next boundary
        let mut next = index + 1;
        while next < len && !s.is_char_boundary(next) {
            next += 1;
        }
        return next;
    }

    // Get the character at this position
    if let Some(ch) = s[index..].chars().next() {
        // Advance by the byte length of this character
        // In unicode mode, if it's an astral character (takes 2 UTF-16 code units),
        // we advance by its full UTF-8 byte length
        return index + ch.len_utf8();
    }

    index.saturating_add(1)
}

/// Convert a UTF-16 code unit index to a UTF-8 byte offset
fn utf16_index_to_byte_offset(s: &str, utf16_index: usize) -> usize {
    let mut byte_offset = 0;
    let mut code_unit_pos = 0;

    for ch in s.chars() {
        if code_unit_pos >= utf16_index {
            break;
        }
        byte_offset += ch.len_utf8();
        code_unit_pos += ch.len_utf16();
    }

    byte_offset
}

/// GetSubstitution as per ECMA-262 section 22.1.3.17.1
fn get_substitution(
    matched: &YSString,
    str: &str,
    position: usize,
    captures: &[Value],
    named_captures: &Value,
    replacement_template: &YSString,
    realm: &mut Realm,
) -> Res<YSString> {
    let string_length = str.len();
    let mut result = String::new();
    let template = replacement_template.as_str_lossy();
    let mut chars = template.chars().peekable();

    while let Some(c) = chars.next() {
        if c != '$' {
            result.push(c);
            continue;
        }

        // Look at next character
        match chars.peek() {
            Some('$') => {
                chars.next();
                result.push('$');
            }
            Some('`') => {
                chars.next();
                // Substring before match
                result.push_str(&str[..position]);
            }
            Some('&') => {
                chars.next();
                // The matched substring
                result.push_str(&matched.as_str_lossy());
            }
            Some('\'') => {
                chars.next();
                // Substring after match
                let match_length = matched.as_str_lossy().len();
                let tail_pos = (position + match_length).min(string_length);
                result.push_str(&str[tail_pos..]);
            }
            Some(c) if c.is_ascii_digit() => {
                // $1 through $99
                let first_digit = chars
                    .next()
                    .ok_or_else(|| Error::syn("Unexpected end of replacement template after $"))?;
                let mut index = (first_digit as usize) - ('0' as usize);

                // Check for second digit
                if let Some(&second) = chars.peek() {
                    if second.is_ascii_digit() {
                        let two_digit_index = index * 10 + (second as usize) - ('0' as usize);
                        // Only use the two-digit form if it's a valid index (1 <= nn <= m)
                        // $00 and $nn where nn > m should not consume the second digit
                        if two_digit_index >= 1 && two_digit_index <= captures.len() {
                            chars.next();
                            index = two_digit_index;
                        }
                    }
                }

                if index >= 1 && index <= captures.len() {
                    let capture = &captures[index - 1];
                    if !capture.is_undefined() {
                        let cap_str = capture.to_string(realm)?;
                        result.push_str(&cap_str.as_str_lossy());
                    }
                    // If undefined, append empty string (do nothing)
                } else {
                    // Not a valid index, keep literal
                    result.push('$');
                    result.push(first_digit);
                }
            }
            Some('<') => {
                chars.next();
                // Named capture group
                let mut group_name = String::new();
                let mut found_close = false;

                while let Some(&c) = chars.peek() {
                    if c == '>' {
                        chars.next();
                        found_close = true;
                        break;
                    }
                    group_name.push(chars.next().ok_or_else(|| {
                        Error::syn("Unexpected end of replacement template in named capture")
                    })?);
                }

                if !found_close {
                    // No closing > or no named captures, keep literal $<
                    result.push_str("$<");
                    result.push_str(&group_name);
                } else if named_captures.is_undefined() {
                    // Have closing > but no named captures, output $<name> literally
                    result.push_str("$<");
                    result.push_str(&group_name);
                    result.push('>');
                } else {
                    // Get the named capture - use get_property_opt to avoid throwing
                    // when the group name doesn't exist
                    let capture = named_captures.get_property_opt(group_name, realm)?;
                    if let Some(cap) = capture {
                        if !cap.is_undefined() {
                            let cap_str = cap.to_string(realm)?;
                            result.push_str(&cap_str.as_str_lossy());
                        }
                    }
                    // If property doesn't exist or is undefined, append empty string
                }
            }
            _ => {
                // Just a $ followed by something else, keep literal $
                result.push('$');
            }
        }
    }

    Ok(YSString::from(result))
}

/// EncodeForRegExpEscape as per ECMA-262
/// Escapes a string so it can be safely used as a literal pattern in a regular expression
fn escape_for_regexp(text: &str) -> YSString {
    let mut buf = String::with_capacity(text.len() * 2);

    for (index, c) in text.chars().enumerate() {
        let code = c as u32;

        // 4.a. If escaped is the empty String and c is matched by either DecimalDigit or AsciiLetter
        if index == 0 && (c.is_ascii_digit() || c.is_ascii_alphabetic()) {
            // Use \xXX format for initial digit/letter
            buf.push_str(&format!("\\x{code:02x}"));
            continue;
        }

        // EncodeForRegExpEscape step 1: SyntaxCharacter or U+002F (SOLIDUS)
        if matches!(
            c,
            '^' | '$'
                | '\\'
                | '.'
                | '*'
                | '+'
                | '?'
                | '('
                | ')'
                | '['
                | ']'
                | '{'
                | '}'
                | '|'
                | '/'
        ) {
            buf.push('\\');
            buf.push(c);
        }
        // Step 2: ControlEscape characters (Table 64)
        else if c == '\x09' {
            buf.push_str("\\t");
        } else if c == '\x0A' {
            buf.push_str("\\n");
        } else if c == '\x0B' {
            buf.push_str("\\v");
        } else if c == '\x0C' {
            buf.push_str("\\f");
        } else if c == '\x0D' {
            buf.push_str("\\r");
        }
        // Step 3-5: otherPunctuators or WhiteSpace or LineTerminator
        else if is_other_punctuator(c) || is_whitespace(c) || is_line_terminator(c) {
            if code <= 0xFF {
                // Use \xXX format
                buf.push_str(&format!("\\x{code:02x}"));
            } else {
                // Use \uXXXX format
                buf.push_str(&format!("\\u{code:04x}"));
            }
        }
        // Step 6: All other Unicode characters
        else {
            buf.push(c);
        }
    }

    YSString::from(buf)
}

/// Check if character is an "otherPunctuator" for RegExp.escape
const fn is_other_punctuator(c: char) -> bool {
    matches!(
        c,
        ',' | '-'
            | '='
            | '<'
            | '>'
            | '#'
            | '&'
            | '!'
            | '%'
            | ':'
            | ';'
            | '@'
            | '~'
            | '\''
            | '`'
            | '"'
    )
}

/// Check if character is WhiteSpace for RegExp.escape
const fn is_whitespace(c: char) -> bool {
    matches!(
        c,
        '\u{0009}' | // <TAB>
        '\u{000B}' | // <VT>
        '\u{000C}' | // <FF>
        '\u{0020}' | // <SP>
        '\u{00A0}' | // <NBSP>
        '\u{FEFF}' | // <ZWNBSP>
        '\u{1680}' | // Ogham Space Mark
        '\u{2000}' | '\u{2001}' | '\u{2002}' | '\u{2003}' | '\u{2004}' |
        '\u{2005}' | '\u{2006}' | '\u{2007}' | '\u{2008}' | '\u{2009}' |
        '\u{200A}' | // Various space separators
        '\u{202F}' | // Narrow No-Break Space
        '\u{205F}' | // Medium Mathematical Space
        '\u{3000}' // Ideographic Space
    )
}

/// Check if character is a LineTerminator for RegExp.escape
const fn is_line_terminator(c: char) -> bool {
    matches!(
        c,
        '\u{000A}' | // <LF>
        '\u{000D}' | // <CR>
        '\u{2028}' | // <LS>
        '\u{2029}' // <PS>
    )
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

// ============================================================================
// RegExpStringIterator - 22.2.9
// ============================================================================

#[object(name)]
#[derive(Debug)]
pub struct RegExpStringIterator {
    matcher: ObjectHandle,
    string: YSString,
    global: bool,
    unicode: bool,
    done: Cell<bool>,
}

impl crate::value::CustomName for RegExpStringIterator {
    fn custom_name(&self) -> String {
        "RegExp String Iterator".to_owned()
    }
}

impl RegExpStringIterator {
    pub fn new(
        matcher: ObjectHandle,
        string: YSString,
        global: bool,
        unicode: bool,
        realm: &mut Realm,
    ) -> Res<Self> {
        let proto = realm
            .intrinsics
            .clone_public()
            .regexp_string_iter
            .get(realm)?
            .clone();
        Ok(Self {
            matcher,
            string,
            global,
            unicode,
            done: Cell::new(false),
            inner: RefCell::new(MutableRegExpStringIterator {
                object: MutObject::with_proto(proto),
            }),
        })
    }
}

#[properties]
impl RegExpStringIterator {
    #[prop]
    pub fn next(&self, _args: Vec<Value>, realm: &mut Realm) -> ValueResult {
        // 22.2.9.2.1 %RegExpStringIteratorPrototype%.next ( )

        // If done, return { value: undefined, done: true }
        if self.done.get() {
            return Ok(create_iter_result_object(Value::Undefined, true, realm)?.into_value());
        }

        // i. Let match be ? RegExpExec(R, S).
        let exec_prop = self.matcher.get("exec", realm)?;
        let match_result = exec_prop.call(
            realm,
            vec![self.string.clone().into()],
            Value::from(self.matcher.clone()),
        )?;

        // ii. If match is null, then
        if match_result.is_null() {
            // 1. Set done to true
            self.done.set(true);
            // 2. Return CreateIterResultObject(undefined, true)
            return Ok(create_iter_result_object(Value::Undefined, true, realm)?.into_value());
        }

        // iii. If global is false, then
        if !self.global {
            // 1. Set done to true
            self.done.set(true);
            // 2. Return CreateIterResultObject(match, false)
            return Ok(create_iter_result_object(match_result, false, realm)?.into_value());
        }

        // iv. Else (global is true),
        let match_obj = match_result.as_object()?;

        // 1. Let matchStr be ? ToString(? Get(match, "0")).
        let match_str = match_obj.get("0", realm)?.to_string(realm)?;

        // 2. If matchStr is the empty String, then
        if match_str.is_empty() {
            // a. Let thisIndex be ? ToLength(? Get(R, "lastIndex")).
            let this_index = self.matcher.get("lastIndex", realm)?.to_number(realm)? as usize;

            // b. Let nextIndex be AdvanceStringIndex(S, thisIndex, fullUnicode).
            let next_index = advance_string_index(&self.string, this_index, self.unicode);

            // c. Perform ? Set(R, "lastIndex", nextIndex, true).
            self.matcher
                .define_property("lastIndex".into(), next_index.into(), realm)?;
        }

        // 3. Return CreateIterResultObject(match, false)
        Ok(create_iter_result_object(match_result, false, realm)?.into_value())
    }
}

impl Intrinsic for RegExpStringIterator {
    fn initialize(realm: &mut Realm) -> Res<ObjectHandle> {
        let iter_proto = Iterator::get_intrinsic(realm)?;
        Self::initialize_proto(
            Object::raw_with_proto(iter_proto),
            realm.intrinsics.func.clone(),
            realm,
        )
    }

    fn get_intrinsic(realm: &mut Realm) -> Res<ObjectHandle> {
        Ok(realm
            .intrinsics
            .clone_public()
            .regexp_string_iter
            .get(realm)?
            .clone())
    }
}
