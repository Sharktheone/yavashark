use std::cell::{Cell, RefCell};

pub struct CharIterator {
    position: RefCell<Position>,
    length: usize,
    buffer: RefCell<Vec<u8>>,
    finished_streaming: Cell<bool>,
}


struct Position {
    pos: usize,
    line: usize,
    column: usize,
}

impl CharIterator {
    fn new(buffer: Vec<u8>) -> Self {
        Self {
            position: RefCell::new(Position {
                pos: 0,
                line: 1,
                column: 1,
            }),
            length: buffer.len(),
            buffer: RefCell::new(buffer),
            finished_streaming: Cell::new(false),
        }
    }
}


impl From<String> for CharIterator {
    fn from(s: String) -> Self {
        Self::new(s.into_bytes())
    }
}

impl CharIterator {
    fn next(&self) -> Option<u8> {
        if self.finished_streaming {
            return None;
        }

        self.get_next_checked_eof()
    }

    #[inline(always)]
    fn get_next_checked_eof(&self) -> Option<u8> {
        let pos = self.position.try_borrow_mut().ok()?;

        if let Some(byte) = self.buffer.try_borrow().ok()?.get(pos.pos) {
            let byte = *byte;
            pos.pos += 1;

            if byte == b'\n' {
                pos.line += 1;
                pos.column = 1;
            } else {
                pos.column += 1;
            }

            Some(byte)
        } else if self.finished_streaming.get() {
            None
        } else {
            drop(pos); //TODO: is this necessary?
            self.get_next_checked_eof()
        }
    }
}