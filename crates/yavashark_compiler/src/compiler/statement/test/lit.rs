use crate::Compiler;
use num_traits::Zero;
use swc_ecma_ast::Lit;
use yavashark_bytecode::jmp::Test;

impl Compiler {
    pub fn test_lit(&mut self, l: &Lit) -> Test {
        #[allow(clippy::match_same_arms)]
        match l {
            Lit::Str(str) => {
                if str.is_empty() {
                    Test::Always
                } else {
                    Test::Never
                }
            }
            Lit::Bool(bool) => {
                if bool.value {
                    Test::Never
                } else {
                    Test::Always
                }
            }
            Lit::Null(_) => Test::Always,
            Lit::Num(number) => {
                if number.value == 0.0 {
                    Test::Always
                } else {
                    Test::Never
                }
            }
            Lit::BigInt(big_int) => {
                if big_int.value.is_zero() {
                    Test::Always
                } else {
                    Test::Never
                }
            }
            Lit::Regex(_) => Test::Never,
            Lit::JSXText(_) => Test::Never,
        }
    }
}
