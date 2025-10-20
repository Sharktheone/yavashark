use crate::conversion::downcast_obj;
use crate::utils::coerce_object;
use crate::value::Obj;
use crate::{
    Error, MutObject, NativeFunction, Object, ObjectHandle, Realm, Res, Value, ValueResult,
    Variable,
};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::str::FromStr;
use unicode_normalization::{char::is_combining_mark, UnicodeNormalization};
use yavashark_macro::{object, props};

const DEFAULT_LOCALE: &str = "und";
const USAGE_SORT: &str = "sort";
const USAGE_SEARCH: &str = "search";
const CASE_FIRST_FALSE: &str = "false";
const SENSITIVITY_BASE: &str = "base";
const SENSITIVITY_ACCENT: &str = "accent";
const SENSITIVITY_CASE: &str = "case";
const SENSITIVITY_VARIANT: &str = "variant";

#[object]
#[derive(Debug)]
pub struct Collator {
    #[mutable]
    initialized: bool,
    #[mutable]
    locale: String,
    #[mutable]
    usage: String,
    #[mutable]
    collation: String,
    #[mutable]
    numeric: bool,
    #[mutable]
    case_first: String,
    #[mutable]
    sensitivity: String,
    #[mutable]
    ignore_punctuation: bool,
    #[mutable]
    bound_compare: Option<ObjectHandle>,
}

impl Collator {
    pub fn new(realm: &mut Realm) -> Res<Self> {
        Ok(Self {
            inner: RefCell::new(MutableCollator {
                object: MutObject::with_proto(realm.intrinsics.clone_public().intl_collator.get(realm)?.clone()),
                initialized: false,
                locale: DEFAULT_LOCALE.to_string(),
                usage: USAGE_SORT.to_string(),
                collation: "default".to_string(),
                numeric: false,
                case_first: CASE_FIRST_FALSE.to_string(),
                sensitivity: SENSITIVITY_VARIANT.to_string(),
                ignore_punctuation: false,
                bound_compare: None,
            }),
        })
    }
}

#[props(intrinsic_name = intl_collator, to_string_tag = "Intl.Collator")]
impl Collator {
    #[constructor]
    fn construct(
        locales: Option<Value>,
        options: Option<Value>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let locale = canonicalize_locale(locales, realm)?;

        let options_value = options.unwrap_or(Value::Undefined);
        let options_obj = if options_value.is_undefined() || options_value.is_nullish() {
            Object::with_proto(realm.intrinsics.obj.clone())
        } else {
            coerce_object(options_value, realm)?
        };

        let usage = get_option_string(&options_obj, "usage", &[USAGE_SORT, USAGE_SEARCH], realm)?
            .unwrap_or_else(|| USAGE_SORT.to_string());

        let collation = get_option_string(&options_obj, "collation", &[], realm)?
            .unwrap_or_else(|| "default".to_string());

        let numeric = get_option_bool(&options_obj, "numeric", realm)?.unwrap_or(false);

        let case_first = get_option_string(
            &options_obj,
            "caseFirst",
            &["upper", "lower", CASE_FIRST_FALSE],
            realm,
        )?
        .unwrap_or_else(|| CASE_FIRST_FALSE.to_string());

        let sensitivity_default = if usage == USAGE_SEARCH {
            SENSITIVITY_ACCENT
        } else {
            SENSITIVITY_VARIANT
        };

        let sensitivity = get_option_string(
            &options_obj,
            "sensitivity",
            &[
                SENSITIVITY_BASE,
                SENSITIVITY_ACCENT,
                SENSITIVITY_CASE,
                SENSITIVITY_VARIANT,
            ],
            realm,
        )?
        .unwrap_or_else(|| sensitivity_default.to_string());

        let ignore_punctuation =
            get_option_bool(&options_obj, "ignorePunctuation", realm)?.unwrap_or(false);

        let collator = Self::new(realm)?;

        {
            let mut inner = collator.inner.borrow_mut();
            inner.locale = locale;
            inner.usage = usage;
            inner.collation = collation;
            inner.numeric = numeric;
            inner.case_first = case_first;
            inner.sensitivity = sensitivity;
            inner.ignore_punctuation = ignore_punctuation;
            inner.initialized = true;
            inner.bound_compare = None;
        }

        Ok(collator.into_object())
    }

