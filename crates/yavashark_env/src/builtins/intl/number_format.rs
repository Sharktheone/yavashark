use crate::array::Array;
use crate::builtins::intl::utils::{LocaleMatcher, LocaleMatcherOptions};
use crate::value::{fmt_num, IntoValue, Obj};
use crate::{Error, MutObject, NativeFunction, Object, ObjectHandle, Realm, Res, Value};
use icu::decimal::input::Decimal;
use icu::decimal::options::{DecimalFormatterOptions, GroupingStrategy};
use icu::decimal::{parts as icu_parts, DecimalFormatter};
use icu::locale::Locale;
use std::cell::RefCell;
use std::sync::Arc;
use writeable::{Part, PartsWrite, Writeable};
use yavashark_macro::{data_object, object, props};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[data_object(error = "range")]
pub enum Style {
    #[default]
    Decimal,
    Currency,
    Percent,
    Unit,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[data_object(error = "range")]
pub enum CurrencyDisplay {
    #[default]
    Symbol,
    NarrowSymbol,
    Code,
    Name,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[data_object(error = "range")]
pub enum CurrencySign {
    #[default]
    Standard,
    Accounting,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[data_object(error = "range")]
pub enum UnitDisplay {
    #[default]
    Short,
    Narrow,
    Long,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[data_object(error = "range")]
pub enum Notation {
    #[default]
    Standard,
    Scientific,
    Engineering,
    Compact,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[data_object(error = "range")]
pub enum CompactDisplay {
    #[default]
    Short,
    Long,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[data_object(error = "range")]
pub enum SignDisplay {
    #[default]
    Auto,
    Never,
    Always,
    ExceptZero,
    Negative,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[data_object(error = "range")]
pub enum RoundingMode {
    Ceil,
    Floor,
    Expand,
    Trunc,
    HalfCeil,
    HalfFloor,
    #[default]
    HalfExpand,
    HalfTrunc,
    HalfEven,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[data_object(error = "range")]
pub enum RoundingPriority {
    #[default]
    Auto,
    MorePrecision,
    LessPrecision,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[data_object(error = "range")]
pub enum TrailingZeroDisplay {
    #[default]
    Auto,
    StripIfInteger,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[data_object(error = "range")]
pub enum UseGrouping {
    #[default]
    Auto,
    Always,
    Min2,
    False,
}

#[derive(Default)]
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
    pub rounding_increment: Option<u16>,
    #[prop("trailingZeroDisplay")]
    pub trailing_zero_display: Option<TrailingZeroDisplay>,
}

#[derive(Debug, Clone)]
struct NumberFormatConfig {
    locale: Locale,
    numbering_system: Option<String>,
    style: Style,
    currency: Option<String>,
    currency_display: CurrencyDisplay,
    currency_sign: CurrencySign,
    unit: Option<String>,
    unit_display: UnitDisplay,
    minimum_integer_digits: u8,
    minimum_fraction_digits: Option<u8>,
    maximum_fraction_digits: Option<u8>,
    minimum_significant_digits: Option<u8>,
    maximum_significant_digits: Option<u8>,
    use_grouping: UseGrouping,
    notation: Notation,
    compact_display: CompactDisplay,
    sign_display: SignDisplay,
    rounding_mode: RoundingMode,
    rounding_priority: RoundingPriority,
    rounding_increment: u16,
    trailing_zero_display: TrailingZeroDisplay,
    grouping_strategy: GroupingStrategy,
}

#[object]
#[derive(Debug)]
pub struct NumberFormat {
    #[mutable]
    config: Arc<NumberFormatConfig>,
    #[mutable]
    bound_format: Option<ObjectHandle>,
}

impl NumberFormat {
    fn create(realm: &mut Realm, config: Arc<NumberFormatConfig>) -> Res<Self> {
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
                config,
                bound_format: None,
            }),
        })
    }

