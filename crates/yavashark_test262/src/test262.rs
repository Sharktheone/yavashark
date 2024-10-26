use swc_common::input::StringInput;
use swc_common::BytePos;
use swc_ecma_parser::{EsSyntax, Parser, Syntax};
use yavashark_env::scope::Scope;
use yavashark_env::{
    Context, ControlFlow, NativeFunction, Object, ObjectHandle, Value, ValueResult,
};
use yavashark_macro::{object, properties};
use yavashark_value::Error;

pub fn print(ctx: &mut Context) -> ObjectHandle {
    NativeFunction::new(
        "print",
        |args, _, _| {
            let Some(first) = args.first() else {
                return Err(Error::ty("expected at least one argument"));
            };

            println!("{first:?}");

            Ok(Value::Undefined)
        },
        ctx,
    )
}

#[object(direct(abstract_module_source(AbstractModuleSource)))]
#[derive(Debug)]
struct Test262 {
    ctx: Option<Context>,
}

impl Test262 {
    fn new(ctx: &Context) -> Self {
        Self {
            object: Object::raw(ctx),
            abstract_module_source: Value::Undefined.into(),
            ctx: None,
        }
    }

    fn with_realm(ctx: &Context, new_ctx: Context) -> Self {
        Self {
            object: Object::raw(ctx),
            abstract_module_source: Value::Undefined.into(),
            ctx: Some(new_ctx),
        }
    }
}

#[properties]
#[allow(clippy::needless_pass_by_value)]
impl Test262 {
    #[prop(createRealm)]
    fn create_realm(&self, _args: Vec<Value>, ctx: &Context) -> ValueResult {
        let new_ctx = Context::new().map_err(|e| Error::new_error(e.to_string()))?;
        let this: Value = ObjectHandle::new(Self::with_realm(ctx, new_ctx)).into();

        Ok(this)
    }

    #[prop(detachArrayBuffer)]
    fn detach_array_buffer(&mut self, args: Vec<Value>, ctx: &mut Context) -> ValueResult {
        Ok(Value::Undefined)
    }

    #[prop(evalScript)]
    fn eval_script(&mut self, args: Vec<Value>, ctx: &mut Context) -> ValueResult {
        let input = args.first().ok_or(Error::ty("expected one argument"))?;

        let Value::String(input) = input else {
            return Err(Error::ty("expected string"));
        };

        if input.is_empty() {
            return Ok(Value::Undefined);
        }

        let input = StringInput::new(&input, BytePos(0), BytePos(input.len() as u32 - 1));

        let c = EsSyntax::default();

        let mut p = Parser::new(Syntax::Es(c), input, None);

        let script = p
            .parse_script()
            .map_err(|e| Error::syn_error(format!("{e:?}")))?;

        let ctx = self.ctx.as_mut().unwrap_or(ctx);

        let mut scope = Scope::global(ctx);

        // scope.declare_var("$test262".to_owned(), test262) TODO: we need the realm for that :/

        yavashark_interpreter::Interpreter::run_statements(ctx, &script.body, &mut scope).or_else(
            |e| match e {
                ControlFlow::Error(e) => Err(e),
                ControlFlow::Return(v) => Ok(v),
                _ => Ok(Value::Undefined),
            },
        )

        //TODO: we should respect, what interpreter is currently running. Since the bytecode is highly experimental ride now, this is okay.
    }

    #[allow(clippy::unused_self)]
    fn gc(&self, _args: Vec<Value>, _ctx: &Context) -> ValueResult {
        // gc is always handled automatically when something goes out of scope. We don't need an extra function for that.

        Ok(Value::Undefined)
    }
}
