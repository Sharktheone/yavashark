use crate::value::{Func, IntoValue, Obj};
use crate::{MutObject, Object, ObjectHandle, Realm, Res, Symbol, Value, ValueResult};
use std::cell::RefCell;
use yavashark_macro::{object, properties_new};
use yavashark_string::ToYSString;

#[object]
#[derive(Debug)]
pub struct SymbolObj {
    #[mutable]
    #[primitive]
    symbol: Symbol,
}

#[object(function)]
#[derive(Debug)]
pub struct SymbolConstructor {
    #[mutable]
    symbols: Vec<Symbol>,
}

impl SymbolConstructor {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(_: &Object, func: ObjectHandle, realm: &mut Realm) -> crate::Res<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableSymbolConstructor {
                object: MutObject::with_proto(func.clone()),
                symbols: Vec::new(),
            }),
        };

        this.initialize(realm)?;

        Ok(this.into_object())
    }

    pub fn find_symbol(&self, symbol: &str) -> Option<Symbol> {
        self.inner
            .borrow()
            .symbols
            .iter()
            .find(|s| s.as_ref() == symbol)
            .cloned()
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

    #[prop("dispose")]
    const DISPOSE: &'static Symbol = Symbol::DISPOSE;

    #[prop("asyncDispose")]
    const ASYNC_DISPOSE: &'static Symbol = Symbol::ASYNC_DISPOSE;

    #[prop("keyFor")]
    fn key_for(symbol: Symbol) -> Value {
        let str = symbol.to_ys_string();

        if str.is_empty() {
            return Value::Undefined;
        }

        str.into_value()
    }

    #[prop("for")]
    fn for_(&self, key: &str) -> Symbol {
        if let Some(sym) = self.find_symbol(key) {
            return sym;
        }

        let new_symbol = Symbol::new_str(key);
        self.inner.borrow_mut().symbols.push(new_symbol.clone());

        new_symbol
    }
}

impl Func for SymbolConstructor {
    fn call(&self, realm: &mut Realm, args: Vec<Value>, _this: Value) -> ValueResult {
        let sym = args.first().map_or(Res::<String>::Ok(String::new()), |v| {
            if v.is_undefined() {
                return Ok(String::new());
            }

            Ok(v.to_string(realm)?.to_string())
        })?;

        let sym = Symbol::new_str(&sym);

        self.inner.borrow_mut().symbols.push(sym.clone());

        Ok(sym.into())
    }
}

impl SymbolObj {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(realm: &mut Realm, symbol: Symbol) -> Res<ObjectHandle> {
        Ok(Self {
            inner: RefCell::new(MutableSymbolObj {
                object: MutObject::with_proto(
                    realm.intrinsics.clone_public().symbol.get(realm)?.clone(),
                ),
                symbol,
            }),
        }
        .into_object())
    }
}

#[properties_new(intrinsic_name(symbol), constructor(SymbolConstructor::new))]
impl SymbolObj {
    #[prop("valueOf")]
    fn value_of(&self) -> Symbol {
        let inner = self.inner.borrow();

        inner.symbol.clone()
    }

    #[get("description")]
    fn description(&self) -> Value {
        SymbolConstructor::key_for(self.inner.borrow().symbol.clone())
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
