use std::io;
use std::io::Write;

pub struct Repl {
    callback: Box<dyn FnMut(&str)>,
}

const OPEN_BRACES: &[char] = &['(', '{', '['];
const CLOSE_BRACES: &[char] = &[')', '}', ']'];

impl Repl {
    pub fn new(callback: Box<dyn FnMut(&str)>) -> Self {
        Self { callback }
    }

    pub fn run(&mut self) -> ! {
        let mut braces_open = 0u8;

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

            let offset = if input.is_empty() { 0 } else { input.len() - 1 };

            if let Err(e) = io::stdin().read_line(&mut input) {
                println!("Error: \n{e}");
                continue;
            }

            for i in input[offset..].chars() {
                if OPEN_BRACES.contains(&i) {
                    braces_open += 1;
                }

                if CLOSE_BRACES.contains(&i) && braces_open != 0 {
                    braces_open -= 1;
                }
            }

            if braces_open == 0 {
                (self.callback)(&input);
            }
        }
    }
}
