use crate::value::{IntoValue, Obj};
use crate::{Error, MutObject, NativeFunction, Object, ObjectHandle, PropertyKey, Realm, Res, Value};
use icu::collator::options::{AlternateHandling, CaseLevel, CollatorOptions, Strength};
use icu::collator::preferences::{CollationCaseFirst, CollationNumericOrdering};
use icu::collator::{Collator as IcuCollator, CollatorPreferences};
use icu::locale::Locale;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::sync::Arc;
use yavashark_macro::{data_object, object, props};
use crate::builtins::intl::utils::LocaleMatcherOptions;

#[derive(Debug, Clone, Copy, Default)]
#[data_object(error = "range")]
pub enum Usage {
    #[default]
    Sort,
    Search,
}

#[derive(Debug, Clone, Copy, Default)]
#[data_object(error = "range")]
pub enum Sensitivity {
    Base,
    Accent,
    Case,
    #[default]
    Variant,
}

#[derive(Debug, Clone, Copy)]
#[data_object(error = "range")]
pub enum CaseFirst {
    Upper,
    Lower,
    False,
}


#[derive(Debug, Clone, Copy)]
#[data_object(error = "range")]
pub enum LocaleMatcher {
    Lookup,
    BestFit,
}


/// Parsed Collator options with proper RangeError handling
#[data_object]
#[derive(Debug, Default)]
struct CollatorOptionsJs {
    usage: Option<Usage>,
    locale_matcher: Option<LocaleMatcher>,
    numeric: Option<bool>,
    case_first: Option<CaseFirst>,
    sensitivity: Option<Sensitivity>,
    collation: Option<String>,
    ignore_punctuation: Option<bool>,
}


/// Internal configuration for creating ICU collators
#[derive(Clone, Debug)]
struct CollatorConfig {
    prefs: CollatorPreferences,
    opts: CollatorOptions,
}

#[object]
#[derive(Debug)]
pub struct Collator {
    #[mutable]
    config: Arc<CollatorConfig>,
    #[mutable]
    locale: String,
    #[mutable]
    usage: Usage,
    #[mutable]
    sensitivity: Sensitivity,
    #[mutable]
    ignore_punctuation: bool,
    #[mutable]
    collation: String,
    #[mutable]
    numeric: bool,
    #[mutable]
    case_first: Option<CaseFirst>,
    #[mutable]
    bound_compare: Option<ObjectHandle>,
}

impl Collator {
    fn create(
        realm: &mut Realm,
        locale: String,
        usage: Usage,
        sensitivity: Sensitivity,
        ignore_punctuation: bool,
        collation: String,
        numeric: bool,
        case_first: Option<CaseFirst>,
        config: Arc<CollatorConfig>,
    ) -> Res<Self> {
        Ok(Self {
            inner: RefCell::new(MutableCollator {
                object: MutObject::with_proto(
                    realm
                        .intrinsics
                        .clone_public()
                        .intl_collator
                        .get(realm)?
                        .clone(),
                ),
                config,
                locale,
                usage,
                sensitivity,
                ignore_punctuation,
                collation,
                numeric,
                case_first,
                bound_compare: None,
            }),
        })
    }
}

// https://tc39.es/ecma402/#sec-intl-collator-constructor
#[props(intrinsic_name = intl_collator, to_string_tag = "Intl.Collator")]
impl Collator {
    #[constructor]
    fn construct(
        locales: Option<String>,
        opts: Option<CollatorOptionsJs>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let opts = opts.unwrap_or_default();
        let locale_str = locales.unwrap_or_else(|| "en".to_string());
        let locale: Locale = locale_str
            .parse()
            .unwrap_or_else(|_| "en".parse().expect("en is a valid locale"));

        let usage = opts.usage.unwrap_or_default();
        let sensitivity = opts.sensitivity.unwrap_or_default();
        
        // Thai locale defaults ignorePunctuation to true
        let is_thai = locale_str.starts_with("th") && 
            (locale_str.len() == 2 || locale_str.chars().nth(2).is_some_and(|c| c == '-' || c == '_'));
        let ignore_punctuation = opts.ignore_punctuation.unwrap_or(is_thai);
        
        let numeric = opts.numeric.unwrap_or(false);
        let case_first = opts.case_first;
        let collation = opts.collation.unwrap_or_else(|| "default".to_string());

        let mut collator_opts = CollatorOptions::default();

        let (strength, case_level) = match sensitivity {
            Sensitivity::Base => (Strength::Primary, CaseLevel::Off),
            Sensitivity::Accent => (Strength::Secondary, CaseLevel::Off),
            Sensitivity::Case => (Strength::Primary, CaseLevel::On),
            Sensitivity::Variant => (Strength::Tertiary, CaseLevel::On),
        };

        collator_opts.strength = Some(strength);
        collator_opts.case_level = Some(case_level);

        if ignore_punctuation {
            collator_opts.alternate_handling = Some(AlternateHandling::Shifted);
        }

        // Build preferences with numeric and case_first options
        let mut prefs = CollatorPreferences::from(&locale);

        if numeric {
            prefs.numeric_ordering = Some(CollationNumericOrdering::True);
        }

        if let Some(ref cf) = case_first {
            prefs.case_first = Some(match cf {
                CaseFirst::Upper => CollationCaseFirst::Upper,
                CaseFirst::Lower => CollationCaseFirst::Lower,
                CaseFirst::False => CollationCaseFirst::False,
            });
        }

        // Verify the collator can be created (validates the options)
        let _ = IcuCollator::try_new(prefs, collator_opts)
            .map_err(|e| Error::ty_error(format!("Failed to create Collator: {e}")))?;

        let config = Arc::new(CollatorConfig {
            prefs,
            opts: collator_opts,
        });

        Ok(Self::create(
            realm,
            locale_str,
            usage,
            sensitivity,
            ignore_punctuation,
            collation,
            numeric,
            case_first,
            config,
        )?
        .into_object())
    }

