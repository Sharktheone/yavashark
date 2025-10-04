#![allow(unused)]

mod computed;
mod state;

use crate::builtins::signal::computed::{Computed, ComputedProtoObj};
use crate::builtins::signal::state::State;
use crate::value::BoxedObj;
use crate::{Object, ObjectHandle, Realm, Res, Value};
use std::cell::RefCell;
use yavashark_garbage::Gc;

pub struct Protos {
    pub state: ObjectHandle,
    pub computed: ObjectHandle,
}

pub fn get_signal(
    obj_proto: ObjectHandle,
    func_proto: ObjectHandle,
    realm: &mut Realm
) -> Res<(ObjectHandle, Protos)> {
    let obj = Object::with_proto(obj_proto.clone());

    let state = State::initialize_proto(
        Object::raw_with_proto(obj_proto.clone()),
        func_proto.clone().into(),
        realm,
    )?;

    let proto = ComputedProtoObj {
        obj: Object::raw_with_proto(obj_proto),
        current_dep: RefCell::default(),
    };

    let computed = Computed::initialize_proto(proto, func_proto.into(), realm)?;

    let state_constructor = state.resolve_property("constructor", realm)?.unwrap_or(Value::Undefined);
    let computed_constructor = computed.resolve_property("constructor", realm)?.unwrap_or(Value::Undefined);

    obj.define_property("State".into(), state_constructor, realm);

    obj.define_property("Computed".into(), computed_constructor, realm);

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
