use crate::context::Context;
use crate::object::Object;
use crate::{NativeFunction, Value};
use std::cell::RefCell;
use std::rc::Rc;

#[macro_export]
macro_rules! test_eval {
    ($code:expr, $sends:literal, $values:expr, $ret:expr) => {
        use swc_common::BytePos;
        let src = $code;
        let input =
            swc_ecma_parser::StringInput::new(src, BytePos(0), BytePos(src.len() as u32 - 1));

        let c = Default::default();

        let mut p = swc_ecma_parser::Parser::new(swc_ecma_parser::Syntax::Es(c), input, None);
        let script = p.parse_script().unwrap();

        let interpreter = $crate::Interpreter::new(script.body);
        let (result, values) = interpreter.run_test();

        let result = result.unwrap();

        assert_eq!(result, $ret);
        let state = values.borrow();
        assert_eq!(state.send_called, $sends);
        assert_eq!(state.got_values, $values);
    };
}

pub struct State {
    pub send_called: u16,
    pub got_values: Vec<Vec<Value>>,
}

pub fn mock_object(ctx: &mut Context) -> (Value, Rc<RefCell<State>>) {
    let mut obj = Object::new(ctx);

    let state = Rc::new(RefCell::new(State {
        send_called: 0,
        got_values: Vec::new(),
    }));

    let send_state = state.clone();
    obj.define_property(
        "send".into(),
        NativeFunction::new(
            "send",
            move |args, _| {
                let mut state = send_state.borrow_mut();
                state.send_called += 1;

                Ok(Value::Undefined)
            },
            ctx,
        )
        .into(),
    );

    let values_state = state.clone();
    obj.define_property(
        "values".into(),
        NativeFunction::new(
            "values",
            move |args, _| {
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
