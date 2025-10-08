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

use crate::{Object, ObjectHandle, Realm, Res, Value, Variable};

pub struct Protos {
    pub duration: ObjectHandle,
    pub instant: ObjectHandle,
    pub now: ObjectHandle,
    pub plain_date: ObjectHandle,
    pub plain_time: ObjectHandle,
    pub plain_date_time: ObjectHandle,
    pub plain_month_day: ObjectHandle,
    pub plain_year_month: ObjectHandle,
    pub zoned_date_time: ObjectHandle,
}

fn constr(obj: &ObjectHandle, realm: &mut Realm) -> Variable {
    Variable::write_config(
        obj.resolve_property("constructor", realm)
            .ok()
            .flatten()
            .unwrap_or(Value::Undefined),
    )
}

pub fn get_temporal(
    obj_proto: ObjectHandle,
    func_proto: ObjectHandle,
    realm: &mut crate::Realm,
) -> Res<(ObjectHandle, Protos)> {
    let obj = Object::with_proto(obj_proto.clone());

    let duration = Duration::initialize_proto(
        Object::raw_with_proto(obj_proto.clone()),
        func_proto.clone().into(),
        realm,
    )?;
    obj.define_property_attributes("Duration".into(), constr(&duration, realm), realm)?;

    let instant = Instant::initialize_proto(
        Object::raw_with_proto(obj_proto.clone()),
        func_proto.clone().into(),
        realm,
    )?;
    obj.define_property_attributes("Instant".into(), constr(&instant, realm), realm)?;

    let now = Now::initialize_proto(
        Object::raw_with_proto(obj_proto.clone()),
        func_proto.clone().into(),
        realm,
    )?;
    obj.define_property_attributes("Now".into(), constr(&now, realm), realm)?;

    let plain_date = PlainDate::initialize_proto(
        Object::raw_with_proto(obj_proto.clone()),
        func_proto.clone().into(),
        realm,
    )?;

    obj.define_property_attributes("PlainDate".into(), constr(&plain_date, realm), realm)?;

    let plain_time = PlainTime::initialize_proto(
        Object::raw_with_proto(obj_proto.clone()),
        func_proto.clone().into(),
        realm,
    )?;
    obj.define_property_attributes("PlainTime".into(), constr(&plain_time, realm), realm)?;

    let plain_date_time = PlainDateTime::initialize_proto(
        Object::raw_with_proto(obj_proto.clone()),
        func_proto.clone().into(),
        realm,
    )?;
    obj.define_property_attributes(
        "PlainDateTime".into(),
        constr(&plain_date_time, realm),
        realm,
    )?;

    let plain_month_day = PlainMonthDay::initialize_proto(
        Object::raw_with_proto(obj_proto.clone()),
        func_proto.clone().into(),
        realm,
    )?;
    obj.define_property_attributes(
        "PlainMonthDay".into(),
        constr(&plain_month_day, realm),
        realm,
    )?;

    let plain_year_month = PlainYearMonth::initialize_proto(
        Object::raw_with_proto(obj_proto.clone()),
        func_proto.clone().into(),
        realm,
    )?;
    obj.define_property_attributes(
        "PlainYearMonth".into(),
        constr(&plain_year_month, realm),
        realm,
    )?;

    let zoned_date_time = ZonedDateTime::initialize_proto(
        Object::raw_with_proto(obj_proto),
        func_proto.into(),
        realm,
    )?;
    obj.define_property_attributes(
        "ZonedDateTime".into(),
        constr(&zoned_date_time, realm),
        realm,
    )?;

    Ok((
        obj,
        Protos {
            duration,
            instant,
            now,
            plain_date,
            plain_time,
            plain_date_time,
            plain_month_day,
            plain_year_month,
            zoned_date_time,
        },
    ))
}
