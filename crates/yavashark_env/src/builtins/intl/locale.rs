use crate::value::Obj;
use crate::{Error, MutObject, ObjectHandle, Realm, Res, Value};
use icu::locale::extensions::unicode::{Key, Value as UnicodeValue};
use icu::locale::subtags::{Variant, Variants};
use icu::locale::{Locale as IcuLocale, LocaleCanonicalizer, LocaleExpander};
use std::cell::RefCell;
use yavashark_macro::{data_object, object, props};
use crate::builtins::intl::utils::HourCycle;

#[data_object(error = "range")]
pub enum CaseFirst {
    Upper,
    Lower,
    False,
}

#[data_object]
pub struct LocaleOptions {
    pub language: Option<String>,
    pub script: Option<String>,
    pub region: Option<String>,
    pub variants: Option<String>,
    pub calendar: Option<String>,
    pub collation: Option<String>,
    #[prop("hourCycle")]
    pub hour_cycle: Option<HourCycle>,
    #[prop("caseFirst")]
    pub case_first: Option<CaseFirst>,
    pub numeric: Option<bool>,
    #[prop("numberingSystem")]
    pub numbering_system: Option<String>,
}

#[object]
#[derive(Debug)]
pub struct Locale {
    #[mutable]
    locale: IcuLocale,
    #[mutable]
    numeric: bool,
}

impl Locale {
    fn new_with_locale(realm: &mut Realm, locale: IcuLocale, numeric: bool) -> Res<Self> {
        Ok(Self {
            inner: RefCell::new(MutableLocale {
                object: MutObject::with_proto(
                    realm
                        .intrinsics
                        .clone_public()
                        .intl_locale
                        .get(realm)?
                        .clone(),
                ),
                locale,
                numeric,
            }),
        })
    }
}


fn get_extension_value(locale: &IcuLocale, key_str: &str) -> Option<String> {
    let key = Key::try_from_str(key_str).ok()?;

    locale
        .extensions
        .unicode
        .keywords
        .get(&key)
        .map(ToString::to_string)
}

fn set_extension_value(locale: &mut IcuLocale, key_str: &str, value_str: &str) {
    let Ok(key) = Key::try_from_str(key_str) else {
        return;
    };

    let Ok(value) = UnicodeValue::try_from_str(value_str) else {
        return;
    };

    locale.extensions.unicode.keywords.set(key, value);
}

