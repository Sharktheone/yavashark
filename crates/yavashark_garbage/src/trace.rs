#![allow(clippy::unwrap_used)]

use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

use eframe::{Frame, NativeOptions};
use egui::{Area, CentralPanel, Context, Id};
use egui::emath::TSTransform;
use egui::mutex::Mutex;
use layout::backends::svg::SVGWriter;
use layout::core::base::Orientation;
use layout::core::geometry::Point;
use layout::core::style::StyleAttr;
use layout::std_shapes::shapes::{Arrow, Element, ShapeKind};
use layout::topo::layout::VisualGraph;
use lazy_static::lazy_static;
use log::{error, warn};
use winit::platform::wayland::EventLoopBuilderExtWayland;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TraceID(u64);

pub struct TraceItem {
    // id: TraceID,
    ref_by: Vec<TraceID>,
    refs: Vec<TraceID>,
}

// const ZOOM_SPEED: f32 = 1.0;
// const SCROLL_SPEED: f32 = 1.0;

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

        let tracer2 = Arc::clone(&tracer);

        thread::spawn(|| {
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
        let mut trace = self.0.lock();
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
        let mut trace = self.0.lock();
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

lazy_static! {
    pub static ref TRACER: Tracer = Tracer::new();
}

struct App {
    tracer: Arc<Mutex<Trace>>,
    transform: TSTransform,
    // drag_value: f32,
}

impl App {
    pub fn new(tracer: Arc<Mutex<Trace>>) -> Self {
        Self {
            tracer,
            transform: TSTransform::default(),
            // drag_value: 0.0,
        }
    }

    pub fn layout(&mut self) -> Vec<u8> {
        let mut trace = self.tracer.lock();
        if let Some(ref svg_content) = trace.svg_content {
            return svg_content.as_bytes().to_vec();
        }

        if trace.items.is_empty() {
            return Vec::new();
        }

        let mut graph = VisualGraph::new(Orientation::TopToBottom);

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
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Garbage Collector Trace");

            let img = egui::Image::from_bytes("bytes://graph.svg", self.layout())
                .fit_to_original_size(1.0);

            let (id, rect) = ui.allocate_space(ui.available_size());
            let response = ui.interact(rect, id, egui::Sense::click_and_drag());
            // Allow dragging the background as well.
            if response.dragged() {
                self.transform.translation += response.drag_delta();
            }

            // Plot-like reset
            if response.double_clicked() {
                self.transform = TSTransform::default();
            }

            let transform =
                TSTransform::from_translation(ui.min_rect().left_top().to_vec2()) * self.transform;

            if let Some(pointer) = ui.ctx().input(|i| i.pointer.hover_pos()) {
                // Note: doesn't catch zooming / panning if a button in this PanZoom container is hovered.
                if response.hovered() {
                    let pointer_in_layer = transform.inverse() * pointer;
                    let zoom_delta = ui.ctx().input(egui::InputState::zoom_delta);
                    let pan_delta = ui.ctx().input(|i| i.smooth_scroll_delta);

                    // Zoom in on pointer:
                    self.transform = self.transform
                        * TSTransform::from_translation(pointer_in_layer.to_vec2())
                        * TSTransform::from_scaling(zoom_delta)
                        * TSTransform::from_translation(-pointer_in_layer.to_vec2());

                    // Pan:
                    self.transform = TSTransform::from_translation(pan_delta) * self.transform;
                }
            }

            let id = Area::new(Id::new("graph"))
                .show(ctx, |ui| {
                    ui.set_clip_rect(self.transform.inverse() * rect);
                    ui.add(img);
                })
                .response
                .layer_id;

            ui.ctx().set_transform_layer(id, transform);
        });
    }
}
