use std::cell::RefCell;
use std::fmt::Debug;
use yavashark_macro::{object, props};
use yavashark_value::Obj;
use crate::experiments::gui::runtime_lifetime::{RuntimeLifetime, RuntimeLifetimeGuard};
use crate::{MutObject, Object, ObjectHandle, Realm, Res};

#[object]
pub struct Ui {
    ui: RuntimeLifetime<egui::Ui>
}

impl Debug for Ui {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Ui")
    }
}


impl Ui {
    
    pub fn init(realm: &mut Realm) -> Res {
        let proto = Self::initialize_proto(
            Object::raw_with_proto(realm.intrinsics.obj.clone().into()),
            realm.intrinsics.func.clone().into(),
        )?;
        
        realm.intrinsics.insert::<Self>(proto);
        
        Ok(())
        
    }


    #[allow(clippy::new_ret_no_self)]
    pub fn new(realm: &Realm) -> Res<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableUi {
                object: MutObject::with_proto(realm.intrinsics.get_of::<Self>()?.into()),
            }),
            ui:  RuntimeLifetime::empty(),
        };

        Ok(this.into_object())
    }
    
    pub fn update_ui<'a>(&self, ui: &'a mut egui::Ui) -> RuntimeLifetimeGuard<'a, egui::Ui> {
        self.ui.update(ui)
    }
}

#[props]
impl Ui {
    fn heading(&self, heading: String) -> Res {
        self.ui.with(move |ui| {
            ui.heading(heading);

            Ok(())
        })
    }
}