// https://tc39.es/ecma402/#sec-intl-locale-constructor
#[props(intrinsic_name = intl_locale, to_string_tag = "Intl.Locale")]
impl Locale {
    // https://tc39.es/ecma402/#sec-Intl.Locale
    // The constructor must follow a specific evaluation order:
    // 1. First: tag.toString() (if tag is not a Locale object)
    // 2. Then CoerceOptionsToObject(options)
    // 3. Then UpdateLanguageId (reads language, script, region, variants in order)
    // 4. Then GetOption for: calendar, collation, hourCycle, caseFirst, numeric, numberingSystem
    #[constructor]
    fn construct(tag: &Value, options: Option<LocaleOptions>, realm: &mut Realm) -> Res<Self> {
        // Step 7: If tag is not a String and tag is not an Object, throw TypeError
        if tag.is_undefined() || tag.is_null() {
            return Err(Error::ty_error(
                "First argument to Intl.Locale must be a string or Locale object".to_string(),
            ));
        }

        // Steps 8-9: Get the tag string
        // If tag is a Locale object, use its [[Locale]] internal slot
        // Otherwise, call ToString(tag) FIRST before processing options
        let tag_str = if let Value::Object(obj) = tag {
            if let Some(locale_obj) = obj.downcast::<Self>() {
                locale_obj.inner.borrow().locale.to_string()
            } else {
                tag.to_string(realm)?.to_string()
            }
        } else {
            tag.to_string(realm)?.to_string()
        };

        // Step 11: Validate the tag is structurally valid
        if tag_str.contains('_') {
            return Err(Error::range_error(
                "locale is not a structurally valid language tag".to_string(),
            ));
        }

        let mut locale: IcuLocale = tag_str.parse().map_err(|_| {
            Error::range_error(format!("Invalid language tag: {tag_str}"))
        })?;

        // Step 12: Canonicalize the locale
        let canonicalizer = LocaleCanonicalizer::new_extended();
        canonicalizer.canonicalize(&mut locale);

        // Get initial numeric value from locale extension
        let mut numeric = get_extension_value(&locale, "kn")
            .is_some_and(|v| v.is_empty() || v == "true");

        // Step 13: UpdateLanguageId - process language, script, region, variants in ORDER
        if let Some(opts) = options {
            // language (first in UpdateLanguageId)
            if let Some(lang) = opts.language {
                if let Ok(language) = lang.parse() {
                    locale.id.language = language;
                } else {
                    return Err(Error::range_error(format!(
                        "Invalid language subtag: {lang}"
                    )));
                }
            }

            // script (second in UpdateLanguageId)
            if let Some(scr) = opts.script {
                if let Ok(script) = scr.parse() {
                    locale.id.script = Some(script);
                } else {
                    return Err(Error::range_error(format!(
                        "Invalid script subtag: {scr}"
                    )));
                }
            }

            // region (third in UpdateLanguageId)
            if let Some(reg) = opts.region {
                if let Ok(region) = reg.parse() {
                    locale.id.region = Some(region);
                } else {
                    return Err(Error::range_error(format!(
                        "Invalid region subtag: {reg}"
                    )));
                }
            }

            // variants (fourth in UpdateLanguageId)
            if let Some(variants_str) = opts.variants {
                if variants_str.is_empty() {
                    return Err(Error::range_error(
                        "Invalid variant subtag: empty string".to_string(),
                    ));
                }
                let lower_variants = variants_str.to_ascii_lowercase();
                let variant_subtags: Vec<&str> = lower_variants.split('-').collect();

                // Check for duplicates
                let mut seen = std::collections::HashSet::new();
                for variant in &variant_subtags {
                    if !seen.insert(*variant) {
                        return Err(Error::range_error(format!(
                            "Duplicate variant subtag: {variant}"
                        )));
                    }
                }

                // Parse and collect variants into a Vec
                let mut parsed_variants: Vec<Variant> = Vec::new();
                for variant in variant_subtags {
                    if let Ok(v) = variant.parse() {
                        parsed_variants.push(v);
                    } else {
                        return Err(Error::range_error(format!(
                            "Invalid variant subtag: {variant}"
                        )));
                    }
                }
                
                // Sort and set variants
                parsed_variants.sort();
                parsed_variants.dedup();
                locale.id.variants = Variants::from_vec_unchecked(parsed_variants);
            }

            // Step 15-17: calendar
            if let Some(cal) = opts.calendar {
                if !is_valid_unicode_type(&cal) {
                    return Err(Error::range_error(format!("Invalid calendar: {cal}")));
                }
                set_extension_value(&mut locale, "ca", &cal);
            }

            // Step 18-20: collation
            if let Some(col) = opts.collation {
                if !is_valid_unicode_type(&col) {
                    return Err(Error::range_error(format!("Invalid collation: {col}")));
                }
                set_extension_value(&mut locale, "co", &col);
            }

            // Step 21-22: hourCycle
            if let Some(hc) = opts.hour_cycle {
                set_extension_value(&mut locale, "hc", hc.as_str());
            }

            // Step 23-24: caseFirst
            if let Some(cf) = opts.case_first {
                set_extension_value(&mut locale, "kf", cf.as_str());
            }

            // Step 25-27: numeric (boolean, converted to string for extension)
            if let Some(num) = opts.numeric {
                numeric = num;
                if num {
                    set_extension_value(&mut locale, "kn", "true");
                } else {
                    set_extension_value(&mut locale, "kn", "false");
                }
            }

            // Step 28-30: numberingSystem
            if let Some(nu) = opts.numbering_system {
                if !is_valid_unicode_type(&nu) {
                    return Err(Error::range_error(format!("Invalid numberingSystem: {nu}")));
                }
                set_extension_value(&mut locale, "nu", &nu);
            }
        }

        // Step 31: MakeLocaleRecord - canonicalize again after applying options
        canonicalizer.canonicalize(&mut locale);

        Self::new_with_locale(realm, locale, numeric)
    }

