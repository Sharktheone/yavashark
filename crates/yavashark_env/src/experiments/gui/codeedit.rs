use super::jswidget::{DynWidget, JSWidget};
use crate::{Error, MutObject, Object, ObjectHandle, Realm, Res};
use egui::{Response, TextEdit, Ui, Widget};
use std::any::TypeId;
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::rc::Rc;
use yavashark_macro::props;
use yavashark_value::{MutObj, ObjectImpl};

#[derive(Debug)]
pub struct CodeEdit {
    lang: String,
    code: Rc<RefCell<String>>,
}

impl CodeEdit {
    pub fn new(lang: String, code: Rc<RefCell<String>>) -> Self {
        Self { lang, code }
    }
}

impl Widget for CodeEdit {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut theme =
            egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());
        ui.collapsing("Theme", |ui| {
            ui.group(|ui| {
                theme.ui(ui);
                theme.clone().store_in_memory(ui.ctx());
            });
        });

        let mut layouter = |ui: &egui::Ui, buf: &str, wrap_width: f32| {
            let mut layout_job = egui_extras::syntax_highlighting::highlight(
                ui.ctx(),
                ui.style(),
                &theme,
                buf,
                &self.lang,
            );
            layout_job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(layout_job))
        };

        let mut code = self.code.borrow_mut();
        let e = TextEdit::multiline(code.deref_mut())
            .font(egui::TextStyle::Monospace) // for cursor height
            .code_editor()
            .desired_rows(10)
            .lock_focus(true)
            .desired_width(f32::INFINITY)
            .layouter(&mut layouter);

        egui::ScrollArea::vertical().show(ui, |ui| ui.add(e)).inner
    }
}

#[derive(Debug)]
pub struct JSCodeEditor {
    mut_object: RefCell<MutObject>,
    editor: RefCell<Option<CodeEdit>>,
    code: Rc<RefCell<String>>,
}

impl JSCodeEditor {
    pub fn init(realm: &mut Realm) -> Res {
        let proto = Self::initialize_proto(
            Object::raw_with_proto(realm.intrinsics.obj.clone().into()),
            realm.intrinsics.func.clone().into(),
        )?;

        realm.intrinsics.other.insert(TypeId::of::<Self>(), proto);

        Ok(())
    }

    pub fn new(realm: &Realm, code: String, lang: String) -> Res<Self> {
        let code = Rc::new(RefCell::new(code));

        Ok(Self {
            mut_object: RefCell::new(MutObject::with_proto(
                realm.intrinsics.get_of::<Self>()?.into(),
            )),
            code: Rc::clone(&code),
            editor: RefCell::new(Some(CodeEdit::new(lang, code))),
        })
    }
}

impl JSWidget for JSCodeEditor {
    fn ui(&self, ui: &mut egui::Ui) -> Res {
        if let Some(ed) = self.editor.borrow_mut().take() {
            ui.add(ed);
        } else {
            return Err(Error::new("Widget already rendered"));
        }

        Ok(())
    }
}

impl ObjectImpl<Realm> for JSCodeEditor {
    type Inner = MutObject;

    fn get_wrapped_object(&self) -> impl DerefMut<Target = impl MutObj<Realm>> {
        self.mut_object.borrow_mut()
    }

    fn get_inner(&self) -> impl Deref<Target = Self::Inner> {
        self.mut_object.borrow()
    }

    fn get_inner_mut(&self) -> impl DerefMut<Target = Self::Inner> {
        self.mut_object.borrow_mut()
    }

    unsafe fn inner_downcast(&self, ty: TypeId) -> Option<NonNull<()>> {
        if ty == TypeId::of::<Self>() {
            Some(NonNull::from(self).cast())
        } else if ty == TypeId::of::<DynWidget>() {
            let d = Box::new(DynWidget(NonNull::from(self as &dyn JSWidget)));

            Some(NonNull::new_unchecked(Box::into_raw(d)).cast())
        } else {
            None
        }
    }
}

#[props]
impl JSCodeEditor {
    #[constructor]
    fn construct(#[realm] realm: &Realm, lang: String, code: String) -> Res<ObjectHandle> {
        Ok(Self::new(realm, code, lang)?.into_object())
    }

    #[prop("getCode")]
    fn get_code(&self) -> String {
        self.code.borrow().clone()
    }
}