    fn format_number(&self, value: f64) -> String {
        let inner = self.inner.borrow();
        let config = &inner.config;

        let locale = &config.locale;
        let mut options = DecimalFormatterOptions::default();
        options.grouping_strategy = Some(config.grouping_strategy);

        let Ok(formatter) = DecimalFormatter::try_new(locale.into(), options) else {
            return value.to_string();
        };

        let mut formatted_value = value;

        if config.style == Style::Percent {
            formatted_value *= 100.0;
        }

        let decimal = value_to_decimal(formatted_value);

        let result = formatter.format(&decimal).to_string();

        match config.style {
            Style::Percent => format!("{}%", result),
            Style::Currency => {
                if let Some(ref currency) = config.currency {
                    match config.currency_display {
                        CurrencyDisplay::Code => format!("{} {}", currency, result),
                        CurrencyDisplay::Symbol | CurrencyDisplay::NarrowSymbol => {
                            let symbol = get_currency_symbol(currency);
                            format!("{}{}", symbol, result)
                        }
                        CurrencyDisplay::Name => format!("{} {}", result, currency.to_lowercase()),
                    }
                } else {
                    result
                }
            }
            Style::Unit => {
                if let Some(ref unit) = config.unit {
                    let unit_str = get_unit_str(unit, config.unit_display);
                    format!("{} {}", result, unit_str)
                } else {
                    result
                }
            }
            Style::Decimal => result,
        }
    }
}

fn value_to_decimal(value: f64) -> Decimal {
    if value.is_nan() || value.is_infinite() {
        return Decimal::from(0);
    }

    let rounded = value.round() as i64;
    Decimal::from(rounded)
}

fn get_currency_symbol(currency: &str) -> &'static str {
    match currency.to_uppercase().as_str() {
        "USD" => "$",
        "EUR" => "€",
        "GBP" => "£",
        "JPY" => "¥",
        "CNY" => "¥",
        "KRW" => "₩",
        "INR" => "₹",
        "RUB" => "₽",
        "BRL" => "R$",
        "CAD" => "CA$",
        "AUD" => "A$",
        "CHF" => "CHF",
        "SEK" => "kr",
        "NOK" => "kr",
        "DKK" => "kr",
        "PLN" => "zł",
        "TRY" => "₺",
        "MXN" => "MX$",
        _ => "$",
    }
}

fn get_unit_str(unit: &str, display: UnitDisplay) -> &'static str {
    match display {
        UnitDisplay::Short | UnitDisplay::Narrow => match unit {
            "meter" => "m",
            "kilometer" => "km",
            "centimeter" => "cm",
            "millimeter" => "mm",
            "mile" => "mi",
            "yard" => "yd",
            "foot" => "ft",
            "inch" => "in",
            "liter" => "L",
            "milliliter" => "mL",
            "gallon" => "gal",
            "kilogram" => "kg",
            "gram" => "g",
            "pound" => "lb",
            "ounce" => "oz",
            "celsius" => "°C",
            "fahrenheit" => "°F",
            "second" => "s",
            "millisecond" => "ms",
            "microsecond" => "μs",
            "nanosecond" => "ns",
            "minute" => "min",
            "hour" => "h",
            "day" => "d",
            "week" => "wk",
            "month" => "mo",
            "year" => "yr",
            "byte" => "B",
            "kilobyte" => "kB",
            "megabyte" => "MB",
            "gigabyte" => "GB",
            "terabyte" => "TB",
            "petabyte" => "PB",
            "bit" => "bit",
            "kilobit" => "kbit",
            "megabit" => "Mbit",
            "gigabit" => "Gbit",
            "terabit" => "Tbit",
            "percent" => "%",
            "degree" => "°",
            "acre" => "ac",
            "hectare" => "ha",
            _ => "unit",
        },
        UnitDisplay::Long => match unit {
            "meter" => "meters",
            "kilometer" => "kilometers",
            "centimeter" => "centimeters",
            "millimeter" => "millimeters",
            "mile" => "miles",
            "yard" => "yards",
            "foot" => "feet",
            "inch" => "inches",
            "liter" => "liters",
            "milliliter" => "milliliters",
            "gallon" => "gallons",
            "kilogram" => "kilograms",
            "gram" => "grams",
            "pound" => "pounds",
            "ounce" => "ounces",
            "celsius" => "degrees Celsius",
            "fahrenheit" => "degrees Fahrenheit",
            "second" => "seconds",
            "millisecond" => "milliseconds",
            "microsecond" => "microseconds",
            "nanosecond" => "nanoseconds",
            "minute" => "minutes",
            "hour" => "hours",
            "day" => "days",
            "week" => "weeks",
            "month" => "months",
            "year" => "years",
            "byte" => "bytes",
            "kilobyte" => "kilobytes",
            "megabyte" => "megabytes",
            "gigabyte" => "gigabytes",
            "terabyte" => "terabytes",
            "petabyte" => "petabytes",
            "bit" => "bits",
            "kilobit" => "kilobits",
            "megabit" => "megabits",
            "gigabit" => "gigabits",
            "terabit" => "terabits",
            "percent" => "percent",
            "degree" => "degrees",
            "acre" => "acres",
            "hectare" => "hectares",
            _ => "units",
        },
    }
}

fn is_well_formed_currency_code(currency: &str) -> bool {
    currency.len() == 3 && currency.chars().all(|c| c.is_ascii_alphabetic())
}

const SANCTIONED_UNITS: &[&str] = &[
    "acre",
    "bit",
    "byte",
    "celsius",
    "centimeter",
    "day",
    "degree",
    "fahrenheit",
    "fluid-ounce",
    "foot",
    "gallon",
    "gigabit",
    "gigabyte",
    "gram",
    "hectare",
    "hour",
    "inch",
    "kilobit",
    "kilobyte",
    "kilogram",
    "kilometer",
    "liter",
    "megabit",
    "megabyte",
    "meter",
    "microsecond",
    "mile",
    "mile-scandinavian",
    "milliliter",
    "millimeter",
    "millisecond",
    "minute",
    "month",
    "nanosecond",
    "ounce",
    "percent",
    "petabyte",
    "pound",
    "second",
    "stone",
    "terabit",
    "terabyte",
    "week",
    "yard",
    "year",
];

fn is_well_formed_unit_identifier(unit: &str) -> bool {
    if let Some((num, den)) = unit.split_once("-per-") {
        if den.is_empty() {
            return false;
        }
        SANCTIONED_UNITS.binary_search(&num).is_ok() && SANCTIONED_UNITS.binary_search(&den).is_ok()
    } else {
        SANCTIONED_UNITS.binary_search(&unit).is_ok()
    }
}

fn parse_use_grouping(value: &Value, default: UseGrouping, realm: &mut Realm) -> Res<UseGrouping> {
    if value.is_undefined() {
        return Ok(default);
    }

    if let Value::Boolean(true) = value {
        return Ok(UseGrouping::Always);
    }

    if !value.is_truthy() {
        return Ok(UseGrouping::False);
    }

    let s = value.to_string(realm)?;
    match s.as_str() {
        "min2" => Ok(UseGrouping::Min2),
        "auto" => Ok(UseGrouping::Auto),
        "always" => Ok(UseGrouping::Always),
        "true" | "false" => Ok(default),
        _ => Err(Error::range_error(
            "expected one of `min2`, `auto`, `always`, `true`, or `false`".to_string(),
        )),
    }
}

/// A part of a formatted number with type and value
#[derive(Debug, Clone)]
struct NumberPart {
    part_type: &'static str,
    value: String,
}

/// Writer that collects parts from ICU's write_to_parts
struct PartsWriter {
    string: String,
    parts: Vec<(usize, usize, Part)>,
}

impl PartsWriter {
    fn new() -> Self {
        Self {
            string: String::new(),
            parts: Vec::new(),
        }
    }

