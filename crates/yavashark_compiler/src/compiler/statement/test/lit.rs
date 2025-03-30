use num_traits::Zero;
use swc_ecma_ast::Lit;
use yavashark_bytecode::jmp::Test;
use crate::Compiler;

impl Compiler {
    pub fn test_lit(&mut self, l: &Lit) -> Test {
        #[allow(clippy::match_same_arms)]
        match l {
            Lit::Str(str) => {
                if str.is_empty() {
                    Test::Never
                } else {
                    Test::Unconditional
                }
                
            }
            Lit::Bool(bool) => {
                if bool.value {
                    Test::Unconditional
                } else {
                    Test::Never
                }
                
            }
            Lit::Null(_) => Test::Never,
            Lit::Num(number) => {
                if number.value == 0.0 {
                    Test::Never
                } else {
                    Test::Unconditional
                }
                
            }
            Lit::BigInt(big_int) => {
                if big_int.value.is_zero() {
                    Test::Never
                } else {
                    Test::Unconditional
                }
                
            }
            Lit::Regex(_) => Test::Unconditional,
            _ => Test::Unconditional,
        }
    }
}