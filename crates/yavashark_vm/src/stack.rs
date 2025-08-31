use yavashark_env::Value;

#[derive(Debug, Clone)]
pub struct Stack {
    stack: Vec<Value>,
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

impl Stack {
    #[must_use]
    pub const fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    pub fn pop(&mut self) -> Option<Value> {
        self.stack.pop()
    }

    #[must_use]
    pub fn peek(&self) -> Option<&Value> {
        self.stack.last()
    }

    #[must_use]
    pub fn get(&self, idx: usize) -> Option<&Value> {
        self.stack.get(idx)
    }

    pub fn set(&mut self, idx: usize, value: Value) {
        if idx >= self.stack.len() {
            self.stack.resize(idx + 1, Value::Undefined);
        }
        self.stack[idx] = value;
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

    #[must_use]
    pub const fn len(&self) -> usize {
        self.stack.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
}
