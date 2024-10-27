use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use eframe::emath::TSTransform;
use eframe::Frame;
use egui::{Area, CentralPanel, Context, Id};
use layout::backends::svg::SVGWriter;
use layout::core::base::Orientation;
use layout::core::geometry::Point;
use layout::core::style::StyleAttr;
use layout::std_shapes::shapes::{Arrow, Element, ShapeKind};
use layout::topo::layout::VisualGraph;

use crate::trace::Trace;

pub struct App {
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
        let mut trace = self.tracer.lock().unwrap();
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
    fn update(&mut self, realm: &Realm, _frame: &mut Frame) {
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
