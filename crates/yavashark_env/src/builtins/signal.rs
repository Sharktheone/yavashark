mod computed;
mod state;

use yavashark_garbage::Gc;
use yavashark_value::BoxedObj;
use crate::{Object, ObjectHandle, Realm, Res};

pub struct Protos {
    pub state: ObjectHandle,
    pub computed: ObjectHandle,
}

pub fn get_signal(obj_proto: ObjectHandle, func_proto: ObjectHandle) -> Res<(ObjectHandle, Protos)> {
    let obj = Object::with_proto(obj_proto.clone().into());

    todo!()

}


pub fn notify_dependent(dep: Gc<BoxedObj<Realm>>, realm: &mut Realm) -> Res<()> {
    Ok(())
}
