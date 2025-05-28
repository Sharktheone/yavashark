use std::ptr::NonNull;
use crate::Res;

pub trait JSWidget {
    fn ui(&self, ui: &mut egui::Ui) -> Res;
}

pub struct DynWidget(pub NonNull<dyn JSWidget>);

impl DynWidget {
    pub unsafe fn get_widget(&self) -> &dyn JSWidget {
        self.0.as_ref()
    }
}
