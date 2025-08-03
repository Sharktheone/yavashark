use crate::{ObjectHandle, Realm, Res};

pub trait Intrinsic {
    fn initialize(&self, realm: &mut Realm) -> Res;
    fn get_prototype(&self, realm: &Realm) -> ObjectHandle;
    
    fn setup_global(&self, realm: &mut Realm, global: &ObjectHandle) -> Res {
        Ok(())
    }
}