    #[get("compare")]
    fn compare(&self, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        self.ensure_initialized()?;

        let mut inner = self.inner.borrow_mut();
        if let Some(existing) = &inner.bound_compare {
            return Ok(existing.clone());
        }

        let func = NativeFunction::with_proto_and_len(
            "compare",
            |args, this, realm| {
                let collator = downcast_obj::<Self>(this.copy())?;
                let left = args.get(0).map_or(Value::Undefined, Value::copy);
                let right = args.get(1).map_or(Value::Undefined, Value::copy);

                Self::compare_values(&collator, realm, left, right)
            },
            realm.intrinsics.func.clone(),
            2,
            realm,
        );

        inner.bound_compare = Some(func.clone());

        Ok(func)
    }

    #[prop("resolvedOptions")]
    fn resolved_options(&self, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        self.ensure_initialized()?;

        let (locale, usage, sensitivity, ignore_punctuation, collation, numeric, case_first) = {
            let inner = self.inner.borrow();
            (
                inner.locale.clone(),
                inner.usage.clone(),
                inner.sensitivity.clone(),
                inner.ignore_punctuation,
                inner.collation.clone(),
                inner.numeric,
                inner.case_first.clone(),
            )
        };

        let result = Object::with_proto(realm.intrinsics.obj.clone());

        result.define_property_attributes(
            "locale".into(),
            Variable::write_config(locale.into()),
            realm,
        )?;
        result.define_property_attributes(
            "usage".into(),
            Variable::write_config(usage.into()),
            realm,
        )?;
        result.define_property_attributes(
            "sensitivity".into(),
            Variable::write_config(sensitivity.into()),
            realm,
        )?;
        result.define_property_attributes(
            "ignorePunctuation".into(),
            Variable::write_config(Value::Boolean(ignore_punctuation)),
            realm,
        )?;
        result.define_property_attributes(
            "collation".into(),
            Variable::write_config(collation.into()),
            realm,
        )?;
        result.define_property_attributes(
            "numeric".into(),
            Variable::write_config(Value::Boolean(numeric)),
            realm,
        )?;
        result.define_property_attributes(
            "caseFirst".into(),
            Variable::write_config(case_first.into()),
            realm,
        )?;

        Ok(result)
    }
}

impl Collator {
    fn ensure_initialized(&self) -> Res<()> {
        if self.inner.borrow().initialized {
            Ok(())
        } else {
            Err(Error::ty("Intl.Collator object is not initialized"))
        }
    }

    fn compare_values(
        collator: &Self,
        realm: &mut Realm,
        left: Value,
        right: Value,
    ) -> ValueResult {
        collator.ensure_initialized()?;

        let left_s = left.to_string(realm)?.to_string();
        let right_s = right.to_string(realm)?.to_string();

        let ordering = collator.compare_prepared(&left_s, &right_s);

        Ok(match ordering {
            Ordering::Less => Value::Number(-1.0),
            Ordering::Greater => Value::Number(1.0),
            Ordering::Equal => Value::Number(0.0),
        })
    }

    fn compare_prepared(&self, a: &str, b: &str) -> Ordering {
        let (numeric, ignore_punctuation, sensitivity, case_first) = {
            let inner = self.inner.borrow();
            (
                inner.numeric,
                inner.ignore_punctuation,
                inner.sensitivity.clone(),
                inner.case_first.clone(),
            )
        };

        if numeric {
            if let Some(ord) = compare_numeric(a, b) {
                if ord != Ordering::Equal {
                    return ord;
                }
            }
        }

        let prepared_a = preprocess_string(a, &sensitivity, ignore_punctuation);
        let prepared_b = preprocess_string(b, &sensitivity, ignore_punctuation);

        let ordering = prepared_a.cmp(&prepared_b);
        if ordering != Ordering::Equal {
            return ordering;
        }

        let case_order = apply_case_first(&case_first, a, b);
        if case_order != Ordering::Equal {
            return case_order;
        }

        a.cmp(b)
    }
}

