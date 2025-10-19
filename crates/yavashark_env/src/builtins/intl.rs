mod collator;
mod date_time_format;
mod display_names;
mod duration_format;
mod get_canonical_locales;
mod list_format;
mod locale;
mod number_format;
mod plural_rules;
mod relative_time_format;
mod segmenter;
mod supported_values_of;
mod utils;

pub use collator::*;
pub use date_time_format::*;
pub use display_names::*;
pub use duration_format::*;
pub use get_canonical_locales::*;
pub use list_format::*;
pub use locale::*;
pub use number_format::*;
pub use plural_rules::*;
pub use relative_time_format::*;
pub use segmenter::*;
pub use supported_values_of::*;

use crate::{Object, ObjectHandle, Realm, Res, Value, Variable};
use crate::realm::Intrinsic;

pub struct Protos {
    pub collator: ObjectHandle,
    pub date_time_format: ObjectHandle,
    pub display_names: ObjectHandle,
    pub duration_format: ObjectHandle,
    pub list_format: ObjectHandle,
    pub locale: ObjectHandle,
    pub number_format: ObjectHandle,
    pub plural_rules: ObjectHandle,
    pub relative_time_format: ObjectHandle,
    pub segmenter: ObjectHandle,
}

fn constr(obj: &ObjectHandle, realm: &mut Realm) -> Variable {
    Variable::write_config(
        obj.resolve_property("constructor", realm)
            .ok()
            .flatten()
            .unwrap_or(Value::Undefined),
    )
}

pub fn get_intl(
    realm: &mut Realm,
) -> Res<(ObjectHandle, Protos)> {
    let obj = Object::with_proto(realm.intrinsics.obj.clone());

    let collator = Collator::initialize(
        realm,
    )?;
    obj.define_property_attributes("Collator".into(), constr(&collator, realm), realm)?;

    let date_time_format = DateTimeFormat::initialize(
        realm,
    )?;
    obj.define_property_attributes(
        "DateTimeFormat".into(),
        constr(&date_time_format, realm),
        realm,
    )?;

    let display_names = DisplayNames::initialize(
        realm,
    )?;
    obj.define_property_attributes("DisplayNames".into(), constr(&display_names, realm), realm)?;

    let duration_format = DurationFormat::initialize(
        realm,
    )?;
    obj.define_property_attributes(
        "DurationFormat".into(),
        constr(&duration_format, realm),
        realm,
    )?;

    let list_format = ListFormat::initialize(
        realm,
    )?;
    obj.define_property_attributes("ListFormat".into(), constr(&list_format, realm), realm)?;

    let locale = Locale::initialize(
        realm,
    )?;
    obj.define_property_attributes("Locale".into(), constr(&locale, realm), realm)?;

    let number_format = NumberFormat::initialize(
        realm,
    )?;
    obj.define_property_attributes("NumberFormat".into(), constr(&number_format, realm), realm)?;

    let plural_rules = PluralRules::initialize(
        realm,
    )?;
    obj.define_property_attributes("PluralRules".into(), constr(&plural_rules, realm), realm)?;

    let relative_time_format = RelativeTimeFormat::initialize(
        realm,
    )?;
    obj.define_property_attributes(
        "RelativeTimeFormat".into(),
        constr(&relative_time_format, realm),
        realm,
    )?;

    let segmenter = Segmenter::initialize(
        realm,
    )?;
    obj.define_property_attributes("Segmenter".into(), constr(&segmenter, realm), realm)?;

    let get_canonical_locales = get_get_canonical_locales(realm);
    obj.define_property_attributes(
        "getCanonicalLocales".into(),
        Variable::write_config(get_canonical_locales.into()),
        realm,
    )?;

    let supported_values_of = get_supported_values_of(realm);
    obj.define_property_attributes(
        "supportedValuesOf".into(),
        Variable::write_config(supported_values_of.into()),
        realm,
    )?;

    Ok((
        obj,
        Protos {
            collator,
            date_time_format,
            display_names,
            duration_format,
            list_format,
            locale,
            number_format,
            plural_rules,
            relative_time_format,
            segmenter,
        },
    ))
}