    fn finish(mut self) -> (String, Vec<(usize, usize, Part)>) {
        // Sort by first open and last closed
        self.parts
            .sort_unstable_by_key(|(begin, end, _)| (*begin, end.wrapping_neg()));
        (self.string, self.parts)
    }
}

impl std::fmt::Write for PartsWriter {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.string.write_str(s)
    }
    fn write_char(&mut self, c: char) -> std::fmt::Result {
        self.string.write_char(c)
    }
}

impl PartsWrite for PartsWriter {
    type SubPartsWrite = Self;
    fn with_part(
        &mut self,
        part: Part,
        mut f: impl FnMut(&mut Self::SubPartsWrite) -> std::fmt::Result,
    ) -> std::fmt::Result {
        let start = self.string.len();
        f(self)?;
        let end = self.string.len();
        if start < end {
            self.parts.push((start, end, part));
        }
        Ok(())
    }
}

/// Convert ICU part type to JavaScript NumberFormat part type
fn icu_part_to_js_type(part: &Part) -> &'static str {
    if *part == icu_parts::INTEGER {
        "integer"
    } else if *part == icu_parts::FRACTION {
        "fraction"
    } else if *part == icu_parts::DECIMAL {
        "decimal"
    } else if *part == icu_parts::GROUP {
        "group"
    } else if *part == icu_parts::MINUS_SIGN {
        "minusSign"
    } else if *part == icu_parts::PLUS_SIGN {
        "plusSign"
    } else {
        "unknown"
    }
}

/// Extract parts from a FormattedDecimal using ICU's write_to_parts
fn extract_decimal_parts(formatted: &impl Writeable) -> Vec<NumberPart> {
    let mut writer = PartsWriter::new();
    let _ = formatted.write_to_parts(&mut writer);
    let (string, raw_parts) = writer.finish();

    // We need to convert the nested/overlapping parts into flat, non-overlapping parts
    // ICU gives us parts like (0, 8, INTEGER) which contains (4, 5, GROUP)
    // We need to split into: [integer: 0-4, group: 4-5, integer: 5-8]

    let mut result = Vec::new();

    if raw_parts.is_empty() {
        // No parts, just return the whole string as integer
        if !string.is_empty() {
            result.push(NumberPart {
                part_type: "integer",
                value: string,
            });
        }
        return result;
    }

    // Find the "leaf" parts (parts that don't contain other parts)
    // and split the parent parts around them
    let mut events: Vec<(usize, i8, &Part)> = Vec::new(); // (pos, type: 0=end, 1=start, part)

    for (start, end, part) in &raw_parts {
        events.push((*start, 1, part));
        events.push((*end, 0, part));
    }

    // Sort by position, then by type (ends before starts at same position)
    events.sort_by_key(|(pos, typ, _)| (*pos, *typ));

    // Track active parts using a stack
    let mut active_parts: Vec<&Part> = Vec::new();
    let mut last_pos = 0;

    for (pos, event_type, part) in events {
        if pos > last_pos && !active_parts.is_empty() {
            // Emit a part for the range [last_pos, pos) using the innermost active part
            let innermost = active_parts.last().unwrap_or(&&Part::ERROR);
            let value = string[last_pos..pos].to_string();
            if !value.is_empty() {
                result.push(NumberPart {
                    part_type: icu_part_to_js_type(innermost),
                    value,
                });
            }
        }

        if event_type == 1 {
            // Start event - push to stack
            active_parts.push(part);
        } else {
            // End event - pop from stack
            active_parts.pop();
        }

        last_pos = pos;
    }

    // Handle any remaining text after the last part
    if last_pos < string.len() {
        result.push(NumberPart {
            part_type: "literal",
            value: string[last_pos..].to_string(),
        });
    }

    result
}

// https://tc39.es/ecma402/#sec-intl-numberformat-constructor
#[props(intrinsic_name = intl_number_format, to_string_tag = "Intl.NumberFormat")]
impl NumberFormat {
    #[constructor]
    fn construct(
        locales: Option<String>,
        options: Option<NumberFormatOptions>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let opts = options.unwrap_or_default();
        let locale_str = locales.unwrap_or_else(|| "en".to_string());
        let locale: Locale = locale_str
            .parse()
            .unwrap_or_else(|_| "en".parse().expect("en is a valid locale"));

        let style = opts.style.unwrap_or_default();

        let currency = opts.currency.as_ref();
        if style == Style::Currency && currency.is_none() {
            return Err(Error::ty_error(
                "currency must be provided when style is 'currency'".to_string(),
            ));
        }
        if let Some(c) = currency {
            if !is_well_formed_currency_code(c) {
                return Err(Error::range_error(format!("Invalid currency code: {}", c)));
            }
        }

        let unit = opts.unit.as_ref();
        if style == Style::Unit && unit.is_none() {
            return Err(Error::ty_error(
                "unit must be provided when style is 'unit'".to_string(),
            ));
        }
        if let Some(u) = unit {
            if !is_well_formed_unit_identifier(u) {
                return Err(Error::range_error(format!("Invalid unit: {}", u)));
            }
        }

        let notation = opts.notation.unwrap_or_default();

        let (mnfd_default, mxfd_default) =
            if style == Style::Currency && notation == Notation::Standard {
                (2u8, 2u8)
            } else if style == Style::Percent {
                (0u8, 0u8)
            } else {
                (0u8, 3u8)
            };

        let minimum_integer_digits = opts.minimum_integer_digits.unwrap_or(1);
        if !(1..=21).contains(&minimum_integer_digits) {
            return Err(Error::range_error(
                "minimumIntegerDigits must be between 1 and 21".to_string(),
            ));
        }

        let has_sd =
            opts.minimum_significant_digits.is_some() || opts.maximum_significant_digits.is_some();
        let has_fd =
            opts.minimum_fraction_digits.is_some() || opts.maximum_fraction_digits.is_some();

        let rounding_priority = opts.rounding_priority.unwrap_or_default();

        let (minimum_fraction_digits, maximum_fraction_digits) = if has_fd {
            let min = opts.minimum_fraction_digits;
            let max = opts.maximum_fraction_digits;

            let min_val = min.unwrap_or(mnfd_default);
            let max_val = max.unwrap_or_else(|| u8::max(mxfd_default, min_val));

            if min_val > max_val {
                return Err(Error::range_error(
                    "minimumFractionDigits cannot be greater than maximumFractionDigits"
                        .to_string(),
                ));
            }
            (Some(min_val), Some(max_val))
        } else if !has_sd || rounding_priority != RoundingPriority::Auto {
            (Some(mnfd_default), Some(mxfd_default))
        } else {
            (None, None)
        };

        let (minimum_significant_digits, maximum_significant_digits) = if has_sd {
            let min = opts.minimum_significant_digits.unwrap_or(1);
            let max = opts.maximum_significant_digits.unwrap_or(21);
            if !(1..=21).contains(&min) || !(1..=21).contains(&max) || min > max {
                return Err(Error::range_error(
                    "significantDigits must be between 1 and 21, and min <= max".to_string(),
                ));
            }
            (Some(min), Some(max))
        } else if rounding_priority != RoundingPriority::Auto {
            (Some(1), Some(21))
        } else {
            (None, None)
        };

        let default_use_grouping = if notation == Notation::Compact {
            UseGrouping::Min2
        } else {
            UseGrouping::Auto
        };

        let use_grouping_value = opts.use_grouping.unwrap_or(Value::Undefined);
        let use_grouping = parse_use_grouping(&use_grouping_value, default_use_grouping, realm)?;

        let grouping_strategy = match use_grouping {
            UseGrouping::Auto => GroupingStrategy::Auto,
            UseGrouping::Always => GroupingStrategy::Always,
            UseGrouping::Min2 => GroupingStrategy::Min2,
            UseGrouping::False => GroupingStrategy::Never,
        };

        let rounding_increment = opts.rounding_increment.unwrap_or(1);
        let valid_increments = [
            1, 2, 5, 10, 20, 25, 50, 100, 200, 250, 500, 1000, 2000, 2500, 5000,
        ];
        if !valid_increments.contains(&rounding_increment) {
            return Err(Error::range_error(format!(
                "Invalid rounding increment: {}",
                rounding_increment
            )));
        }

        let config = Arc::new(NumberFormatConfig {
            locale,
            numbering_system: opts.numbering_system.clone(),
            style,
            currency: opts.currency.map(|c| c.to_uppercase()),
            currency_display: opts.currency_display.unwrap_or_default(),
            currency_sign: opts.currency_sign.unwrap_or_default(),
            unit: opts.unit.clone(),
            unit_display: opts.unit_display.unwrap_or_default(),
            minimum_integer_digits,
            minimum_fraction_digits,
            maximum_fraction_digits,
            minimum_significant_digits,
            maximum_significant_digits,
            use_grouping,
            notation,
            compact_display: opts.compact_display.unwrap_or_default(),
            sign_display: opts.sign_display.unwrap_or_default(),
            rounding_mode: opts.rounding_mode.unwrap_or_default(),
            rounding_priority,
            rounding_increment,
            trailing_zero_display: opts.trailing_zero_display.unwrap_or_default(),
            grouping_strategy,
        });

        Ok(Self::create(realm, config)?.into_object())
    }

