use crate::{Ctx, Value};

macro_rules! symbol {
    ($name:ident, $symbol:ident) => {
        pub fn $name() -> Value<C> {
            Value::symbol(stringify!($symbol))
        }
    };
    
    ($name:ident) => {
        pub fn $name() -> Value<C> {
            Value::symbol(stringify!(Symbol.$name))
        }
    };
}

pub struct Symbol<C: Ctx> {
    _marker: std::marker::PhantomData<C>,
}

impl<C: Ctx> Symbol<C> {
    symbol!(async_iterator, asyncIterator);
    symbol!(has_instance, hasInstance);
    symbol!(is_concat_spreadable, isConcatSpreadable);
    symbol!(iterator);
    symbol!(match_, match);
    symbol!(match_all, matchAll);
    symbol!(replace);
    symbol!(search);
    symbol!(species);
    symbol!(split);
    symbol!(to_primitive, toPrimitive);
    symbol!(to_string_tag, toStringTag);
    symbol!(unscopables, unscopables);
}