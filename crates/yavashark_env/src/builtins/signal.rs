#![allow(unused)]

mod computed;
mod state;

use crate::builtins::signal::computed::{Computed, ComputedProtoObj};
use crate::builtins::signal::state::State;
use crate::{Object, ObjectHandle, Realm, Res};
use std::cell::RefCell;
use yavashark_garbage::Gc;
use crate::value::BoxedObj;

pub struct Protos {
    pub state: ObjectHandle,
    pub computed: ObjectHandle,
}

pub fn get_signal(
    obj_proto: ObjectHandle,
    func_proto: ObjectHandle,
) -> Res<(ObjectHandle, Protos)> {
    let obj = Object::with_proto(obj_proto.clone().into());

    let state = State::initialize_proto(
        Object::raw_with_proto(obj_proto.clone().into()),
        func_proto.clone().into(),
    )?;

    let proto = ComputedProtoObj {
        obj: Object::raw_with_proto(obj_proto.into()),
        current_dep: RefCell::default(),
    };

    let computed = Computed::initialize_proto(proto, func_proto.into())?;

    let state_constructor = state.get_property(&"constructor".into())?.value;
    let computed_constructor = computed.get_property(&"constructor".into())?.value;

    obj.define_property("State".into(), state_constructor);

    obj.define_property("Computed".into(), computed_constructor);

    let protos = Protos { state, computed };

    Ok((obj, protos))
}

pub fn notify_dependent(dep: &ObjectHandle, realm: &mut Realm) -> Res<()> {
    let Some(computed) = dep.downcast::<Computed>() else {
        return Ok(());
    };

    computed.dirty.set(true);

    Ok(())
}

pub const fn make_dependent(dep: &ObjectHandle, realm: &mut Realm) -> Res<()> {
    //TODO

    Ok(())
}
