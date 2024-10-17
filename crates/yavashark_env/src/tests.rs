use std::cell::RefCell;
use std::rc::Rc;

use crate::context::Context;
use crate::object::Object;
use crate::{NativeFunction, Value};

#[macro_export]
#[allow(clippy::crate_in_macro_def)]
macro_rules! test_eval {
    ($code:expr, $sends:literal, $values:expr, object) => {
        use swc_common::BytePos;
        let src = $code;
        let input =
            swc_ecma_parser::StringInput::new(src, BytePos(0), BytePos(src.len() as u32 - 1));

        let c = Default::default();

        let mut p = swc_ecma_parser::Parser::new(swc_ecma_parser::Syntax::Es(c), input, None);
        let script = p.parse_script().unwrap();

        let (result, values) = crate::Interpreter::run_test(&script.body);

        let result = result.unwrap();

        assert!(matches!(result, Value::Object(_)));
        let state = values.borrow();
        assert_eq!(state.send_called, $sends);
        assert_eq!(state.got_values, $values);
    }; // ($code:expr, $sends:literal, $values:expr, $ret:expr) => {}; //TODO
    
    
    ($code:expr, $sends:literal, $values:expr, $ret:expr) => {
        use swc_common::BytePos;
        let src = $code;
        let input =
            swc_ecma_parser::StringInput::new(src, BytePos(0), BytePos(src.len() as u32 - 1));

        let c = Default::default();

        let mut p = swc_ecma_parser::Parser::new(swc_ecma_parser::Syntax::Es(c), input, None);
        let script = p.parse_script().unwrap();

        let (result, values) = crate::Interpreter::run_test(&script.body);

        let result = result.unwrap();

        assert_eq!(result, $ret);
        let state = values.borrow();
        assert_eq!(state.send_called, $sends);
        assert_eq!(state.got_values, $values);
    }; // ($code:expr, $sends:literal, $values:expr, $ret:expr) => {}; //TODO

    ($code:expr) => {{
        use swc_common::BytePos;
        let src = $code;
        let input =
            swc_ecma_parser::StringInput::new(src, BytePos(0), BytePos(src.len() as u32 - 1));

        let c = Default::default();

        let mut p = swc_ecma_parser::Parser::new(swc_ecma_parser::Syntax::Es(c), input, None);
        let script = p.parse_script().unwrap();

        crate::Interpreter::run_test(&script.body)
    }};
}

#[macro_export]
macro_rules! expr {
    ($code:expr, NaN) => {
        let (res, _) = test_eval!($code);

        let res = res.unwrap();

        if let Value::Number(n) = &res {
            assert!(n.is_nan(), "Expected NaN, got {}", *n);
        } else {
            panic!("Expected a number, got {:?}", res);
        }
    };
    ($code:expr, $res:expr) => {
        $crate::test_eval!($code, 0, Vec::<Vec<Value>>::new(), $res)
    };
}

pub struct State {
    pub send_called: u16,
    pub got_values: Vec<Vec<Value>>,
}

#[must_use]
pub fn mock_object(ctx: &Context) -> (Value, Rc<RefCell<State>>) {
    let obj = Object::new(ctx);

    let state = Rc::new(RefCell::new(State {
        send_called: 0,
        got_values: Vec::new(),
    }));

    let send_state = Rc::clone(&state);
    let _ = obj.define_property(
        "send".into(),
        NativeFunction::new(
            "send",
            move |_, _, _| {
                let mut state = send_state.borrow_mut();
                state.send_called += 1;

                Ok(Value::Undefined)
            },
            ctx,
        )
        .into(),
    );

    let values_state = Rc::clone(&state);
    let _ = obj.define_property(
        "values".into(),
        NativeFunction::new(
            "values",
            move |args, _, _| {
                let mut state = values_state.borrow_mut();
                state.got_values.push(args);

                Ok(Value::Undefined)
            },
            ctx,
        )
        .into(),
    );

    (obj.into(), state)
}
