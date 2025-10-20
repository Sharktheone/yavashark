mod duration;
mod instant;
mod now;
mod plain_date;
mod plain_date_time;
mod plain_month_day;
mod plain_time;
mod plain_year_month;
mod utils;
mod zoned_date_time;

pub use crate::builtins::temporal::duration::*;
pub use crate::builtins::temporal::instant::*;
pub use crate::builtins::temporal::now::*;
pub use crate::builtins::temporal::plain_date::*;
pub use crate::builtins::temporal::plain_date_time::*;
pub use crate::builtins::temporal::plain_month_day::*;
pub use crate::builtins::temporal::plain_time::*;
pub use crate::builtins::temporal::plain_year_month::*;
pub use crate::builtins::temporal::zoned_date_time::*;

use crate::{Object, ObjectHandle, Realm, Res, Symbol, Value, Variable};
use crate::partial_init::Initializer;

fn constr(obj: &ObjectHandle, realm: &mut Realm) -> Variable {
    Variable::write_config(
        obj.resolve_property("constructor", realm)
            .ok()
            .flatten()
            .unwrap_or(Value::Undefined),
    )
}

pub struct Temporal;

impl Initializer<ObjectHandle> for Temporal {
    fn initialize(realm: &mut Realm) -> Res<ObjectHandle> {
        get_temporal(realm)
    }
}

pub fn get_temporal(
    realm: &mut crate::Realm,
) -> Res<ObjectHandle> {
    let obj = Object::with_proto(realm.intrinsics.obj.clone());

    let intrinsics = realm.intrinsics.clone_public();

    obj.define_property_attributes("Duration".into(), constr(intrinsics.temporal_duration.get(realm)?, realm), realm)?;

    obj.define_property_attributes("Instant".into(), constr(intrinsics.temporal_instant.get(realm)?, realm), realm)?;

    obj.define_property_attributes("Now".into(), constr(intrinsics.temporal_now.get(realm)?, realm), realm)?;


    obj.define_property_attributes("PlainDate".into(), constr(intrinsics.temporal_plain_date.get(realm)?, realm), realm)?;

    obj.define_property_attributes("PlainTime".into(), constr(intrinsics.temporal_plain_time.get(realm)?, realm), realm)?;

    obj.define_property_attributes(
        "PlainDateTime".into(),
        constr(intrinsics.temporal_plain_date_time.get(realm)?, realm),
        realm,
    )?;

    obj.define_property_attributes(
        "PlainMonthDay".into(),
        constr(intrinsics.temporal_plain_month_day.get(realm)?, realm),
        realm,
    )?;

    obj.define_property_attributes(
        "PlainYearMonth".into(),
        constr(intrinsics.temporal_plain_year_month.get(realm)?, realm),
        realm,
    )?;

    obj.define_property_attributes(
        "ZonedDateTime".into(),
        constr(intrinsics.temporal_zoned_date_time.get(realm)?, realm),
        realm,
    )?;

    obj.define_property_attributes(
        Symbol::TO_STRING_TAG.into(),
        Variable::config("Temporal".into()),
        realm,
    )?;

    Ok(obj)
}
