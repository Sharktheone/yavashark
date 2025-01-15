use crate::experiments::fs::Fs;
use crate::experiments::http::Http;
use crate::experiments::tcp::Tcp;
use crate::{ObjectHandle, Realm, Res};

mod fs;
mod http;
mod tcp;

pub fn init(obj: &ObjectHandle, realm: &Realm) -> Res {
    let obj = obj.get();

    obj.define_variable("fs".into(), Fs::new(realm)?.into())?;
    obj.define_variable("http".into(), Http::new(realm)?.into())?;
    obj.define_variable("tcp".into(), Tcp::new(realm)?.into())?;

    Ok(())
}
