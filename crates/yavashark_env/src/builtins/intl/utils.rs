#![allow(unused)]
use crate::array::Array;
use crate::value::Obj;
use crate::{Error, ObjectHandle, Realm, Res, Value};

use std::fmt::Display;

const DEFAULT_LOCALE: &str = "und";

pub fn canonicalize_locale_tag(tag: &str) -> Result<String, Error> {
    let s = tag.trim();
    if s.is_empty() {
        Ok(DEFAULT_LOCALE.to_string())
    } else {
        // TODO: replace with full Unicode Locale ID canonicalization via provider
        Ok(s.to_string())
    }
}

pub fn canonicalize_locale_list(locales: Option<Value>, realm: &mut Realm) -> Res<Vec<String>> {
    let Some(value) = locales else {
        return Ok(vec![DEFAULT_LOCALE.to_string()]);
    };

    if value.is_nullish() {
        return Ok(vec![DEFAULT_LOCALE.to_string()]);
    }

    if value.is_string() {
        let s = value.to_string(realm)?.to_string();
        let s = s.trim();
        if s.is_empty() {
            return Ok(vec![DEFAULT_LOCALE.to_string()]);
        }
        let list = s
            .split(',')
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty())
            .map(|p| canonicalize_locale_tag(&p).unwrap_or_else(|_| DEFAULT_LOCALE.to_string()))
            .collect();
        return Ok(list);
    }

    if let Value::Object(ref obj) = value {
        let mut out = Vec::new();
        let mut idx = 0usize;
        loop {
            let key = idx.to_string();
            let v = obj.get(key, realm)?;
            if v.is_undefined() {
                break;
            }
            if v.is_nullish() {
                idx += 1;
                continue;
            }
            let tag = v.to_string(realm)?.to_string();
            let canonical =
                canonicalize_locale_tag(&tag).map_err(|e| Error::range_error(e.to_string()))?;
            out.push(canonical);
            idx += 1;
        }
        if out.is_empty() {
            Ok(vec![DEFAULT_LOCALE.to_string()])
        } else {
            Ok(out)
        }
    } else {
        // Fallback: coerce to string
        let s = value.to_string(realm)?.to_string();
        if s.trim().is_empty() {
            Ok(vec![DEFAULT_LOCALE.to_string()])
        } else {
            Ok(vec![canonicalize_locale_tag(&s.trim())
                .map_err(|e| Error::range_error(e.to_string()))?])
        }
    }
}

pub fn get_canonical_locales_object(
    locales: Option<Value>,
    realm: &mut Realm,
) -> Res<ObjectHandle> {
    let list = canonicalize_locale_list(locales, realm)?
        .into_iter()
        .map(Into::into)
        .collect();

    let arr = Array::with_elements(realm, list)?;

    Ok(arr.into_object())
}

pub fn get_option_string(
    options: &ObjectHandle,
    key: &'static str,
    allowed: &[&str],
    realm: &mut Realm,
) -> Res<Option<String>> {
    let v = options.get(key, realm)?;
    if v.is_undefined() {
        return Ok(None);
    }
    let raw = v.to_string(realm)?.to_string();
    let normalized = raw.trim().to_lowercase();
    if !allowed.is_empty()
        && !allowed
            .iter()
            .any(|entry| entry.eq_ignore_ascii_case(&normalized))
    {
        return Err(Error::range_error(format!(
            "Invalid value for Intl option {key}"
        )));
    }
    Ok(Some(if allowed.is_empty() {
        raw.trim().to_string()
    } else {
        normalized
    }))
}

pub fn get_option_bool(
    options: &ObjectHandle,
    key: &'static str,
    realm: &mut Realm,
) -> Res<Option<bool>> {
    let v = options.get(key, realm)?;
    if v.is_undefined() {
        return Ok(None);
    }
    Ok(Some(v.is_truthy()))
}

pub fn map_provider_error<E: Display>(e: E) -> Error {
    Error::range_error(e.to_string())
}

pub fn supported_values_of(key: &str) -> Result<Vec<&'static str>, Error> {
    match key {
        "numberingSystem" => Ok(vec!["latn", "arab", "arabext"]),
        "calendar" => Ok(vec!["gregory", "buddhist", "iso8601"]),
        "timeZone" => Ok(vec!["UTC", "Etc/UTC"]),
        _ => Err(Error::ty_error(format!(
            "Unsupported key for supportedValuesOf: {key}"
        ))),
    }
}

pub fn validate_currency_code(code: &str) -> Result<(), Error> {
    let trimmed = code.trim();
    if trimmed.len() == 3 && trimmed.chars().all(|c| c.is_ascii_alphabetic()) {
        Ok(())
    } else {
        Err(Error::range_error(format!("Invalid currency code: {code}")))
    }
}
