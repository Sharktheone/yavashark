use std::io;
use std::io::Write;

struct Repl {
    callback: Box<dyn Fn(String)>,
}

const OPEN_BRACES: &[char] = &['(', '{', '['];
const CLOSE_BRACES: &[char] = &[')', '}', ']'];

impl Repl {
    fn new(callback: Box<dyn Fn(String)>) -> Self {
        Self { callback }
    }

    fn run(&self) {
        let braces_open = 0u8;

        let mut input = String::new();

        loop {
            if braces_open == 0 {
                print!("> ");
                let _ = io::stdout().flush();

                input.clear();
            } else {
                print!("...");
                let _ = io::stdout().flush();
            }

            io::stdin().read_line(&mut input).unwrap();
        }
    }
}
