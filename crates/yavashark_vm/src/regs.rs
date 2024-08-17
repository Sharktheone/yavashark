use yavashark_env::Value;

pub const NUM_REGS: usize = 32;

pub struct Registers {
    regs: [Value; NUM_REGS],
}
