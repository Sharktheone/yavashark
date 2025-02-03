use crate::{MutObject, Object, ObjectHandle, Realm, Symbol, Value, ValueResult};
use std::cell::RefCell;
use yavashark_macro::{object, properties_new};
use yavashark_value::{Func, Obj};

#[object]
#[derive(Debug)]
pub struct SymbolObj {
    #[mutable]
    #[primitive]
    symbol: Symbol,
}

#[object(function)]
#[derive(Debug)]
pub struct SymbolConstructor {}

impl SymbolConstructor {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(_: &Object, func: &Value) -> crate::Result<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableSymbolConstructor {
                object: MutObject::with_proto(func.copy()),
            }),
        };

        this.initialize(func.copy())?;

        Ok(this.into_object())
    }
}

#[properties_new(raw)]
impl SymbolConstructor {
    #[prop("asyncIterator")]
    const ASYNC_ITERATOR: Symbol = Symbol::ASYNC_ITERATOR;

    #[prop("hasInstance")]
    const HAS_INSTANCE: Symbol = Symbol::HAS_INSTANCE;

    #[prop("isConcatSpreadable")]
    const IS_CONCAT_SPREADABLE: Symbol = Symbol::IS_CONCAT_SPREADABLE;

    #[prop("iterator")]
    const ITERATOR: Symbol = Symbol::ITERATOR;

    #[prop("match")]
    const MATCH: Symbol = Symbol::MATCH;

    #[prop("matchAll")]
    const MATCH_ALL: Symbol = Symbol::MATCH_ALL;

    #[prop("replace")]
    const REPLACE: Symbol = Symbol::REPLACE;

    #[prop("search")]
    const SEARCH: Symbol = Symbol::SEARCH;

    #[prop("species")]
    const SPECIES: Symbol = Symbol::SPECIES;

    #[prop("split")]
    const SPLIT: Symbol = Symbol::SPLIT;

    #[prop("toPrimitive")]
    const TO_PRIMITIVE: Symbol = Symbol::TO_PRIMITIVE;

    #[prop("toStringTag")]
    const TO_STRING_TAG: Symbol = Symbol::TO_STRING_TAG;

    #[prop("unscopables")]
    const UNSCOPABLES: Symbol = Symbol::UNSCOPABLES;
}

impl Func<Realm> for SymbolConstructor {
    fn call(&self, realm: &mut Realm, args: Vec<Value>, _this: Value) -> ValueResult {
        let sym = args
            .first()
            .map_or(Ok(String::new()), |v| v.to_string(realm))?;

        Ok(Symbol::from(sym).into())
    }
}

impl SymbolObj {
    #[allow(clippy::new_ret_no_self)]
    #[must_use]
    pub fn new(realm: &Realm, symbol: Symbol) -> ObjectHandle {
        Self {
            inner: RefCell::new(MutableSymbolObj {
                object: MutObject::with_proto(realm.intrinsics.boolean.clone().into()),
                symbol,
            }),
        }
        .into_object()
    }
}

#[properties_new(constructor(SymbolConstructor::new))]
impl SymbolObj {
    #[prop("valueOf")]
    fn value_of(&self) -> Symbol {
        let inner = self.inner.borrow();

        inner.symbol.clone()
    }

    #[prop("toString")]
    fn to_js_string(&self) -> String {
        let inner = self.inner.borrow();

        inner.symbol.to_string()
    }

    #[prop(Symbol::TO_PRIMITIVE)]
    fn to_primitive(&self) -> Symbol {
        let inner = self.inner.borrow();

        inner.symbol.clone()
    }

    #[prop(Symbol::TO_STRING_TAG)]
    fn to_string_tag() -> String {
        "Symbol".to_string()
    }
}
