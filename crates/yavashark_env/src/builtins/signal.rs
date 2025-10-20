#![allow(unused)]

mod computed;
mod state;

use crate::value::{BoxedObj, Obj};
use crate::{Object, ObjectHandle, Realm, Res, Value};
use std::cell::RefCell;
use yavashark_garbage::Gc;

pub use computed::*;
pub use state::*;
use crate::partial_init::Initializer;
use crate::realm::Intrinsic;


pub struct Signal;

impl Initializer<ObjectHandle> for Signal {
    fn initialize(realm: &mut Realm) -> Res<ObjectHandle> {
        get_signal(realm)
    }
}


pub fn get_signal(
    realm: &mut Realm,
) -> Res<ObjectHandle> {
    let obj = Object::with_proto(realm.intrinsics.obj.clone());

    let intrinsics = realm.intrinsics.clone_public();

    let computed = intrinsics.signal_computed.get(realm)?;
    let proto = ComputedProtoObj {
        obj: Object::raw_with_proto(realm.intrinsics.obj.clone()),
        current_dep: RefCell::default(),
    };


    computed.set_prototype(proto.into_object().into(), realm)?; //TODO find a better way for this

    let computed_constructor = computed
        .resolve_property("constructor", realm)?
        .unwrap_or(Value::Undefined);

    obj.define_property("Computed".into(), computed_constructor, realm);

    let state_constructor = intrinsics
        .signal_state
        .get(realm)?
        .resolve_property("constructor", realm)?
        .unwrap_or(Value::Undefined);

    obj.define_property("State".into(), state_constructor, realm);


    Ok(obj)
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
