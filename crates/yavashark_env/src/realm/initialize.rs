use crate::{ObjectHandle, Realm, Res};

#[allow(unused_variables)]
pub trait Intrinsic {
    fn initialize(&self, realm: &mut Realm) -> Res;
    fn get_prototype(&self, realm: &Realm) -> ObjectHandle;

    fn setup_global(&self, realm: &mut Realm) -> Res {
        Ok(())
    }
}
