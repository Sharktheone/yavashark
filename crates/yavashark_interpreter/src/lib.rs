#![allow(unused, clippy::needless_pass_by_ref_mut)] //pass by ref mut is just temporary until all functions are implemented

extern crate core;

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use swc_ecma_ast::Stmt;

use yavashark_env::scope::Scope;
use yavashark_env::{scope, ControlFlow, Realm, Result, Value, ValueResult};

mod class;
mod function;
mod location;
mod pat;
pub mod statement;
#[cfg(test)]
mod tests;
pub mod eval;

pub struct Interpreter;

impl Interpreter {
    pub fn run(script: &Vec<Stmt>, file: PathBuf) -> Result<Value> {
        let mut realm = &mut Realm::new()?;
        let mut scope = Scope::global(realm, file);

        Self::run_statements(realm, script, &mut scope).or_else(|e| match e {
            ControlFlow::Error(e) => Err(e),
            ControlFlow::Return(v) => Ok(v),
            _ => Ok(Value::Undefined),
        })
    }

    pub fn run_in(script: &Vec<Stmt>, realm: &mut Realm, scope: &mut Scope) -> Result<Value> {
        Self::run_statements(realm, script, scope).or_else(|e| match e {
            ControlFlow::Error(e) => Err(e),
            ControlFlow::Return(v) => Ok(v),
            _ => Ok(Value::Undefined),
        })
    }

    #[cfg(test)]
    #[allow(clippy::missing_panics_doc)]
    pub fn run_test(script: &Vec<Stmt>) -> (ValueResult, Rc<RefCell<yavashark_env::tests::State>>) {
        let mut context = &mut Realm::new().unwrap();
        let mut scope = Scope::global(context, PathBuf::from("test.js"));

        let (mock, state) = yavashark_env::tests::mock_object(context);

        scope.declare_global_var("mock".into(), mock);

        (
            Self::run_statements(context, script, &mut scope).or_else(|e| match e {
                ControlFlow::Error(e) => Err(e),
                ControlFlow::Return(v) => Ok(v),
                _ => Ok(Value::Undefined),
            }),
            state,
        )
    }
}

#[cfg(test)]
mod temp_test {
    use super::*;
    use swc_common::input::StringInput;
    use swc_common::BytePos;
    use swc_ecma_parser::{EsSyntax, Parser, Syntax};
    use yavashark_env::test_eval;

    #[test]
    fn math() {
        {
            let src = r#"

        let x = 1 + 2

        let y = x + true

        let k = x + y

        try {
            console.log(k)
            k = k + 2
            console.log(k)
        } catch (e) {
            console.log("i don't care")
        }

        let z = 69;
        function hello(a, b) {
            return a + b
        }
        // 
        if (k > 0) {
            z = 1337
        } else {
            z = 42
        }
        
        console.log(3, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12)
        console.log("3+4 is", hello(3, 4))
        
        
        let yyy = 1 + 2
        
        switch (yyy) {
            case 1:
                console.log("one")
                break
            case 2:
                console.log("two")
                break
            case 3:
                console.log("three")
            case 4:
                console.log("four")
            case 5:
                console.log("five")
            default:
                console.log("default")
        }
        
        for (let i = 0; i < 10; i++) {
            console.log(i)
        }
        
        console.log(this)
        
        
        function Hello() {
            this.x = 1
            this.y = 2
        }
        
        
        console.log(new Hello())
        
        
        try {
            throw 1
        } catch ({message}) {
            console.log("error:", message)
        }
        
        let a = 1
        while (a < 10) {
            console.log("infinite loop")
            a++;
        }

        let array = new Array(4, 5, 6, 7)
        
        
        console.log(array[0], array[1], array[2], array[3])
        
        for (let i in array) {
            console.log("in", i)
        }
        
        for (let i of array) {
            console.log("of", i)
        }
        
        
        
        let arrow = (a, b) => {
            return a + b
        }
        
        console.log("arrow", arrow(1, 2))
        
        function Arrows() {
            this.x = "hello"
            this.y = "world"
            
            this.arrow = (a, b) => {
                console.log("from_arrows", this.x, this.y)
                return a + b
            }
        }
        
        let arr = new Arrows()
        
        console.log("arrow", arr.arrow(1, 2))
        
        
        // console.log(Array) //TODO: this causes an stack overflow (probably because of the prototype)


        let lit_array = [1,2,3,,4,5,6,7,,,8,9,10]
        
        console.log(lit_array)
        
        let obj = {
            x: 11,
            y: 22,
            z: 33
        }
        
        console.log(obj)
        console.log(obj.x, obj.y, obj.z)
        
        
        let x = function() {
        console.log("hello")
        }
        
        x()
        


        class HelloClass {
            constructor() {
                this.x = 1
                this.y = 2
                console.log("Hello from cosntructor!")
            }
            
            hello() {
                console.log("hello from class")
            }
            
            
            static staticHello() {
                console.log("static hello")
            }
        }
        
        HelloClass.staticHello()
        
        let h = new HelloClass() //constructor is wrong somehow


        console.log(h.__proto__)

        h.hello()

        console.log(h.x, h.y)


        // console.log([] instanceof Array)
        
        console.log(undefined?.x)
        
        console.log(null?.x)
        
        console.log(undefined?.x?.y)
        console.log(null?.x?.y)
        
        console.log(undefined?.x.y.z())
        
        
        console.log(true ? 1 : 2)
        console.log(false ? 1 : 2)
        console.log({} ? 1 : 2)        
        console.log([] ? 1 : 2)
        console.log(0 ? 1 : 2)
        console.log(1 ? 1 : 2)
        
        
        function returnsSomething() {
            return 1
        }
        
        console.log(returnsSomething(), "returnsSomething")
        
        
        
        let obj = {
            x: 1,
            y: 2,
            z: 3,
            
            get hello() {
                console.log("executing getter")
                return this.x + this.y + this.z
            },
            
            set hello(value) {
                console.log("executing setter")
                this.x = value
            }
        }
        
        
        console.log(obj.hello, "getter")
        obj.hello = 10
        
        console.log(obj.hello, "yeeeeet")
        
        
        console.log(`hello ${1 + 2} world`)
        console.log(`hello ${1 + 2} world ${3 + 4}`)
        console.log(`hello ${obj.hello} world ${obj}`)

        z
        "#;

            env_logger::Builder::from_default_env()
                .filter_level(log::LevelFilter::Warn)
                .init();

            let input = StringInput::new(src, BytePos(0), BytePos(src.len() as u32 - 1));

            let c = EsSyntax::default();

            let mut p = Parser::new(Syntax::Es(c), input, None);
            let script = p.parse_script().unwrap();

            let result = Interpreter::run(&script.body, PathBuf::from("test.js")).unwrap();

            println!("{result:?}");
        }

        // #[cfg(not(miri))]
        // std::thread::sleep(std::time::Duration::from_secs(20));

        println!(
            "LEAKED OBJECTS: {}/{}",
            yavashark_value::OBJECT_COUNT.get(),
            yavashark_value::OBJECT_ALLOC.get()
        );
    }

    #[test]
    fn iterator() {
        test_eval!(
            r#"
            let array = [1,2,3,4]
            
            
            console.log(array.__proto__)
            
               
            // for (let i in array) {
            //     console.log("in", i)
            // }
            
            for (let i of array) {
                console.log("of", i)
            }
            
            
            "#,
            0,
            Vec::<Vec<Value>>::new(),
            Value::Undefined
        );
    }
}
