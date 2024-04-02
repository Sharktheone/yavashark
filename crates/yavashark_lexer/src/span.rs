use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    pub fn contains(&self, other: &Self) -> bool {
        self.start <= other.start && self.end >= other.end
    }

    pub fn contains_pos(&self, pos: usize) -> bool {
        self.start <= pos && self.end >= pos
    }

    pub(crate) fn extend(&mut self) {
        self.end += 1;
    }


    pub(crate) fn extend_by(&mut self, amount: usize) {
        self.end += amount;
    }

    pub(crate) fn shrink(&mut self) {
        self.end -= 1;
    }

    pub(crate) fn shrink_by(&mut self, amount: usize) {
        self.end -= amount;
    }
    
    
    pub(crate) fn replace(&mut self) -> Self {
        std::mem::replace(self, Span::new(0, 0))
    }

}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}..#{}", self.start, self.end)
    }
}
