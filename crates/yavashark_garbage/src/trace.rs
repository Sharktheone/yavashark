#![allow(clippy::unwrap_used)]
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

use eframe::Frame;
use egui::{CentralPanel, Context};
use egui::mutex::Mutex;
use layout::backends::svg::SVGWriter;
use layout::core::base::Orientation;
use layout::core::geometry::Point;
use layout::core::style::StyleAttr;
use layout::std_shapes::shapes::{Arrow, Element, ShapeKind};
use layout::topo::layout::VisualGraph;
use lazy_static::lazy_static;

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
}


impl Trace {
    fn delete_cache(&mut self) {
        self.svg_content = None;
    }
}


pub struct Tracer(Arc<Mutex<Trace>>);

impl Tracer {
    pub fn new() -> Self {
        let tracer = Arc::new(Mutex::new(Trace {
            items: HashMap::new(),
            next: TraceID(0),
            svg_content: None,
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
        
        trace.delete_cache();

        id
    }

    pub fn add_ref(&self, id: TraceID, ref_id: TraceID) {
        let mut trace = self.0.lock();
        let item = trace.items.get_mut(&id).unwrap();
        item.refs.push(ref_id);

        let ref_item = trace.items.get_mut(&ref_id).unwrap();
        ref_item.ref_by.push(id);
        
        trace.delete_cache();
    }
    
    
    pub fn remove_ref(&self, id: TraceID, ref_id: TraceID) {
        let mut trace = self.0.lock();
        let item = trace.items.get_mut(&id).unwrap();
        item.refs.retain(|&x| x != ref_id);

        let ref_item = trace.items.get_mut(&ref_id).unwrap();
        ref_item.ref_by.retain(|&x| x != id);
        
        trace.delete_cache();
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
        
        trace.delete_cache();
    }
}



lazy_static!(
    pub static ref TRACER: Tracer = Tracer::new();
);


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
        let mut trace = self.tracer.lock();
        if let Some(ref svg_content) = trace.svg_content {
            return svg_content.as_bytes().to_vec();
        }

        let mut  graph = VisualGraph::new(Orientation::TopToBottom);
        
        let mut nodes = HashMap::new();
        
        for (id, item) in &trace.items {
            
            
            
            let element = Element::create(
                ShapeKind::Circle("GcBox".to_string()),
                StyleAttr::simple(),
                Orientation::TopToBottom,
                Point::new(10.0, 10.0),
            );
            
            let handle = graph.add_node(element);
            nodes.insert(*id, (handle, item));
        }
        
        for (handle, item) in nodes.values() {
            for ref_id in &item.refs {
                if let Some((ref_handle, _)) = nodes.get(ref_id) {
                    graph.add_edge(Arrow::default(), *handle, *ref_handle);
                }
            }
        }

        let mut svg = SVGWriter::new();
        graph.do_it(false, false, false, &mut svg);
        
        let content = svg.finalize();
        
        trace.svg_content = Some(content.clone());
        drop(trace);
        
        content.as_bytes().to_vec()
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