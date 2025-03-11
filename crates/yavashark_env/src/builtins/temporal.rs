mod duration;
mod instant;
mod now;
mod plain_date;
mod plain_month_day;
mod plain_time;
mod plain_year_month;
mod zoned_date_time;

use crate::{Object, ObjectHandle, Res, Value, Variable};

pub fn get_temporal(obj_proto: ObjectHandle, func_proto: ObjectHandle) -> Res<ObjectHandle> {
    let obj = Object::with_proto(obj_proto.clone().into());
    _ = obj_proto;
    _ = func_proto;
    
    obj.define_variable("Duration".into(), Variable::write_config(Value::Undefined))?;
    obj.define_variable("Instant".into(), Variable::write_config(Value::Undefined))?;
    obj.define_variable("Now".into(), Variable::write_config(Value::Undefined))?;
    obj.define_variable("PlainDate".into(), Variable::write_config(Value::Undefined))?;
    obj.define_variable("PlainDateTime".into(), Variable::write_config(Value::Undefined))?;
    obj.define_variable("PlainMonthDay".into(), Variable::write_config(Value::Undefined))?;
    obj.define_variable("PlainYearMonth".into(), Variable::write_config(Value::Undefined))?;
    obj.define_variable("ZonedDateTime".into(), Variable::write_config(Value::Undefined))?;
    
    
    

    Ok(obj)
}