    // https://tc39.es/ecma402/#sec-intl.numberformat.prototype.format
    #[get("format")]
    fn format(&self, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let mut inner = self.inner.borrow_mut();

        if let Some(ref bound_format) = inner.bound_format {
            return Ok(bound_format.clone());
        }

        let config = Arc::clone(&inner.config);

        let format_fn = NativeFunction::with_proto_and_len(
            "",
            move |args, _this, realm| {
                let value = if let Some(v) = args.first() {
                    v.to_number(realm)?
                } else {
                    f64::NAN
                };

                if value.is_nan() {
                    return Ok(Value::from("NaN"));
                }
                if value.is_infinite() {
                    return Ok(Value::from(if value.is_sign_negative() {
                        "-∞"
                    } else {
                        "∞"
                    }));
                }

                let locale = &config.locale;
                let mut options = DecimalFormatterOptions::default();
                options.grouping_strategy = Some(config.grouping_strategy);

                let Ok(formatter) = DecimalFormatter::try_new(locale.into(), options) else {
                    return Ok(fmt_num(value).into());
                };

                let mut formatted_value = value;
                if config.style == Style::Percent {
                    formatted_value *= 100.0;
                }

                let decimal = value_to_decimal(formatted_value);

                let result = formatter.format(&decimal).to_string();

                let final_result = match config.style {
                    Style::Percent => format!("{result}%"),
                    Style::Currency => {
                        if let Some(ref currency) = config.currency {
                            match config.currency_display {
                                CurrencyDisplay::Code => format!("{currency} {result}"),
                                CurrencyDisplay::Symbol | CurrencyDisplay::NarrowSymbol => {
                                    let symbol = get_currency_symbol(currency);
                                    format!("{symbol}{result}")
                                }
                                CurrencyDisplay::Name => {
                                    format!("{} {}", result, currency.to_lowercase())
                                }
                            }
                        } else {
                            result
                        }
                    }
                    Style::Unit => {
                        if let Some(ref unit) = config.unit {
                            let unit_str = get_unit_str(unit, config.unit_display);
                            format!("{result} {unit_str}")
                        } else {
                            result
                        }
                    }
                    Style::Decimal => result,
                };

                Ok(Value::from(final_result))
            },
            realm.intrinsics.func.clone(),
            1,
            realm,
        );

        inner.bound_format = Some(format_fn.clone());
        Ok(format_fn)
    }

