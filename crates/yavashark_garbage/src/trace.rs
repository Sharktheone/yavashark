use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

use eframe::Frame;
use egui::{CentralPanel, Context};
use egui::mutex::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TraceID(u64);

pub struct TraceItem {
    id: TraceID,
    ref_by: Vec<TraceID>,
    refs: Vec<TraceID>,
}


pub struct Trace {
    items: HashMap<TraceID, TraceItem>,
    next: TraceID,
    svg_content: Option<String>,
    dot_content: Option<String>,

}


struct Tracer(Arc<Mutex<Trace>>);

impl Tracer {
    pub fn new() -> Self {
        let tracer = Arc::new(Mutex::new(Trace {
            items: HashMap::new(),
            next: TraceID(0),
            svg_content: None,
            dot_content: None,
        }));

        let tracer2 = tracer.clone();

        let handle = thread::spawn(|| {
            let x = eframe::run_native("GC Trace",
                                       Default::default(),
                                       Box::new(move |cc| {
                                           egui_extras::install_image_loaders(&cc.egui_ctx);

                                           Box::new(App::new(tracer2))
                                       }),
            );
        });


        Self(tracer)
    }

    pub fn add(&self) -> TraceID {
        let mut trace = self.0.lock();
        let id = trace.next;
        trace.next.0 += 1;
        trace.items.insert(id, TraceItem {
            id,
            ref_by: Vec::new(),
            refs: Vec::new(),
        });

        id
    }

    pub fn add_ref(&self, id: TraceID, ref_id: TraceID) {
        let mut trace = self.0.lock();
        let item = trace.items.get_mut(&id).unwrap();
        item.refs.push(ref_id);

        let ref_item = trace.items.get_mut(&ref_id).unwrap();
        ref_item.ref_by.push(id);
    }

    pub fn remove(&self, id: TraceID) {
        let mut trace = self.0.lock();
        let item = trace.items.remove(&id).unwrap();

        for ref_id in item.refs {
            let ref_item = trace.items.get_mut(&ref_id).unwrap();
            ref_item.ref_by.retain(|&x| x != id);
        }

        for ref_id in item.ref_by {
            let ref_item = trace.items.get_mut(&ref_id).unwrap();
            ref_item.refs.retain(|&x| x != id);
        }
    }
    
    pub fn add_ref_by(&self, id: TraceID, ref_id: TraceID) {
        todo!()
    }
    
    pub fn remove_ref_by(&self, id: TraceID, ref_id: TraceID) {
        todo!()
    }
}


// static mut TRACE: Tracer = Tracer::new();

struct App {
    tracer: Arc<Mutex<Trace>>,
}

impl App {
    pub fn new(tracer: Arc<Mutex<Trace>>) -> Self {
        Self {
            tracer,
        }
    }


    pub fn layout(&mut self) -> Vec<u8> {
        todo!()
    }
}


impl eframe::App for App {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Garbage Collector Trace");

            let img = egui::Image::from_bytes("bytes://graph.svg", self.layout());

            ui.add(img);
        });
    }
}