use bitflags::bitflags;

#[derive(Clone, Debug)]
pub struct Metadata {
    negative: Option<Negative>,
    includes: Vec<String>,
    flags: Flags,
    locale: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Negative {
    phase: NegativePhase,
    ty: String,
}

#[derive(Clone, Debug)]
pub enum NegativePhase {
    Parse,
    Resolution,
    Runtime,
}

bitflags! {
    #[derive(Clone, Debug)]
    struct Flags: u16 {
        const ONLY_STRICT = 1 << 0;
        const NO_STRICT = 1 << 1;
        const MODULE = 1 << 2;
        const RAW = 1 << 3;
        const ASYNC = 1 << 4;
        const GENERATED = 1 << 5;
        const CAN_BLOCK_IS_FALSE = 1 << 6;
        const CAN_BLOCK_IS_TRUE = 1 << 7;
        const NON_DETERMINISTIC = 1 << 8;
    }

}
