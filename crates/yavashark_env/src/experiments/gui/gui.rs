use crate::conversion::{downcast_obj, FromValueOutput};
use crate::experiments::gui::ui::Ui;
use crate::value::{BoxedObj, Obj};
use crate::{Error, MutObject, Object, ObjectHandle, Realm, Res};
use eframe::{App, Frame, NativeOptions};
use egui::{Context, ViewportCommand};
use std::any::TypeId;
use std::cell::RefCell;
use std::error;
use std::fmt::Display;
use std::rc::Rc;
use yavashark_garbage::OwningGcGuard;
use yavashark_macro::{object, props};

#[object]
#[derive(Debug)]
pub struct Gui {
    name: String,
    h: f32,
    w: f32,
}

impl Gui {
    pub fn init(realm: &mut Realm) -> Res {
        let proto = Self::initialize_proto(
            Object::raw_with_proto(realm.intrinsics.obj.clone()),
            realm.intrinsics.func.clone().into(),
            realm,
        )?;

        realm.intrinsics.other.insert(TypeId::of::<Self>(), proto);

        Ok(())
    }

    #[allow(clippy::new_ret_no_self)]
    pub fn new(realm: &Realm, name: String, w: f32, h: f32) -> Res<ObjectHandle> {
        let this = Self {
            inner: RefCell::new(MutableGui {
                object: MutObject::with_proto(realm.intrinsics.get_of::<Self>()?),
            }),
            name,
            w,
            h,
        };

        Ok(this.into_object())
    }
}

#[props]
impl Gui {
    #[constructor]
    pub fn construct(name: String, h: f32, w: f32, #[realm] realm: &Realm) -> Res<ObjectHandle> {
        Self::new(realm, name, h, w)
    }

    pub fn run(&self, f: ObjectHandle, #[realm] realm: &mut Realm) -> Res {
        if !f.is_callable() {
            return Err(Error::ty("passed non function value to Gui.run"));
        }

        let opts = NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([self.w, self.h]),
            ..Default::default()
        };

        let error = Rc::new(RefCell::new(None));
        let error2 = Rc::clone(&error);

        // let (realm_ref, _) = RuntimeLifetime::new(realm);

        // let mut func = move |ctx, frame| {
        //     egui::CentralPanel::default().show(ctx, |ui| {
        //         ui_ref.update_ui(ui);
        //
        //         if let Err(e) = realm_ref.with(|realm| {
        //             let global = realm.global.clone();
        //             f.call(realm, vec![ui_ob.clone().into()], global.into())
        //         }) {
        //             *error.borrow_mut() = Some(e);
        //             ctx.send_viewport_cmd(ViewportCommand::Close);
        //         }
        //     });
        // };
        //
        // let

        eframe::run_native(
            &self.name,
            opts,
            Box::new(|_| {
                Ok(Box::new(GuiApp::new(realm, error, f).map_err(|e| {
                    *error2.borrow_mut() = Some(e);
                    GuiCreationError
                })?))
            }),
        )?;

        // eframe::run_simple_native(&self.name, opts, func).unwrap();

        if let Some(error) = error2.borrow_mut().take() {
            return Err(error);
        }

        Ok(())
    }
}

pub struct GuiApp<'a> {
    realm: &'a mut Realm,
    ui: ObjectHandle,
    ui_ref: OwningGcGuard<'a, BoxedObj, Ui>,
    error: Rc<RefCell<Option<Error>>>,
    f: ObjectHandle,
}

impl<'a> GuiApp<'a> {
    fn new(realm: &'a mut Realm, error: Rc<RefCell<Option<Error>>>, f: ObjectHandle) -> Res<Self> {
        let ui = Ui::new(realm)?;
        let ui_ref = downcast_obj::<Ui>(ui.clone().into())?;

        Ok(Self {
            realm,
            ui,
            ui_ref,
            error,
            f,
        })
    }
}

impl App for GuiApp<'_> {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let x = self.ui_ref.update_ui(ui);

            // if let Err(e) = realm_ref.with(|realm| {
            //     let global = realm.global.clone();
            //     f.call(realm, vec![ui_ob.clone().into()], global.into())
            // }) {
            //     *error.borrow_mut() = Some(e);
            //     ctx.send_viewport_cmd(ViewportCommand::Close);
            // }

            let global = self.realm.global.clone();

            if let Err(e) = self
                .f
                .call(vec![self.ui.clone().into()], global.into(), self.realm)
            {
                *self.error.borrow_mut() = Some(e);
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }

            drop(x);
        });
    }
}

#[derive(Debug)]
pub struct GuiCreationError;

impl error::Error for GuiCreationError {}

impl Display for GuiCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to create GUI")
    }
}