    // https://tc39.es/ecma402/#sec-intl.numberformat.prototype.resolvedoptions
    #[prop("resolvedOptions")]
    fn resolved_options(&self, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let inner = self.inner.borrow();
        let config = &inner.config;

        let options = Object::new(realm);

        options.define_property("locale".into(), config.locale.to_string().into(), realm)?;

        if let Some(ref ns) = config.numbering_system {
            options.define_property("numberingSystem".into(), ns.clone().into(), realm)?;
        }

        options.define_property("style".into(), config.style.into_value(), realm)?;

        if config.style == Style::Currency {
            if let Some(ref currency) = config.currency {
                options.define_property("currency".into(), currency.clone().into(), realm)?;
            }
            options.define_property(
                "currencyDisplay".into(),
                config.currency_display.into_value(),
                realm,
            )?;

            options.define_property(
                "currencySign".into(),
                config.currency_sign.into_value(),
                realm,
            )?;
        }

        if config.style == Style::Unit {
            if let Some(ref unit) = config.unit {
                options.define_property("unit".into(), unit.clone().into(), realm)?;
            }
            options.define_property(
                "unitDisplay".into(),
                config.unit_display.into_value(),
                realm,
            )?;
        }

        options.define_property(
            "minimumIntegerDigits".into(),
            config.minimum_integer_digits.into(),
            realm,
        )?;

        if let Some(min) = config.minimum_fraction_digits {
            options.define_property("minimumFractionDigits".into(), min.into(), realm)?;
        }
        if let Some(max) = config.maximum_fraction_digits {
            options.define_property("maximumFractionDigits".into(), max.into(), realm)?;
        }
        if let Some(min) = config.minimum_significant_digits {
            options.define_property(
                "minimumSignificantDigits".into(),
                (min as f64).into(),
                realm,
            )?;
        }
        if let Some(max) = config.maximum_significant_digits {
            options.define_property("maximumSignificantDigits".into(), max.into(), realm)?;
        }

        options.define_property(
            "useGrouping".into(),
            config.use_grouping.into_value(),
            realm,
        )?;

        options.define_property("notation".into(), config.notation.into_value(), realm)?;

        if config.notation == Notation::Compact {
            options.define_property(
                "compactDisplay".into(),
                config.compact_display.into_value(),
                realm,
            )?;
        }

        options.define_property(
            "signDisplay".into(),
            config.sign_display.into_value(),
            realm,
        )?;

        options.define_property(
            "roundingIncrement".into(),
            config.rounding_increment.into(),
            realm,
        )?;

        options.define_property(
            "roundingMode".into(),
            config.rounding_mode.into_value(),
            realm,
        )?;

        options.define_property(
            "roundingPriority".into(),
            config.rounding_priority.into_value(),
            realm,
        )?;

        options.define_property(
            "trailingZeroDisplay".into(),
            config.trailing_zero_display.into_value(),
            realm,
        )?;

        Ok(options)
    }

