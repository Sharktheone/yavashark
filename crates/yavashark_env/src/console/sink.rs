#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::RwLock;

#[cfg(target_arch = "wasm32")]
thread_local! {
    static LOG_SINK: RefCell<Option<Box<dyn Fn(&str) + 'static>>> = RefCell::new(None);
}

#[cfg(target_arch = "wasm32")]
pub fn set_log_sink<F>(f: Option<F>)
where
    F: Fn(&str) + 'static,
{
    LOG_SINK.with(|cell| {
        *cell.borrow_mut() = f.map(|f| Box::new(f) as Box<dyn Fn(&str) + 'static>);
    });
}

#[cfg(target_arch = "wasm32")]
pub fn clear_log_sink() {
    set_log_sink::<fn(&str)>(None);
}

#[cfg(target_arch = "wasm32")]
pub fn call_log_sink(msg: &str) -> bool {
    LOG_SINK.with(|cell| {
        if let Some(f) = &*cell.borrow() {
            f(msg);
            true
        } else {
            false
        }
    })
}

#[cfg(not(target_arch = "wasm32"))]
type SinkFn = dyn Fn(&str) + Send + Sync + 'static;

#[cfg(not(target_arch = "wasm32"))]
static LOG_SINK: RwLock<Option<Box<SinkFn>>> = RwLock::new(None);

#[cfg(not(target_arch = "wasm32"))]
pub fn set_log_sink<F>(f: Option<F>)
where
    F: Fn(&str) + Send + Sync + 'static,
{
    let mut w = LOG_SINK
        .write()
        .unwrap_or_else(|e| {
            e.into_inner()
        });

    *w = f.map(|f| Box::new(f) as Box<SinkFn>);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn clear_log_sink() {
    set_log_sink::<fn(&str)>(None);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn call_log_sink(msg: &str) -> bool {
    let r = LOG_SINK.read().unwrap_or_else(|e| {
        e.into_inner()
    });

    (*r).as_ref().is_some_and(|f| {
        f(msg);
        true
    })
}
