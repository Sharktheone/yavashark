#![allow(unused)]

mod computed;
mod state;

use crate::value::{BoxedObj, Obj};
use crate::{Object, ObjectHandle, Realm, Res, Value};
use std::cell::RefCell;
use yavashark_garbage::Gc;

pub use computed::*;
pub use state::*;
use crate::realm::Intrinsic;

pub struct Protos {
    pub state: ObjectHandle,
    pub computed: ObjectHandle,
}

pub fn get_signal(
    realm: &mut Realm,
) -> Res<(ObjectHandle, Protos)> {
    let obj = Object::with_proto(realm.intrinsics.obj.clone());

    let state = State::initialize(
        realm,
    )?;

    let proto = ComputedProtoObj {
        obj: Object::raw_with_proto(realm.intrinsics.obj.clone()),
        current_dep: RefCell::default(),
    };

    let computed = Computed::initialize(realm)?;

    computed.set_prototype(proto.into_object().into(), realm)?;

    let state_constructor = state
        .resolve_property("constructor", realm)?
        .unwrap_or(Value::Undefined);
    let computed_constructor = computed
        .resolve_property("constructor", realm)?
        .unwrap_or(Value::Undefined);

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
