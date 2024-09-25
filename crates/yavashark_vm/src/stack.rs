use yavashark_env::Value;

pub struct Stack {
    stack: Vec<Value>,
}

impl Stack {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    pub fn pop(&mut self) -> Option<Value> {
        self.stack.pop()
    }

    pub fn peek(&self) -> Option<&Value> {
        self.stack.last()
    }

    pub fn get(&self, idx: usize) -> Option<&Value> {
        self.stack.get(idx)
    }

    pub fn pop_n(&mut self, n: usize) -> Vec<Value> {
        let mut values = Vec::with_capacity(n);
        for _ in 0..n {
            if let Some(value) = self.pop() {
                values.push(value);
            } else {
                break;
            }
        }
        values
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }
}
