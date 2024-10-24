use swc_common::BytePos;
use swc_common::input::StringInput;
use swc_ecma_parser::{EsSyntax, Parser, Syntax};
use yavashark_env::{Context, ControlFlow, NativeFunction, Object, ObjectHandle, Value, ValueResult};
use yavashark_macro::{object, properties};
use yavashark_value::Error;
use yavashark_env::scope::Scope;


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
    realm: Option<Realm>
}



#[derive(Debug)]
struct Realm {
    ctx: Context,
    scope: Scope,
}


impl Test262 {
    fn new(ctx: &Context) -> Self {
        Self {
            object: Object::raw(ctx),
            abstract_module_source: Value::Undefined.into(),
            realm: None,
        }
    }
    
    fn with_realm(ctx: Context, scope: Scope) -> Self {
        Self {
            object: Object::raw(&ctx),
            abstract_module_source: Value::Undefined.into(),
            realm: Some(Realm { ctx, scope }),
        }       
    }
}

#[properties]
#[allow(clippy::needless_pass_by_value)]
impl Test262 {
    #[prop(createRealm)]
    fn create_realm(&self, _args: Vec<Value>, _ctx: &Context) -> ValueResult {
        
        let new_ctx = Context::new().map_err(|e| Error::new_error(e.to_string()))?;
        
        let mut scope = Scope::global(&new_ctx);
        
        let this: Value = ObjectHandle::new(Self::with_realm(new_ctx, scope.clone())).into();
        
        scope.declare_var("$262".to_string(), this.copy())?;
        
        Ok(this)
    }


    #[prop(detachArrayBuffer)]
    fn detach_array_buffer(&mut self, args: Vec<Value>, ctx: &mut Context) -> ValueResult {
        Ok(Value::Undefined)
    }

    #[prop(evalScript)]
    fn eval_script(&mut self, args: Vec<Value>, ctx: &mut Context) -> ValueResult {
        
        
        let input = args.first()
            .ok_or(Error::ty("expected one argument"))?;
        
        let Value::String(input) = input else {
            return Err(Error::ty("expected string"));
        };
        
        
        if input.is_empty() {
            return Ok(Value::Undefined)
        }


        let input = StringInput::new(&input, BytePos(0), BytePos(input.len() as u32 - 1));

        let c = EsSyntax::default();

        let mut p = Parser::new(Syntax::Es(c), input, None);

        let script = p.parse_script().map_err(|e| Error::syn_error(format!("{e:?}")))?;


        let mut other_scope = Scope::global(ctx); //TODO: this is NOT the correct way of handling scopes here, since the script should be executed in the current scope.
        
        let (ctx, scope) = self.realm.as_mut().map(|r| (&mut r.ctx, &mut r.scope)).unwrap_or((ctx, &mut other_scope));


        yavashark_interpreter::Interpreter::run_statements(ctx, &script.body, scope)
            .or_else(|e| match e {
                ControlFlow::Error(e) => Err(e),
                ControlFlow::Return(v) => Ok(v),
                _ => Ok(Value::Undefined),
            })


        //TODO: we should respect, what interpreter is currently running. Since the bytecode is highly experimental ride now, this is okay.
    }

    #[allow(clippy::unused_self)]
    fn gc(&self, _args: Vec<Value>, _ctx: &Context) -> ValueResult {
        // gc is always handled automatically when something goes out of scope. We don't need an extra function for that.
        
        Ok(Value::Undefined)
    }
}