    // https://tc39.es/ecma402/#sec-intl.collator.prototype.compare
    #[get("compare")]
    fn compare(&self, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let mut inner = self.inner.borrow_mut();

        if let Some(ref bound_compare) = inner.bound_compare {
            return Ok(bound_compare.clone());
        }

        let config = inner.config.clone();

        let compare_fn =
            NativeFunction::with_proto_and_len(
                "",  // Anonymous function - name should be empty string per spec
                move |args, _this, realm| {
                    let x = args
                        .first()
                        .map(|v| v.to_string(realm))
                        .transpose()?
                        .unwrap_or_else(|| "undefined".into());
                    let y = args
                        .get(1)
                        .map(|v| v.to_string(realm))
                        .transpose()?
                        .unwrap_or_else(|| "undefined".into());

                    // Create a collator from the stored config
                    let collator = IcuCollator::try_new(config.prefs, config.opts)
                        .map_err(|e| Error::ty_error(format!("Failed to create Collator: {e}")))?;

                    let result = match collator.compare(x.as_str(), y.as_str()) {
                        Ordering::Less => -1,
                        Ordering::Equal => 0,
                        Ordering::Greater => 1,
                    };

                    Ok(Value::Number(f64::from(result)))
                },
                realm.intrinsics.func.clone(),
                2,
                realm,
            );

        inner.bound_compare = Some(compare_fn.clone());

        Ok(compare_fn)
    }

    // https://tc39.es/ecma402/#sec-intl.collator.prototype.resolvedoptions
    #[prop("resolvedOptions")]
    fn resolved_options(&self, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let inner = self.inner.borrow();

        let options = Object::new(realm);

        options.define_property("locale".into(), inner.locale.clone().into(), realm)?;

        options.define_property("usage".into(), inner.usage.into_value(), realm)?;

        options.define_property("sensitivity".into(), inner.sensitivity.into_value(), realm)?;

        options.define_property(
            "ignorePunctuation".into(),
            inner.ignore_punctuation.into(),
            realm,
        )?;

        options.define_property("collation".into(), inner.collation.clone().into(), realm)?;

        options.define_property("numeric".into(), inner.numeric.into(), realm)?;

        if let Some(ref cf) = inner.case_first {
            options.define_property("caseFirst".into(), cf.into_value(), realm)?;
        }

        Ok(options)
    }

    // https://tc39.es/ecma402/#sec-intl.collator.supportedlocalesof
    #[prop("supportedLocalesOf")]
    fn supported_locales_of(
        locales: &Value,
        _options: Option<LocaleMatcherOptions>,
        #[realm] realm: &mut Realm,
    ) -> Res<Vec<String>> {
        let locale_list = canonicalize_locale_list(locales, realm)?;
        
        let mut supported = Vec::new();
        for locale_str in locale_list {
            if let Ok(locale) = locale_str.parse::<Locale>() {
                let prefs = CollatorPreferences::from(&locale);
                let opts = CollatorOptions::default();
                if IcuCollator::try_new(prefs, opts).is_ok() {
                    supported.push(locale.to_string());
                }
            }
        }
        Ok(supported)
    }
}

/// Canonicalize a locale list from a Value (string or array)
fn canonicalize_locale_list(locales: &Value, realm: &mut Realm) -> Res<Vec<String>> {
    if locales.is_undefined() {
        return Ok(Vec::new());
    }

    let mut result = Vec::new();

    if locales.is_string() {
        // Single string locale
        result.push(locales.to_string(realm)?.to_string());
    } else if let Value::Object(obj) = &locales {
        let length = obj
            .get("length", realm)?
            .to_number(realm)?
            as usize;

        for i in 0..length {
            let locale_value = obj.get(i, realm)?;
            if !locale_value.is_undefined() && !locale_value.is_null() {
                let locale_str = locale_value.to_string(realm)?;
                result.push(locale_str.to_string());
            }
        }
    }

    Ok(result)
}
