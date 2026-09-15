use std::collections::HashMap;
use std::ptr::NonNull;

type GC<T> = NonNull<T>;

struct Shape {
    map: HashMap<String, u32>,
    prototype: GC<()>,
}
