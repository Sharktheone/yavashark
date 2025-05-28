use crate::{Realm, Res};
use crate::experiments::gui::gui::{Gui, GuiConstructor};
use crate::experiments::gui::ui::Ui;

mod runtime_lifetime;
mod ui;
mod gui;



pub fn init(realm: &mut Realm) -> Res {
    Gui::init(realm)?;
    Ui::init(realm)?;
    
    let global = realm.global.clone();
    
    
    let proto = realm.intrinsics.get_of::<Gui>()?;
    
    let gui = proto.get("constructor", realm)?;
    
    global.set("Gui", gui, realm)?;
    
    Ok(())
    
}