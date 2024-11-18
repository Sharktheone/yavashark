use std::io;
use std::io::Write;

struct Repl {
    callback: Box<dyn Fn(&str)>,
}

const OPEN_BRACES: &[char] = &['(', '{', '['];
const CLOSE_BRACES: &[char] = &[')', '}', ']'];

impl Repl {
    fn new(callback: Box<dyn Fn(&str)>) -> Self {
        Self { callback }
    }

    fn run(&self) {
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
            
            let offset = input.len() - 1;

            io::stdin().read_line(&mut input).unwrap();
            
            for i in input[offset..].chars() {
                if OPEN_BRACES.contains(&i) {
                    braces_open += 1;
                }
                
                if CLOSE_BRACES.contains(&i) {
                    braces_open -= 1;
                }
                
            }
            
            
            if braces_open == 0 {
                (self.callback)(&input);
            }
        }
    }
}