fn canonicalize_locale(locales: Option<Value>, realm: &mut Realm) -> Res<String> {
    let Some(value) = locales else {
        return Ok(DEFAULT_LOCALE.to_string());
    };

    if value.is_nullish() {
        return Ok(DEFAULT_LOCALE.to_string());
    }

    let locale_str = value.to_string(realm)?.trim().to_string();

    if locale_str.is_empty() {
        Ok(DEFAULT_LOCALE.to_string())
    } else {
        Ok(locale_str)
    }
}

fn get_option_string(
    options: &ObjectHandle,
    key: &'static str,
    allowed: &[&str],
    realm: &mut Realm,
) -> Res<Option<String>> {
    let value = options.get(key, realm)?;

    if value.is_undefined() {
        return Ok(None);
    }

    let raw = value.to_string(realm)?.to_string();
    let normalized = raw.trim().to_lowercase();

    if !allowed.is_empty()
        && !allowed
            .iter()
            .any(|entry| entry.eq_ignore_ascii_case(&normalized))
    {
        return Err(Error::range_error(format!(
            "Invalid value for Intl.Collator option {key}"
        )));
    }

    Ok(Some(if allowed.is_empty() {
        raw.trim().to_string()
    } else {
        normalized
    }))
}

fn get_option_bool(
    options: &ObjectHandle,
    key: &'static str,
    realm: &mut Realm,
) -> Res<Option<bool>> {
    let value = options.get(key, realm)?;

    if value.is_undefined() {
        return Ok(None);
    }

    Ok(Some(value.is_truthy()))
}

fn compare_numeric(a: &str, b: &str) -> Option<Ordering> {
    let a_num = parse_numeric(a)?;
    let b_num = parse_numeric(b)?;

    a_num.partial_cmp(&b_num)
}

fn parse_numeric(value: &str) -> Option<f64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    let parsed = f64::from_str(trimmed).ok()?;
    parsed.partial_cmp(&parsed)?; // Reject NaN
    Some(parsed)
}

fn preprocess_string(input: &str, sensitivity: &str, ignore_punctuation: bool) -> String {
    let filtered = if ignore_punctuation {
        input
            .chars()
            .filter(|c| !is_punctuation(*c))
            .collect::<String>()
    } else {
        input.to_string()
    };

    match sensitivity {
        SENSITIVITY_BASE => strip_diacritics(&filtered.to_lowercase()),
        SENSITIVITY_ACCENT => filtered.to_lowercase(),
        SENSITIVITY_CASE => strip_diacritics(&filtered),
        _ => filtered,
    }
}

fn strip_diacritics(value: &str) -> String {
    value
        .nfd()
        .filter(|c| !is_combining_mark(*c))
        .collect::<String>()
        .nfc()
        .collect()
}

fn is_punctuation(ch: char) -> bool {
    ch.is_ascii_punctuation()
}

fn apply_case_first(case_first: &str, a: &str, b: &str) -> Ordering {
    match case_first {
        "upper" => match (leading_case_category(a), leading_case_category(b)) {
            (Some(CaseCategory::Upper), Some(CaseCategory::Lower)) => Ordering::Less,
            (Some(CaseCategory::Lower), Some(CaseCategory::Upper)) => Ordering::Greater,
            _ => Ordering::Equal,
        },
        "lower" => match (leading_case_category(a), leading_case_category(b)) {
            (Some(CaseCategory::Lower), Some(CaseCategory::Upper)) => Ordering::Less,
            (Some(CaseCategory::Upper), Some(CaseCategory::Lower)) => Ordering::Greater,
            _ => Ordering::Equal,
        },
        _ => Ordering::Equal,
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum CaseCategory {
    Upper,
    Lower,
    Other,
}

fn leading_case_category(value: &str) -> Option<CaseCategory> {
    for ch in value.chars() {
        if ch.is_alphabetic() {
            if ch.is_uppercase() {
                return Some(CaseCategory::Upper);
            }
            if ch.is_lowercase() {
                return Some(CaseCategory::Lower);
            }
            return Some(CaseCategory::Other);
        }
    }

    None
}
