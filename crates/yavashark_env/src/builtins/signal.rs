mod computed;
mod state;

use crate::{Object, ObjectHandle, Realm, Res};
use yavashark_garbage::Gc;
use yavashark_value::BoxedObj;

pub struct Protos {
    pub state: ObjectHandle,
    pub computed: ObjectHandle,
}

pub fn get_signal(
    obj_proto: ObjectHandle,
    func_proto: ObjectHandle,
) -> Res<(ObjectHandle, Protos)> {
    let obj = Object::with_proto(obj_proto.clone().into());

    let protos = Protos {
        state: Object::null(),
        computed: Object::null(),
    };

    Ok((obj, protos))
}

pub fn notify_dependent(dep: Gc<BoxedObj<Realm>>, realm: &mut Realm) -> Res<()> {
    Ok(())
}