    // https://tc39.es/ecma402/#sec-Intl.Locale.prototype.baseName
    #[get("baseName")]
    fn base_name(&self) -> String {
        self.inner.borrow().locale.id.to_string()
    }

    // https://tc39.es/ecma402/#sec-Intl.Locale.prototype.calendar
    #[get("calendar")]
    fn calendar(&self) -> Option<String> {
        get_extension_value(&self.inner.borrow().locale, "ca")
    }

    // https://tc39.es/ecma402/#sec-Intl.Locale.prototype.caseFirst
    #[get("caseFirst")]
    fn case_first(&self) -> Option<String> {
        get_extension_value(&self.inner.borrow().locale, "kf")
    }

    // https://tc39.es/ecma402/#sec-Intl.Locale.prototype.collation
    #[get("collation")]
    fn collation(&self) -> Option<String> {
        get_extension_value(&self.inner.borrow().locale, "co")
    }

    // https://tc39.es/ecma402/#sec-Intl.Locale.prototype.hourCycle
    #[get("hourCycle")]
    fn hour_cycle(&self) -> Option<String> {
        get_extension_value(&self.inner.borrow().locale, "hc")
    }

    // https://tc39.es/ecma402/#sec-Intl.Locale.prototype.language
    #[get("language")]
    fn language(&self) -> String {
        self.inner.borrow().locale.id.language.to_string()
    }

    // https://tc39.es/ecma402/#sec-Intl.Locale.prototype.numberingSystem
    #[get("numberingSystem")]
    fn numbering_system(&self) -> Option<String> {
        get_extension_value(&self.inner.borrow().locale, "nu")
    }

    // https://tc39.es/ecma402/#sec-Intl.Locale.prototype.numeric
    #[get("numeric")]
    fn numeric(&self) -> bool {
        self.inner.borrow().numeric
    }

    // https://tc39.es/ecma402/#sec-Intl.Locale.prototype.region
    #[get("region")]
    fn region(&self) -> Option<String> {
        self.inner
            .borrow()
            .locale
            .id
            .region
            .map(|r| r.to_string())
    }

    // https://tc39.es/ecma402/#sec-Intl.Locale.prototype.script
    #[get("script")]
    fn script(&self) -> Option<String> {
        self.inner.borrow().locale.id.script.map(|s| s.to_string())
    }

    // https://tc39.es/ecma402/#sec-Intl.Locale.prototype.variants
    #[get("variants")]
    fn variants(&self) -> Option<String> {
        let inner = self.inner.borrow();
        let variants = &inner.locale.id.variants;
        if variants.is_empty() {
            None
        } else {
            Some(variants.to_string())
        }
    }

    // https://tc39.es/ecma402/#sec-Intl.Locale.prototype.maximize
    #[prop("maximize")]
    fn maximize(&self, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let mut locale = self.inner.borrow().locale.clone();
        let numeric = self.inner.borrow().numeric;

        let expander = LocaleExpander::new_extended();
        expander.maximize(&mut locale.id);

        Ok(Self::new_with_locale(realm, locale, numeric)?.into_object())
    }

    // https://tc39.es/ecma402/#sec-Intl.Locale.prototype.minimize
    #[prop("minimize")]
    fn minimize(&self, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let mut locale = self.inner.borrow().locale.clone();
        let numeric = self.inner.borrow().numeric;

        let expander = LocaleExpander::new_extended();
        expander.minimize(&mut locale.id);

        Ok(Self::new_with_locale(realm, locale, numeric)?.into_object())
    }

    // https://tc39.es/ecma402/#sec-Intl.Locale.prototype.toString
    #[prop("toString")]
    fn to_string(&self) -> String {
        self.inner.borrow().locale.to_string()
    }
}

fn is_valid_unicode_type(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    for part in s.split('-') {
        if part.len() < 3 || part.len() > 8 {
            return false;
        }
        if !part.chars().all(|c| c.is_ascii_alphanumeric()) {
            return false;
        }
    }

    true
}
