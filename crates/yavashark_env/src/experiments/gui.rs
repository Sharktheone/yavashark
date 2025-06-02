#![allow(unused)]
use crate::{Realm, Res};

mod codeedit;
mod gui;
mod jswidget;
mod runtime_lifetime;
mod ui;

pub use codeedit::*;
pub use gui::*;
pub use jswidget::*;
pub use runtime_lifetime::*;
pub use ui::*;


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