    // https://tc39.es/ecma402/#sec-intl.numberformat.supportedlocalesof
    #[prop("supportedLocalesOf")]
    pub fn supported_locales_of(
        locales: &Value,
        _options: Option<LocaleMatcherOptions>,
        #[realm] realm: &mut Realm,
    ) -> Res<Vec<String>> {
        let locale_list = canonicalize_locale_list(locales, realm)?;

        let mut supported = Vec::new();
        for locale_str in locale_list {
            if let Ok(locale) = locale_str.parse::<Locale>() {
                let options = DecimalFormatterOptions::default();
                if DecimalFormatter::try_new((&locale).into(), options).is_ok() {
                    supported.push(locale.to_string());
                }
            }
        }
        Ok(supported)
    }

    // https://tc39.es/ecma402/#sec-intl.numberformat.prototype.formattoparts
    #[prop("formatToParts")]
    #[length(1)]
    fn format_to_parts(
        &self,
        value: Option<Value>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let val = match value {
            Some(v) if !v.is_undefined() => v.to_number(realm)?,
            _ => f64::NAN,
        };
        let parts_array = Array::new(realm.intrinsics.clone_public().array.get(realm)?.clone());

        // Handle special values
        if val.is_nan() {
            let part = Object::new(realm);
            part.define_property("type".into(), "nan".into(), realm)?;
            part.define_property("value".into(), "NaN".into(), realm)?;
            parts_array.push(part.into())?;
            return Ok(parts_array.into_object());
        }

        if val.is_infinite() {
            // Handle sign for infinity
            if val.is_sign_negative() {
                let sign_part = Object::new(realm);
                sign_part.define_property("type".into(), "minusSign".into(), realm)?;
                sign_part.define_property("value".into(), "-".into(), realm)?;
                parts_array.push(sign_part.into())?;
            }
            let inf_part = Object::new(realm);
            inf_part.define_property("type".into(), "infinity".into(), realm)?;
            inf_part.define_property("value".into(), "∞".into(), realm)?;
            parts_array.push(inf_part.into())?;
            return Ok(parts_array.into_object());
        }

        let inner = self.inner.borrow();
        let config = &inner.config;

        // Apply percent transformation
        let formatted_value = if config.style == Style::Percent {
            val * 100.0
        } else {
            val
        };

        // Create decimal and formatter
        let decimal = value_to_decimal(formatted_value);
        let mut options = DecimalFormatterOptions::default();
        options.grouping_strategy = Some(config.grouping_strategy);

        let Ok(formatter) = DecimalFormatter::try_new((&config.locale).into(), options) else {
            // Fallback: return entire formatted string as integer
            let part = Object::new(realm);
            part.define_property("type".into(), "integer".into(), realm)?;
            part.define_property("value".into(), fmt_num(val).into(), realm)?;
            parts_array.push(part.into())?;
            return Ok(parts_array.into_object());
        };

        // Format and extract parts
        let formatted = formatter.format(&decimal);
        let number_parts = extract_decimal_parts(&formatted);

        // Handle currency prefix first (before number parts)
        if config.style == Style::Currency {
            if let Some(ref currency) = config.currency {
                let currency_part = Object::new(realm);
                currency_part.define_property("type".into(), "currency".into(), realm)?;

                match config.currency_display {
                    CurrencyDisplay::Code => {
                        currency_part.define_property(
                            "value".into(),
                            currency.clone().into(),
                            realm,
                        )?;
                        parts_array.push(currency_part.into())?;

                        // Add space literal after currency code
                        let space_part = Object::new(realm);
                        space_part.define_property("type".into(), "literal".into(), realm)?;
                        space_part.define_property("value".into(), " ".into(), realm)?;
                        parts_array.push(space_part.into())?;
                    }
                    CurrencyDisplay::Symbol | CurrencyDisplay::NarrowSymbol => {
                        currency_part.define_property(
                            "value".into(),
                            get_currency_symbol(currency).into(),
                            realm,
                        )?;
                        parts_array.push(currency_part.into())?;
                    }
                    CurrencyDisplay::Name => {
                        // For name display, currency comes after the number
                        // We'll handle it in the suffix section
                    }
                }
            }
        }

        // Add number parts
        for np in number_parts {
            let part = Object::new(realm);
            part.define_property("type".into(), np.part_type.into(), realm)?;
            part.define_property("value".into(), np.value.into(), realm)?;
            parts_array.push(part.into())?;
        }

        // Add style-specific suffix parts
        match config.style {
            Style::Percent => {
                let part = Object::new(realm);
                part.define_property("type".into(), "percentSign".into(), realm)?;
                part.define_property("value".into(), "%".into(), realm)?;
                parts_array.push(part.into())?;
            }
            Style::Currency => {
                // Handle CurrencyDisplay::Name (currency name comes after number)
                if let Some(ref currency) = config.currency {
                    if config.currency_display == CurrencyDisplay::Name {
                        // Add space literal before currency name
                        let space_part = Object::new(realm);
                        space_part.define_property("type".into(), "literal".into(), realm)?;
                        space_part.define_property("value".into(), " ".into(), realm)?;
                        parts_array.push(space_part.into())?;

                        let currency_part = Object::new(realm);
                        currency_part.define_property("type".into(), "currency".into(), realm)?;
                        currency_part.define_property(
                            "value".into(),
                            currency.to_lowercase().into(),
                            realm,
                        )?;
                        parts_array.push(currency_part.into())?;
                    }
                }
            }
            Style::Unit => {
                if let Some(ref unit) = config.unit {
                    // Add space literal
                    let space_part = Object::new(realm);
                    space_part.define_property("type".into(), "literal".into(), realm)?;
                    space_part.define_property("value".into(), " ".into(), realm)?;
                    parts_array.push(space_part.into())?;

                    // Add unit
                    let unit_part = Object::new(realm);
                    unit_part.define_property("type".into(), "unit".into(), realm)?;
                    unit_part.define_property(
                        "value".into(),
                        get_unit_str(unit, config.unit_display).into(),
                        realm,
                    )?;
                    parts_array.push(unit_part.into())?;
                }
            }
            Style::Decimal => {}
        }

        Ok(parts_array.into_object())
    }

