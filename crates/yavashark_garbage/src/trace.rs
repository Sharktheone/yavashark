#![allow(clippy::unwrap_used)]

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
#[cfg(feature = "trace")]
use std::thread;

use lazy_static::lazy_static;
use log::warn;
use rand::random;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TraceID(u64);

pub struct TraceItem {
    // id: TraceID,
    ref_by: Vec<TraceID>,
    pub(crate) refs: Vec<TraceID>,
}

lazy_static! {
    pub static ref TRACER: Tracer = Tracer::new();
}



pub struct Trace {
    pub(crate) items: HashMap<TraceID, TraceItem>,
    next: TraceID,
    #[cfg(feature = "trace")]
    pub(crate) svg_content: Option<String>,
}

impl Trace {
    fn delete_cache(&mut self) {
        #[cfg(feature = "trace")]
        {
            self.svg_content = None;
        }
    }
}

pub struct Tracer(Arc<Mutex<Trace>>);

impl Tracer {
    pub fn new() -> Self {
        let tracer = Arc::new(Mutex::new(Trace {
            items: HashMap::new(),
            next: TraceID(0),
            #[cfg(feature = "trace")]
            svg_content: None,
        }));

        #[cfg(feature = "trace")]
        let tracer2 = Arc::clone(&tracer);

        #[cfg(feature = "trace")]
        thread::spawn(|| {
            use crate::trace_gui::App;
            use eframe::NativeOptions;
            use winit::platform::wayland::EventLoopBuilderExtWayland;

            let options = NativeOptions {
                event_loop_builder: Some(Box::new(|builder| {
                    builder.with_any_thread(true);
                })),
                ..Default::default()
            };

            let res = eframe::run_native(
                "GC Trace",
                options,
                Box::new(move |cc| {
                    egui_extras::install_image_loaders(&cc.egui_ctx);
                    Box::new(App::new(tracer2))
                }),
            );

            if res.is_err() {
                error!("Failed to run the GC Trace app");
            }
        });

        Self(tracer)
    }

    pub fn add(&self) -> TraceID {
        let Ok(mut trace) = self.0.lock() else {
            return TraceID(random());
        };
        let id = trace.next;
        trace.next.0 += 1;
        trace.items.insert(
            id,
            TraceItem {
                // id,
                ref_by: Vec::new(),
                refs: Vec::new(),
            },
        );

        trace.delete_cache();

        id
    }

    pub fn add_ref(&self, id: TraceID, ref_id: TraceID) {
        let Ok(mut trace) = self.0.lock() else {
            warn!("Failed to lock the tracer");
            return;
        };
        let item = trace.items.get_mut(&id).unwrap();
        if item.refs.contains(&ref_id) {
            warn!("Ref already exists");
            return;
        }
        item.refs.push(ref_id);

        let ref_item = trace.items.get_mut(&ref_id).unwrap();
        if ref_item.ref_by.contains(&id) {
            warn!("Ref already exists");
            return;
        }
        ref_item.ref_by.push(id);

        trace.delete_cache();
    }

    pub fn remove_ref(&self, id: TraceID, ref_id: TraceID) {
        let Ok(mut trace) = self.0.lock() else {
            warn!("Failed to lock the tracer");
            return;
        };

        if let Some(item) = trace.items.get_mut(&id) {
            item.refs.retain(|&x| x != ref_id);
        }

        if let Some(ref_item) = trace.items.get_mut(&ref_id) {
            ref_item.ref_by.retain(|&x| x != id);
        }

        trace.delete_cache();
    }

    pub fn remove(&self, id: TraceID) {
        let Ok(mut trace) = self.0.lock() else {
            warn!("Failed to lock the tracer");
            return;
        };
        let item = trace.items.remove(&id).unwrap();

        for ref_id in item.refs {
            let ref_item = trace.items.get_mut(&ref_id).unwrap();
            ref_item.ref_by.retain(|&x| x != id);
        }

        for ref_id in item.ref_by {
            let ref_item = trace.items.get_mut(&ref_id).unwrap();
            ref_item.refs.retain(|&x| x != id);
        }

        trace.delete_cache();
    }
}


