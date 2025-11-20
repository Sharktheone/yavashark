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

use crate::partial_init::Initializer;
use crate::{Object, ObjectHandle, Realm, Res, Value, Variable};

pub struct Intl;

impl Initializer<ObjectHandle> for Intl {
    fn initialize(realm: &mut Realm) -> Res<ObjectHandle> {
        get_intl(realm)
    }
}

fn constr(obj: &ObjectHandle, realm: &mut Realm) -> Variable {
    Variable::write_config(
        obj.resolve_property("constructor", realm)
            .ok()
            .flatten()
            .unwrap_or(Value::Undefined),
    )
}

pub fn get_intl(realm: &mut Realm) -> Res<ObjectHandle> {
    let obj = Object::with_proto(realm.intrinsics.obj.clone());

    let intrinsics = realm.intrinsics.clone_public();

    obj.define_property_attributes(
        "Collator".into(),
        constr(intrinsics.intl_collator.get(realm)?, realm),
        realm,
    )?;

    obj.define_property_attributes(
        "DateTimeFormat".into(),
        constr(intrinsics.intl_date_time_format.get(realm)?, realm),
        realm,
    )?;

    obj.define_property_attributes(
        "DisplayNames".into(),
        constr(intrinsics.intl_display_names.get(realm)?, realm),
        realm,
    )?;

    obj.define_property_attributes(
        "DurationFormat".into(),
        constr(intrinsics.intl_duration_format.get(realm)?, realm),
        realm,
    )?;

    obj.define_property_attributes(
        "ListFormat".into(),
        constr(intrinsics.intl_list_format.get(realm)?, realm),
        realm,
    )?;

    obj.define_property_attributes(
        "Locale".into(),
        constr(intrinsics.intl_locale.get(realm)?, realm),
        realm,
    )?;

    obj.define_property_attributes(
        "NumberFormat".into(),
        constr(intrinsics.intl_number_format.get(realm)?, realm),
        realm,
    )?;

    obj.define_property_attributes(
        "PluralRules".into(),
        constr(intrinsics.intl_plural_rules.get(realm)?, realm),
        realm,
    )?;

    obj.define_property_attributes(
        "RelativeTimeFormat".into(),
        constr(intrinsics.intl_relative_time_format.get(realm)?, realm),
        realm,
    )?;

    obj.define_property_attributes(
        "Segmenter".into(),
        constr(intrinsics.intl_segmenter.get(realm)?, realm),
        realm,
    )?;

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

    Ok(obj)
}
