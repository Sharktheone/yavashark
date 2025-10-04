use crate::experiments::fs::Fs;
use crate::experiments::http::Http;
use crate::experiments::tcp::Tcp;
use crate::{ObjectHandle, Realm, Res};

mod fs;
#[cfg(feature = "gui")]
mod gui;
mod http;
mod io;
mod tcp;
mod time;
mod timers;

pub fn init(obj: &ObjectHandle, realm: &mut Realm) -> Res {
    let obj = obj.guard();

    obj.define_property_attributes("fs".into(), Fs::new(realm)?.into(), realm)?;
    obj.define_property_attributes("http".into(), Http::new(realm)?.into(), realm)?;
    obj.define_property_attributes("tcp".into(), Tcp::new(realm)?.into(), realm)?;
    obj.define_property_attributes("io".into(), io::Io::new(realm)?.into(), realm)?;
    obj.define_property_attributes("time".into(), time::Timer::new(realm)?.into(), realm)?;
    obj.define_property_attributes("setTimeout".into(), timers::get_set_timeout(realm).into(), realm)?;

    #[cfg(feature = "gui")]
    gui::init(realm)?;

    Ok(())
}
