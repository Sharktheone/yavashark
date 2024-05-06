use crate::{Ctx, Value};

macro_rules! symbol {
    ($name:ident, $symbol:ident) => {
        pub const $name: Value<C> = Value::symbol(stringify!($symbol));
    };
}

pub struct Symbol<C: Ctx> {
    _marker: std::marker::PhantomData<C>,
}

impl<C: Ctx> Symbol<C> {
    symbol!(ASYNC_ITERATOR, asyncIterator);
    symbol!(HAS_INSTANCE, hasInstance);
    symbol!(IS_CONCAT_SPREADABLE, isConcatSpreadable);
    symbol!(ITERATOR, iterator);
    symbol!(MATCH, match);
    symbol!(MATCH_ALL, matchAll);
    symbol!(REPLACE, replace);
    symbol!(SEARCH, search);
    symbol!(SPECIES, species);
    symbol!(SPLIT, split);
    symbol!(TO_PRIMITIVE, toPrimitive);
    symbol!(TO_STRING_TAG, toStringTag);
    symbol!(UNSCOPABLES, unscopables);
}