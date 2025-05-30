use crate::{Realm, Res};
use crate::experiments::gui::gui::{Gui, GuiConstructor};
use crate::experiments::gui::ui::Ui;

mod runtime_lifetime;
mod ui;
mod gui;
mod codeedit;
mod jswidget;

pub use runtime_lifetime::*;
pub use ui::*;
pub use gui::*;
pub use codeedit::*;
pub use jswidget::*;

pub fn init(realm: &mut Realm) -> Res {
    Gui::init(realm)?;
    Ui::init(realm)?;
    JSCodeEditor::init(realm)?;
    
    let global = realm.global.clone();
    
    
    let proto = realm.intrinsics.get_of::<Gui>()?;
    
    let gui = proto.get("constructor", realm)?;
    
    global.set("Gui", gui, realm)?;

    let proto = realm.intrinsics.get_of::<JSCodeEditor>()?;

    let gui = proto.get("constructor", realm)?;

    global.set("CodeEditor", gui, realm)?;
    
    Ok(())
    
}