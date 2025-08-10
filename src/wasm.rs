use crate::{execute_fmt};
use std::path::PathBuf;
use std::rc::Rc;
use std::cell::RefCell;
use swc_common::input::StringInput;
use swc_common::BytePos;
use swc_ecma_ast::Stmt;
use swc_ecma_parser::{EsSyntax, Parser, Syntax};
use wasm_bindgen::prelude::*;
use yavashark_env::scope::Scope;
use yavashark_env::{Realm, Res, ValueResult};
use yavashark_interpreter::Interpreter;

#[wasm_bindgen(start)]
fn init() {
    console_error_panic_hook::set_once();
    console_log::init().expect("could not initialize logger");
}

fn parse(input: &str) -> Res<Vec<Stmt>> {
    if input.is_empty() {
        return Ok(Vec::new());
    }

    let input = StringInput::new(&input, BytePos(0), BytePos(input.len() as u32));

    let c = EsSyntax::default();
    let mut p = Parser::new(Syntax::Es(c), input, None);

    let script = p
        .parse_script()
        .map_err(|e| yavashark_env::Error::syn_error(format!("{e:?}")))?;

    Ok(script.body)
}

#[wasm_bindgen]
pub fn run_standalone(code: &str) -> String {
    match execute_fmt(code) {
        Ok(v) => v,
        Err(e) => {
            format!("Error: {:?}", e)
        }
    }
}

thread_local! {
    static LOG_CALLBACK: RefCell<Option<js_sys::Function>> = RefCell::new(None);
}

#[wasm_bindgen]
pub fn set_console_log(callback: js_sys::Function) {
    LOG_CALLBACK.with(|cell| {
        *cell.borrow_mut() = Some(callback);
    });

    // Bridge engine console.log -> JS callback
    yavashark_env::console::sink::set_log_sink(Some(|msg: &str| {
        LOG_CALLBACK.with(|cell| {
            if let Some(cb) = &*cell.borrow() {
                let _ = cb.call1(&JsValue::UNDEFINED, &JsValue::from_str(msg));
            }
        });
    }));
}

#[wasm_bindgen]
pub fn clear_console_log() {
    LOG_CALLBACK.with(|cell| {
        *cell.borrow_mut() = None;
    });
    yavashark_env::console::sink::clear_log_sink();
}

#[wasm_bindgen]
pub struct Engine {
    inner: Rc<RefCell<EngineInner>>,
}

struct EngineInner {
    realm: Realm,
    scope: Scope,
}

impl EngineInner {
    #[inline]
    fn split_mut(&mut self) -> (&mut Realm, &mut Scope) {
        (&mut self.realm, &mut self.scope)
    }
}

#[wasm_bindgen]
impl Engine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<Engine, JsValue> {
        let realm = Realm::new().map_err(|e| JsValue::from_str(&format!("{e:?}")))?;
        let scope = Scope::global(&realm, PathBuf::new());
        let inner = EngineInner { realm, scope };
        Ok(Engine { inner: Rc::new(RefCell::new(inner)) })
    }

    pub fn eval(&self, code: &str) -> Result<String, JsValue> {
        let stmts = parse(code).map_err(|e| JsValue::from_str(&format!("{e:?}")))?;
        let mut inner = self.inner.borrow_mut();

        let exec_res = {

            // let inner = &mut *inner;
            // let realm = &mut inner.realm;
            // let scope = &mut inner.scope;

            let (realm, scope) = inner.split_mut();
            Interpreter::run_in(&stmts, realm, scope)
        };

        exec_res
            .and_then(|v| v.to_string(&mut inner.realm))
            .map(|s| s.to_string())
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    pub fn eval_ok(&self, code: &str) -> bool {
        let res: ValueResult = (|| {
            let stmts = parse(code)?;
            let mut inner = self.inner.borrow_mut();
            let (realm, scope) = inner.split_mut();
            Interpreter::run_in(&stmts, realm, scope)
        })();
        res.is_ok()
    }

    pub fn run_event_loop(&self) -> js_sys::Promise {
        let inner = Rc::clone(&self.inner);
        wasm_bindgen_futures::future_to_promise(async move {
            let mut borrow = inner.borrow_mut();
            borrow.realm.run_event_loop().await;
            Ok(JsValue::UNDEFINED)
        })
    }
}
