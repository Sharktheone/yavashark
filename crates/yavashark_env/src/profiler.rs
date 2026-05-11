use crate::Realm;
#[cfg(feature = "profiler")]
use std::time::Instant;

#[cfg(feature = "profiler")]
pub use yavashark_profiler::{FileProfileWriter, FrameId, Profile};

#[cfg(feature = "profiler")]
pub fn profile_call<T>(
    realm: &mut Realm,
    fn_name: impl FnOnce() -> String,
    f: impl FnOnce(&mut Realm) -> T,
) -> T {
    let fn_name = fn_name();

    let start = Instant::now();
    let frame_id = realm.profile_add_frame(fn_name, start);

    let result = f(realm);

    realm.profile_end_frame(frame_id, Instant::now());

    result
}

#[cfg(not(feature = "profiler"))]
pub fn profile_call<T>(
    realm: &mut Realm,
    _fn_name: impl FnOnce() -> String,
    f: impl FnOnce(&mut Realm) -> T,
) -> T {
    f(realm)
}
