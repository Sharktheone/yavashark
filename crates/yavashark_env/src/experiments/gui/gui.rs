use crate::conversion::FromValueOutput;
use crate::experiments::gui::runtime_lifetime::RuntimeLifetime;
use crate::experiments::gui::ui::Ui;
use crate::{Error, MutObject, Object, ObjectHandle, Realm, Res};
use eframe::{App, Frame, NativeOptions};
use egui::{Context, ViewportCommand};
use std::any::TypeId;
use std::cell::RefCell;
use std::rc::Rc;
use yavashark_garbage::OwningGcGuard;
use yavashark_macro::{object, props};
use yavashark_value::{BoxedObj, Obj};

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
            Object::raw_with_proto(realm.intrinsics.obj.clone().into()),
            realm.intrinsics.func.clone().into(),
        )?;

        realm.intrinsics.other.insert(TypeId::of::<Self>(), proto);

        Ok(())
    }

    #[allow(clippy::new_ret_no_self)]
    pub fn new(realm: &Realm, name: String, w: f32, h: f32) -> Res<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableGui {
                object: MutObject::with_proto(realm.intrinsics.get_of::<Self>()?.into()),
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
        if !f.is_function() {
            return Err(Error::ty("passed non function value to Gui.run"));
        }

        let opts = NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([self.w, self.h]),
            ..Default::default()
        };

        let mut error = Rc::new(RefCell::new(None));
        let error2 = Rc::clone(&error);


        let (realm_ref, _) = RuntimeLifetime::new(realm);

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

        eframe::run_native(&self.name, opts, Box::new(|_| {
            Ok(Box::new(GuiApp::new(realm, error, f).unwrap()))
        })).unwrap();

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
    ui_ref: OwningGcGuard<'a, BoxedObj<Realm>, Ui>,
    error: Rc<RefCell<Option<Error>>>,
    f: ObjectHandle,
}

impl<'a> GuiApp<'a> {
    fn new(realm: &'a mut Realm,  error: Rc<RefCell<Option<Error>>>, f: ObjectHandle) -> Res<GuiApp> {
        let ui = Ui::new(realm)?;
        let ui_ref = <&Ui as FromValueOutput>::from_value_out(ui.clone().into())?;
        
        
        
        Ok(Self {
            realm,
            ui,
            ui_ref,
            error,
            f
        })
    }
}

impl<'a> App for GuiApp<'a> {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
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
                .call(self.realm, vec![self.ui.clone().into()], global.into())
            {
                *self.error.borrow_mut() = Some(e);
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
            
            drop(x);
        });
    }
}
