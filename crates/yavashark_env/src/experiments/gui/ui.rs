use super::jswidget::DynWidget;
use crate::experiments::gui::runtime_lifetime::{RuntimeLifetime, RuntimeLifetimeGuard};
use crate::value::Obj;
use crate::{Error, MutObject, Object, ObjectHandle, Realm, Res, Value};
use std::any::TypeId;
use std::cell::RefCell;
use std::fmt::Debug;
use yavashark_macro::{object, props};

#[object]
pub struct Ui {
    ui: RuntimeLifetime<egui::Ui>,
}

impl Debug for Ui {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Ui")
    }
}

impl Ui {
    pub fn init(realm: &mut Realm) -> Res {
        let proto = Self::initialize_proto(
            Object::raw_with_proto(realm.intrinsics.obj.clone()),
            realm.intrinsics.func.clone(),
        )?;

        realm.intrinsics.insert::<Self>(proto);

        Ok(())
    }

    #[allow(clippy::new_ret_no_self)]
    pub fn new(realm: &Realm) -> Res<ObjectHandle> {
        let this = Self {
            inner: RefCell::new(MutableUi {
                object: MutObject::with_proto(realm.intrinsics.get_of::<Self>()?),
            }),
            ui: RuntimeLifetime::empty(),
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

    fn label(&self, label: String) -> Res {
        self.ui.with(move |ui| {
            ui.label(label);

            Ok(())
        })
    }

    fn horizontal(&self, #[this] this: Value, f: ObjectHandle, #[realm] realm: &mut Realm) -> Res {
        self.ui.with(|ui| {
            ui.horizontal(|ui| {
                let x = self.ui.update(ui);
                let global = realm.global.clone().into();
                f.call(realm, vec![this], global)?;

                drop(x);

                Ok(())
            })
            .inner
        })
    }

    fn text_edit_single_line(&self, mut test: String) -> Res<String> {
        self.ui.with(|ui| {
            ui.text_edit_singleline(&mut test);

            Ok(())
        })?;

        Ok(test)
    }

    fn add(&self, widget: ObjectHandle) -> Res {
        let widget = unsafe {
            let widget = widget
                .inner_downcast(TypeId::of::<DynWidget>())
                .ok_or(Error::ty("Expected Widget"))?
                .cast::<DynWidget>();
            Box::from_raw(widget.as_ptr())
        };

        self.ui.with(|ui| {
            unsafe {
                widget.get_widget().ui(ui)?;
            }

            Ok(())
        })
    }

    fn button(&self, label: String) -> Res<bool> {
        self.ui.with(|ui| Ok(ui.button(label).clicked()))
    }
}
