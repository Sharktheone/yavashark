use crate::{MutObject, Object, ObjectHandle, Realm, Res, Symbol, Value, ValueResult};
use std::cell::RefCell;
use yavashark_macro::{object, properties_new};
use yavashark_string::{ToYSString, YSString};
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
    pub fn new(_: &Object, func: &Value) -> crate::Res<ObjectHandle> {
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
    const ASYNC_ITERATOR: &'static Symbol = Symbol::ASYNC_ITERATOR;

    #[prop("hasInstance")]
    const HAS_INSTANCE: &'static Symbol = Symbol::HAS_INSTANCE;

    #[prop("isConcatSpreadable")]
    const IS_CONCAT_SPREADABLE: &'static Symbol = Symbol::IS_CONCAT_SPREADABLE;

    #[prop("iterator")]
    const ITERATOR: &'static Symbol = Symbol::ITERATOR;

    #[prop("match")]
    const MATCH: &'static Symbol = Symbol::MATCH;

    #[prop("matchAll")]
    const MATCH_ALL: &'static Symbol = Symbol::MATCH_ALL;

    #[prop("replace")]
    const REPLACE: &'static Symbol = Symbol::REPLACE;

    #[prop("search")]
    const SEARCH: &'static Symbol = Symbol::SEARCH;

    #[prop("species")]
    const SPECIES: &'static Symbol = Symbol::SPECIES;

    #[prop("split")]
    const SPLIT: &'static Symbol = Symbol::SPLIT;

    #[prop("toPrimitive")]
    const TO_PRIMITIVE: &'static Symbol = Symbol::TO_PRIMITIVE;

    #[prop("toStringTag")]
    const TO_STRING_TAG: &'static Symbol = Symbol::TO_STRING_TAG;

    #[prop("unscopables")]
    const UNSCOPABLES: &'static Symbol = Symbol::UNSCOPABLES;
    
    #[prop("keyFor")]
    fn key_for(symbol: Symbol) -> YSString {
        symbol.to_ys_string()
        
        
    }
}

impl Func<Realm> for SymbolConstructor {
    fn call(&self, realm: &mut Realm, args: Vec<Value>, _this: Value) -> ValueResult {
        let sym = args.first().map_or(Res::<String>::Ok(String::new()), |v| {
            Ok(v.to_string(realm)?.to_string())
        })?;

        Ok(Symbol::new_str(&sym).into())
    }
}

impl SymbolObj {
    #[allow(clippy::new_ret_no_self)]
    #[must_use]
    pub fn new(realm: &Realm, symbol: Symbol) -> ObjectHandle {
        Self {
            inner: RefCell::new(MutableSymbolObj {
                object: MutObject::with_proto(realm.intrinsics.symbol.clone().into()),
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
