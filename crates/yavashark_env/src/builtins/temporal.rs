mod duration;
mod instant;
mod now;
mod plain_date;
mod plain_month_day;
mod plain_time;
mod plain_year_month;
mod zoned_date_time;
mod plain_date_time;

use crate::builtins::temporal::duration::Duration;
use crate::{Object, ObjectHandle, Res, Value, Variable};
use crate::builtins::temporal::instant::Instant;
use crate::builtins::temporal::now::Now;
use crate::builtins::temporal::plain_date::PlainDate;
use crate::builtins::temporal::plain_date_time::PlainDateTime;
use crate::builtins::temporal::plain_month_day::PlainMonthDay;
use crate::builtins::temporal::plain_time::PlainTime;
use crate::builtins::temporal::plain_year_month::PlainYearMonth;
use crate::builtins::temporal::zoned_date_time::ZonedDateTime;

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

fn constr(obj: &ObjectHandle) -> Variable {
    Variable::write_config(
        obj.get_property(&"constructor".into())
            .unwrap_or(Value::Undefined.into())
            .value,
    )
}

pub fn get_temporal(
    obj_proto: ObjectHandle,
    func_proto: ObjectHandle,
) -> Res<(ObjectHandle, Protos)> {
    let obj = Object::with_proto(obj_proto.clone().into());

    let duration =
        Duration::initialize_proto(Object::raw_with_proto(obj_proto.clone().into()), func_proto.clone().into())?;
    obj.define_variable("Duration".into(), constr(&duration))?;
    
    let instant = Instant::initialize_proto(Object::raw_with_proto(obj_proto.clone().into()), func_proto.clone().into())?;
    obj.define_variable("Instant".into(), constr(&instant))?;
    
    let now = Now::initialize_proto(Object::raw_with_proto(obj_proto.clone().into()), func_proto.clone().into())?;
    obj.define_variable("Now".into(), constr(&now))?;
    
    let plain_date = PlainDate::initialize_proto(Object::raw_with_proto(obj_proto.clone().into()), func_proto.clone().into())?;
    obj.define_variable("PlainDate".into(), constr(&plain_date))?;
    
    let plain_time = PlainTime::initialize_proto(Object::raw_with_proto(obj_proto.clone().into()), func_proto.clone().into())?;
    obj.define_variable(
        "PlainTime".into(),
        constr(&plain_time),
    )?;
    
    let plain_date_time = PlainDateTime::initialize_proto(Object::raw_with_proto(obj_proto.clone().into()), func_proto.clone().into())?;
    obj.define_variable(
        "PlainDateTime".into(),
        constr(&plain_date_time),
    )?;
    
    let plain_month_day = PlainMonthDay::initialize_proto(Object::raw_with_proto(obj_proto.clone().into()), func_proto.clone().into())?;
    obj.define_variable(
        "PlainMonthDay".into(),
        constr(&plain_month_day),
    )?;
    
    let plain_year_month = PlainYearMonth::initialize_proto(Object::raw_with_proto(obj_proto.clone().into()), func_proto.clone().into())?;
    obj.define_variable(
        "PlainYearMonth".into(),
        constr(&plain_year_month),
    )?;
    
    let zoned_date_time = ZonedDateTime::initialize_proto(Object::raw_with_proto(obj_proto.into()), func_proto.into())?;
    obj.define_variable(
        "ZonedDateTime".into(),
        constr(&zoned_date_time),
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
