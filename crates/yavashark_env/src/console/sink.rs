use std::sync::RwLock;

#[cfg(target_arch = "wasm32")]
type SinkFn = dyn Fn(&str) + 'static;
#[cfg(not(target_arch = "wasm32"))]
type SinkFn = dyn Fn(&str) + Send + Sync + 'static;

// Global, process-wide log sink. Host environments (e.g. WASM) can install a callback.
// It is intentionally simple and single-callback for now.
static LOG_SINK: RwLock<Option<Box<SinkFn>>> = RwLock::new(None);

#[cfg(target_arch = "wasm32")]
pub fn set_log_sink<F>(f: Option<F>)
where
    F: Fn(&str) + 'static,
{
    let mut w = LOG_SINK
        .write()
        .expect("yavashark_env: failed to acquire LOG_SINK write lock");
    *w = f.map(|f| Box::new(f) as Box<SinkFn>);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_log_sink<F>(f: Option<F>)
where
    F: Fn(&str) + Send + Sync + 'static,
{
    let mut w = LOG_SINK
        .write()
        .unwrap_or_else(|e| e.into_inner());

    *w = f.map(|f| Box::new(f) as Box<SinkFn>);
}

pub fn clear_log_sink() {
    set_log_sink::<fn(&str)>(None);
}

pub fn call_log_sink(msg: &str) -> bool {
    let r = LOG_SINK
        .read()
        .unwrap_or_else(|e| e.into_inner());

    (*r).as_ref().is_some_and(|f| {
        f(msg);
        true
    })
}
