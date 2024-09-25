
#[test]
fn test() {
    use swc_common::BytePos;
    use swc_ecma_parser::{Parser, StringInput, Syntax, TsConfig};
    let src = r#"
    `hello ${world}`
    "#;

    let c = TsConfig {
        ..Default::default()
    };

    let input = StringInput::new(src, BytePos(0), BytePos(src.len() as u32 - 1));

    let mut p = Parser::new(Syntax::Typescript(c), input, None);
    let prog = p.parse_program().unwrap();

    println!("{:#?}", prog);
}

#[test]
fn a() {
    use rand::random;
    struct Test {
        value: i32,
    }

    struct TestClone {
        value: i32,
    }

    struct BorrowedTest<'a> {
        test: &'a Test,
    }

    impl Test {
        fn borrow(&self) -> Option<BorrowedTest> {
            let rand: bool = random();

            if rand {
                return None;
            }

            Some(BorrowedTest { test: self })
        }
    }

    impl TestClone {
        fn cmp(&self, other: &Test) -> bool {
            self.value == other.value
        }
    }

    impl BorrowedTest<'_> {
        fn do_something(&self) -> TestClone {
            TestClone {
                value: self.test.value,
            }
        }
    }

    let test = Test { value: 10 };

    if let Some(t) = test.borrow() {
        let test_clone = t.do_something();

        test_clone.cmp(&test);
    }
}
