#![allow(clippy::needless_pass_by_value, unused)]

use crate::context::Context;
use crate::{Value, ValueResult};

pub fn define_getter(args: Vec<Value>, this: Value, ctx: &mut Context) -> ValueResult {
    todo!()
}

pub fn define_setter(args: Vec<Value>, this: Value, ctx: &mut Context) -> ValueResult {
    todo!()
}

pub fn lookup_getter(args: Vec<Value>, this: Value, ctx: &mut Context) -> ValueResult {
    todo!()
}

pub fn lookup_setter(args: Vec<Value>, this: Value, ctx: &mut Context) -> ValueResult {
    todo!()
}

pub fn object_constructor(args: Vec<Value>, this: Value, ctx: &mut Context) -> ValueResult {
    todo!()
}

pub fn has_own_property(args: Vec<Value>, this: Value, ctx: &mut Context) -> ValueResult {
    todo!()
}

pub fn is_prototype_of(args: Vec<Value>, this: Value, ctx: &mut Context) -> ValueResult {
    todo!()
}

pub fn property_is_enumerable(args: Vec<Value>, this: Value, ctx: &mut Context) -> ValueResult {
    todo!()
}

pub fn to_locale_string(args: Vec<Value>, this: Value, ctx: &mut Context) -> ValueResult {
    todo!()
}

pub fn to_string(args: Vec<Value>, this: Value, ctx: &mut Context) -> ValueResult {
    Ok(this.to_string(ctx)?.into())
}

pub fn value_of(args: Vec<Value>, this: Value, ctx: &mut Context) -> ValueResult {
    todo!()
}
