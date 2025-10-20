use crate::array::Array;
use crate::builtins::intl::utils::{
    canonicalize_locale_list, get_option_string, validate_currency_code,
};
use crate::conversion::downcast_obj;
use crate::value::Obj;
use crate::{Error, MutObject, NativeFunction, Object, ObjectHandle, Realm, Res, Value, Variable};
use std::cell::RefCell;
use yavashark_macro::{object, props};

const DEFAULT_LOCALE: &str = "und";
const STYLE_DECIMAL: &str = "decimal";
const STYLE_CURRENCY: &str = "currency";
const STYLE_PERCENT: &str = "percent";

#[object]
#[derive(Debug)]
pub struct NumberFormat {
    #[mutable]
    initialized: bool,
    #[mutable]
    locale: String,
    #[mutable]
    style: String,
    #[mutable]
    currency: Option<String>,
    #[mutable]
    use_grouping: String,
    #[mutable]
    minimum_fraction_digits: i32,
    #[mutable]
    maximum_fraction_digits: i32,
    #[mutable]
    bound_format: Option<ObjectHandle>,
}

impl NumberFormat {
    pub fn new(realm: &mut Realm) -> Res<Self> {
        Ok(Self {
            inner: RefCell::new(MutableNumberFormat {
                object: MutObject::with_proto(
                    realm
                        .intrinsics
                        .clone_public()
                        .intl_number_format
                        .get(realm)?
                        .clone(),
                ),
                initialized: false,
                locale: DEFAULT_LOCALE.to_string(),
                style: STYLE_DECIMAL.to_string(),
                currency: None,
                use_grouping: "auto".to_string(),
                minimum_fraction_digits: 0,
                maximum_fraction_digits: 3,
                bound_format: None,
            }),
        })
    }

    fn ensure_initialized(&self) -> Res<()> {
        if self.inner.borrow().initialized {
            Ok(())
        } else {
            Err(Error::ty("Intl.NumberFormat object is not initialized"))
        }
    }
}

#[props(intrinsic_name = intl_number_format, to_string_tag = "Intl.NumberFormat")]
impl NumberFormat {
    #[constructor]
    fn construct(
        locales: Option<Value>,
        options: Option<Value>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let locales_list = canonicalize_locale_list(locales, realm)?;
        let locale = locales_list
            .get(0)
            .cloned()
            .unwrap_or_else(|| DEFAULT_LOCALE.to_string());

        let options_value = options.unwrap_or(Value::Undefined);
        let options_obj = if options_value.is_undefined() || options_value.is_nullish() {
            Object::with_proto(realm.intrinsics.obj.clone())
        } else {
            crate::utils::coerce_object(options_value, realm)?
        };

        let style = get_option_string(
            &options_obj,
            "style",
            &[STYLE_DECIMAL, STYLE_CURRENCY, STYLE_PERCENT],
            realm,
        )?
        .unwrap_or_else(|| STYLE_DECIMAL.to_string());

        let currency = if style == STYLE_CURRENCY {
            let c = options_obj.get("currency", realm)?;
            if c.is_undefined() {
                return Err(Error::ty(
                    "currency option is required for style 'currency'",
                ));
            }
            let code = c.to_string(realm)?.to_string();
            validate_currency_code(&code).map_err(|e| e)?;
            Some(code)
        } else {
            None
        };

        let use_grouping = get_option_string(&options_obj, "useGrouping", &[], realm)?
            .unwrap_or_else(|| "auto".to_string());

        let mnfd = get_option_string(&options_obj, "minimumFractionDigits", &[], realm);
        let minimum_fraction_digits = if let Ok(Some(s)) = mnfd {
            s.parse::<i32>().unwrap_or(0)
        } else {
            0
        };

        let mxffd = get_option_string(&options_obj, "maximumFractionDigits", &[], realm);
        let maximum_fraction_digits = if let Ok(Some(s)) = mxffd {
            s.parse::<i32>().unwrap_or(3)
        } else {
            3
        };

        let nf = Self::new(realm)?;

        {
            let mut inner = nf.inner.borrow_mut();
            inner.locale = locale;
            inner.style = style;
            inner.currency = currency;
            inner.use_grouping = use_grouping;
            inner.minimum_fraction_digits = minimum_fraction_digits;
            inner.maximum_fraction_digits = maximum_fraction_digits;
            inner.initialized = true;
            inner.bound_format = None;
        }

        Ok(nf.into_object())
    }

    #[get("format")]
    fn format(&self, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        self.ensure_initialized()?;

        let mut inner = self.inner.borrow_mut();
        if let Some(existing) = &inner.bound_format {
            return Ok(existing.clone());
        }

        let func = NativeFunction::with_proto_and_len(
            "format",
            |args, this, realm| {
                let nf = downcast_obj::<Self>(this.copy())?;
                nf.ensure_initialized()?;

                let value = args.get(0).map_or(Value::Undefined, Value::copy);
                let s = if value.is_undefined() {
                    "NaN".to_string()
                } else {
                    value.to_string(realm)?.to_string()
                };

                Ok(Value::String(s.into()))
            },
            realm.intrinsics.func.clone(),
            1,
            realm,
        );

        inner.bound_format = Some(func.clone());
        Ok(func)
    }

    #[prop("resolvedOptions")]
    fn resolved_options(&self, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        self.ensure_initialized()?;

        let (locale, style, currency, use_grouping, mnfd, mxfd) = {
            let inner = self.inner.borrow();
            (
                inner.locale.clone(),
                inner.style.clone(),
                inner.currency.clone(),
                inner.use_grouping.clone(),
                inner.minimum_fraction_digits,
                inner.maximum_fraction_digits,
            )
        };

        let result = Object::with_proto(realm.intrinsics.obj.clone());

        result.define_property_attributes(
            "locale".into(),
            Variable::write_config(locale.into()),
            realm,
        )?;
        result.define_property_attributes(
            "style".into(),
            Variable::write_config(style.into()),
            realm,
        )?;
        result.define_property_attributes(
            "useGrouping".into(),
            Variable::write_config(use_grouping.into()),
            realm,
        )?;
        result.define_property_attributes(
            "minimumFractionDigits".into(),
            Variable::write_config((mnfd as i32).into()),
            realm,
        )?;
        result.define_property_attributes(
            "maximumFractionDigits".into(),
            Variable::write_config((mxfd as i32).into()),
            realm,
        )?;

        if let Some(cur) = currency {
            result.define_property_attributes(
                "currency".into(),
                Variable::write_config(cur.into()),
                realm,
            )?;
        }

        Ok(result)
    }

    pub fn supported_locales_of(
        locales: Value,
        _options: Option<ObjectHandle>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let list = canonicalize_locale_list(Some(locales), realm)?;
        let values = list
            .into_iter()
            .map(|s| Value::String(s.into()))
            .collect::<Vec<_>>();
        let arr = Array::with_elements(realm, values)?;
        Ok(arr.into_object())
    }
}
