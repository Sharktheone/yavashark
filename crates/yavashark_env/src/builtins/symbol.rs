use std::cell::RefCell;
use yavashark_macro::{object, properties_new};
use yavashark_value::{Func, Obj};
use crate::{MutObject, Object, ObjectHandle, Realm, Symbol, Value, ValueResult};

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
        let this = Self {
            inner: RefCell::new(MutableSymbolConstructor {
                object: MutObject::with_proto(func.copy()),
            }),
        };

        Ok(this.into_object())
    }
}

impl Func<Realm> for SymbolConstructor {
    fn call(&self, realm: &mut Realm, args: Vec<Value>, _this: Value) -> ValueResult {
        let sym = args.first().map_or(Ok(String::new()), |v| v.to_string(realm))?;

        Ok(Symbol::from(sym).into())
    }
}

impl SymbolObj {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(realm: &Realm, symbol: Symbol) -> ObjectHandle {
        Self {
            inner: RefCell::new(MutableSymbolObj {
                object: MutObject::with_proto(realm.intrinsics.boolean.clone().into()),
                symbol,
            }),
        }.into_object()
    }
}

#[properties_new]
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
    fn to_string_tag(&self) -> String {
        "Symbol".to_string()
    }
}

