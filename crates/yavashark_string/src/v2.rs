pub struct YSString {
    ptr: *const (),
    len: u32,
    ptr_offset: u32,
    len_offset: u32,
    ty: Type,
}

enum Type {
    Ascii,
    Wtf16,
    Rope,
}

enum Storage {
    Inline,
    Rc,
    Rope,
    Static,
}
