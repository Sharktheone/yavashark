use std::cell::{Cell, RefCell};
use yavashark::Engine;
use yavashark_env::realm::Intrinsic;
use yavashark_env::value::Obj;
use yavashark_env::{MutObject, ObjectHandle, Realm, Res};
use yavashark_macro::{object, props};

#[object]
#[derive(Debug)]
struct Counter {
    #[mutable]
    label: String,
    count: Cell<f64>,
}

impl Counter {
    fn create(realm: &mut Realm) -> Res<ObjectHandle> {
        let counter = Self {
            // Inner is defined by the macro
            inner: RefCell::new(MutableCounter {
                object: MutObject::with_proto(Self::initialize(realm)?),
                label: String::new(),
            }),
            count: Cell::new(0.0),
        };

        Ok(counter.into_object())
    }
}

#[props(no_intrinsic)]
impl Counter {
    fn increment(&self, by: Option<f64>) -> Res<f64> {
        self.count.set(self.count.get() + by.unwrap_or(1.0));

        Ok(self.count.get())
    }

    fn value(&self) -> Res<f64> {
        Ok(self.count.get())
    }

    fn reset(&self) -> Res {
        self.count.set(0.0);

        Ok(())
    }

    #[prop("setLabel")]
    fn set_label(&self, label: String) -> String {
        let mut inner = self.inner.borrow_mut();

        inner.label = label;

        inner.label.clone()
    }

    #[prop("getLabel")]
    fn get_label(&self) -> String {
        let inner = self.inner.borrow();

        inner.label.clone()
    }
}

const SCRIPT: &str = r#"
counter.setLabel("My Counter");
counter.increment();
counter.increment(5);
console.log("counter:", counter.getLabel(), "value:", counter.value());

counter.reset();
console.log("counter:", counter.getLabel(), "after reset:", counter.value());
"#;

fn main() {
    let mut engine = Engine::new();

    let counter = Counter::create(engine.realm()).expect("failed to create counter");


    engine.realm()
        .global
        .clone()
        .define_property("counter".into(), counter.into(), engine.realm())
        .expect("failed to define counter");

    engine.run(SCRIPT, "examples/extend.rs").expect("failed to run script");
}
