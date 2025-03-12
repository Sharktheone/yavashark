mod duration;
mod instant;
mod now;
mod plain_date;
mod plain_month_day;
mod plain_time;
mod plain_year_month;
mod zoned_date_time;

use crate::builtins::temporal::duration::Duration;
use crate::{Object, ObjectHandle, Res, Value, Variable};

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
        Duration::initialize_proto(Object::raw_with_proto(obj_proto.into()), func_proto.into())?;

    obj.define_variable("Duration".into(), constr(&duration))?;
    obj.define_variable("Instant".into(), Variable::write_config(Value::Undefined))?;
    obj.define_variable("Now".into(), Variable::write_config(Value::Undefined))?;
    obj.define_variable("PlainDate".into(), Variable::write_config(Value::Undefined))?;
    obj.define_variable(
        "PlainTime".into(),
        Variable::write_config(Value::Undefined),
    )?;
    obj.define_variable(
        "PlainDateTime".into(),
        Variable::write_config(Value::Undefined),
    )?;
    obj.define_variable(
        "PlainMonthDay".into(),
        Variable::write_config(Value::Undefined),
    )?;
    obj.define_variable(
        "PlainYearMonth".into(),
        Variable::write_config(Value::Undefined),
    )?;
    obj.define_variable(
        "ZonedDateTime".into(),
        Variable::write_config(Value::Undefined),
    )?;

    Ok((
        obj,
        Protos {
            duration,
            instant: Object::null(),
            now: Object::null(),
            plain_date: Object::null(),
            plain_time: Object::null(),
            plain_date_time: Object::null(),
            plain_month_day: Object::null(),
            plain_year_month: Object::null(),
            zoned_date_time: Object::null(),
        },
    ))
}
