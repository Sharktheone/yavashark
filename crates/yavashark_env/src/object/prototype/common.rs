#![allow(clippy::needless_pass_by_value, unused)]

use yavashark_string::YSString;
use crate::array::Array;
use crate::builtins::{Arguments, BooleanObj, Date, NumberObj, RegExp, StringObj};
use crate::error_obj::ErrorObj;
use crate::realm::Realm;
use crate::utils::coerce_object;
use crate::value::property_key::IntoPropertyKey;
use crate::value::Property;
use crate::{Error, Object, ObjectOrNull, Symbol, Value, ValueResult};

pub fn define_getter(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    if args.len() < 2 {
        return Ok(Value::Undefined);
    }

    let name = args[0].copy();
    let get = args[1].copy().to_object()?;

    let this = this.as_object()?;

    this.define_getter(name.into_internal_property_key(realm)?, get, realm)?;

    Ok(Value::Undefined)
}

pub fn define_setter(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    if args.len() < 2 {
        return Ok(Value::Undefined);
    }

    let name = args[0].copy();
    let set = args[1].copy().to_object()?;

    let this = this.as_object()?;

    this.define_setter(name.into_internal_property_key(realm)?, set, realm)?;

    Ok(Value::Undefined)
}

pub fn lookup_getter(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    if args.is_empty() {
        return Ok(Value::Undefined);
    }

    let name = &args[0];

    let this = this.as_object()?;

    Ok(this
        .resolve_property_no_get_set(name, realm)?
        .map_or(Value::Undefined, |p| {
            if let Property::Getter(get, _) = p {
                get.clone().into()
            } else {
                Value::Undefined
            }
        }))
}

pub fn lookup_setter(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    if args.is_empty() {
        return Ok(Value::Undefined);
    }

    let name = &args[0];

    let _this = this.as_object()?;

    Ok(Value::Undefined)

    // Ok(this
    //     .resolve_property_no_get_set(name, realm)?
    //     .map_or(Value::Undefined, |p| ))
}

pub fn has_own_property(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    if args.is_empty() {
        return Ok(Value::Boolean(false));
    }

    let Value::Object(obj) = this else {
        return Ok(Value::Boolean(false));
    };

    let key = (&args[0]).into_internal_property_key(realm)?;

    Ok(obj.contains_own_key(key, realm)?.into())
}

pub fn get_own_property_descriptor(
    args: Vec<Value>,
    this: Value,
    realm: &mut Realm,
) -> ValueResult {
    if args.len() < 2 {
        return Ok(Value::Undefined);
    }

    let Value::Object(obj) = &args[0] else {
        return Ok(Value::Undefined);
    };

    let key = (&args[1]).into_internal_property_key(realm)?;

    let Some(prop) = obj.get_property_descriptor(key, realm)? else {
        return Ok(Value::Undefined);
    };

    prop.into_value(realm)
}

pub fn is_prototype_of(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    if args.is_empty() {
        return Ok(Value::Undefined);
    }

    let Value::Object(mut v) = args[0].copy() else {
        return Ok(false.into());
    };

    let o = this.to_object()?;

    loop {
        let proto = v.prototype(realm)?;

        let ObjectOrNull::Object(proto) = proto else {
            return Ok(false.into());
        };

        v = proto;

        if v == o {
            return Ok(true.into());
        }
    }
}

pub fn property_is_enumerable(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    let Some(prop) = args.first() else {
        return Ok(false.into());
    };

    let Value::Object(obj) = this else {
        return Ok(false.into());
    };

    let Some(prop) = obj.resolve_property_no_get_set(prop, realm)? else {
        return Ok(false.into());
    };

    Ok(prop.attributes().is_enumerable().into())
}

pub fn to_locale_string(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    Err(Error::new("Not implemented"))
}

pub fn to_string(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    if this.is_null() {
        return Ok("[object Null]".into());
    }

    if this.is_undefined() {
        return Ok("[object Undefined]".into());
    }

    let this = coerce_object(this, realm)?;

    let mut owned_tag = YSString::new();

    let mut tag = if this.is_callable() {
        "Function"
    } else if this.downcast::<Array>().is_some() {
        "Array"
    } else if this.downcast::<Arguments>().is_some() {
        "Arguments"
    } else if this.downcast::<ErrorObj>().is_some() {
        "Error"
    } else if this.downcast::<BooleanObj>().is_some() {
        "Boolean"
    } else if this.downcast::<NumberObj>().is_some() {
        "Number"
    } else if this.downcast::<StringObj>().is_some() {
        "String"
    } else if this.downcast::<Date>().is_some() {
        "Date"
    } else if this.downcast::<RegExp>().is_some() {
        "RegExp"
    } else if let Some(to_string_tag) = this.get_opt(Symbol::TO_STRING_TAG, realm)? {
        owned_tag = to_string_tag.to_string(realm)?;

        owned_tag.as_str()
    } else {
        "Object"
    };

    Ok(format!("[object {tag}]").into())
}

pub fn value_of(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    Ok(this)
}
