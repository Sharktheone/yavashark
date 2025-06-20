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

    obj.define_variable("fs".into(), Fs::new(realm)?.into())?;
    obj.define_variable("http".into(), Http::new(realm)?.into())?;
    obj.define_variable("tcp".into(), Tcp::new(realm)?.into())?;
    obj.define_variable("io".into(), io::Io::new(realm)?.into())?;
    obj.define_variable("time".into(), time::Timer::new(realm)?.into())?;
    obj.define_variable("setTimeout".into(), timers::get_set_timeout(realm).into())?;

    #[cfg(feature = "gui")]
    gui::init(realm)?;

    Ok(())
}
