use crate::array::Array;
use crate::value::Obj;
use crate::{MutObject, Object, ObjectHandle, Realm, Res, Value};
use std::cell::RefCell;
use yavashark_macro::{data_object, object, props};

#[data_object]
pub enum LocaleMatcher {
    Lookup,
    #[name("best fit")]
    BestFit,
}

#[data_object]
pub enum Style {
    Decimal,
    Currency,
    Percent,
    Unit,
}

#[data_object]
pub enum CurrencyDisplay {
    Symbol,
    NarrowSymbol,
    Code,
    Name,
}

#[data_object]
pub enum CurrencySign {
    Standard,
    Accounting,
}

#[data_object]
pub enum UnitDisplay {
    Short,
    Narrow,
    Long,
}

#[data_object]
pub enum Notation {
    Standard,
    Scientific,
    Engineering,
    Compact,
}

#[data_object]
pub enum CompactDisplay {
    Short,
    Long,
}

#[data_object]
pub enum SignDisplay {
    Auto,
    Never,
    Always,
    ExceptZero,
    Negative,
}

#[data_object]
pub enum RoundingMode {
    Ceil,
    Floor,
    Expand,
    Trunc,
    HalfCeil,
    HalfFloor,
    HalfExpand,
    HalfTrunc,
    HalfEven,
}

#[data_object]
pub enum RoundingPriority {
    Auto,
    MorePrecision,
    LessPrecision,
}

#[data_object]
pub enum TrailingZeroDisplay {
    Auto,
    StripIfInteger,
}

#[data_object]
pub struct NumberFormatOptions {
    #[prop("localeMatcher")]
    pub locale_matcher: Option<LocaleMatcher>,
    #[prop("numberingSystem")]
    pub numbering_system: Option<String>,
    pub style: Option<Style>,
    pub currency: Option<String>,
    #[prop("currencyDisplay")]
    pub currency_display: Option<CurrencyDisplay>,
    #[prop("currencySign")]
    pub currency_sign: Option<CurrencySign>,
    pub unit: Option<String>,
    #[prop("unitDisplay")]
    pub unit_display: Option<UnitDisplay>,
    #[prop("minimumIntegerDigits")]
    pub minimum_integer_digits: Option<u8>,
    #[prop("minimumFractionDigits")]
    pub minimum_fraction_digits: Option<u8>,
    #[prop("maximumFractionDigits")]
    pub maximum_fraction_digits: Option<u8>,
    #[prop("minimumSignificantDigits")]
    pub minimum_significant_digits: Option<u8>,
    #[prop("maximumSignificantDigits")]
    pub maximum_significant_digits: Option<u8>,
    #[prop("useGrouping")]
    pub use_grouping: Option<Value>,
    pub notation: Option<Notation>,
    #[prop("compactDisplay")]
    pub compact_display: Option<CompactDisplay>,
    #[prop("signDisplay")]
    pub sign_display: Option<SignDisplay>,
    #[prop("roundingMode")]
    pub rounding_mode: Option<RoundingMode>,
    #[prop("roundingPriority")]
    pub rounding_priority: Option<RoundingPriority>,
    #[prop("roundingIncrement")]
    pub rounding_increment: Option<u64>,
    #[prop("trailingZeroDisplay")]
    pub trailing_zero_display: Option<TrailingZeroDisplay>,
}

#[object]
#[derive(Debug)]
pub struct NumberFormat {}

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
            }),
        })
    }
}

#[props(intrinsic_name = intl_number_format, to_string_tag = "Intl.NumberFormat")]
impl NumberFormat {
    #[constructor]
    fn construct(
        _locales: Option<String>,
        _options: Option<NumberFormatOptions>,
        #[realm] realm: &mut Realm,
    ) -> Res<Self> {
        Self::new(realm)
    }

    fn format(&self) -> Res<String> {
        Ok(String::new())
    }

    #[prop("resolvedOptions")]
    fn resolved_options(&self, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        Ok(Object::new(realm))
    }

    pub fn supported_locales_of(_locales: Value, _options: Option<ObjectHandle>) -> Vec<String> {
        Vec::new()
    }
}