    // https://tc39.es/ecma402/#sec-intl.numberformat.prototype.formatrange
    #[prop("formatRange")]
    fn format_range(&self, start: f64, end: f64) -> Res<String> {
        if start.is_nan() || end.is_nan() {
            return Err(Error::range_error("Invalid number range".to_string()));
        }

        let start_str = self.format_number(start);
        let end_str = self.format_number(end);

        Ok(format!("{} – {}", start_str, end_str))
    }

    // https://tc39.es/ecma402/#sec-intl.numberformat.prototype.formatrangetoparts
    #[prop("formatRangeToParts")]
    fn format_range_to_parts(
        &self,
        start: f64,
        end: f64,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        if start.is_nan() || end.is_nan() {
            return Err(Error::range_error("Invalid number range".to_string()));
        }

        let start_str = self.format_number(start);
        let end_str = self.format_number(end);

        let parts = Array::new(realm.intrinsics.clone_public().array.get(realm)?.clone());

        let start_part = Object::new(realm);
        start_part.define_property("type".into(), "integer".into(), realm)?;
        start_part.define_property("value".into(), start_str.into(), realm)?;
        start_part.define_property("source".into(), "startRange".into(), realm)?;
        parts.push(start_part.into())?;

        let separator_part = Object::new(realm);
        separator_part.define_property("type".into(), "literal".into(), realm)?;
        separator_part.define_property("value".into(), " – ".into(), realm)?;
        separator_part.define_property("source".into(), "shared".into(), realm)?;
        parts.push(separator_part.into())?;

        let end_part = Object::new(realm);
        end_part.define_property("type".into(), "integer".into(), realm)?;
        end_part.define_property("value".into(), end_str.into(), realm)?;
        end_part.define_property("source".into(), "endRange".into(), realm)?;
        parts.push(end_part.into())?;

        Ok(parts.into_object())
    }
}

fn canonicalize_locale_list(locales: &Value, realm: &mut Realm) -> Res<Vec<String>> {
    if locales.is_undefined() {
        return Ok(Vec::new());
    }

    let mut result = Vec::new();

    if locales.is_string() {
        result.push(locales.to_string(realm)?.to_string());
    } else if let Value::Object(obj) = &locales {
        let length = obj.get("length", realm)?.to_number(realm)? as usize;

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
