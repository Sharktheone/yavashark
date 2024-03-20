use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub struct CharIteratorReceiver<'a> {
    pub pos: Position,
    buffer: &'a UnsafeBuffer,
}

pub struct Position {
    pos: usize,
    line: usize,
    column: usize,
}

pub struct CharIteratorSender<'a> {
    buffer: &'a mut UnsafeBuffer,
}

impl Drop for CharIteratorSender<'_> {
    fn drop(&mut self) {
        self.buffer.end.store(true, Ordering::Relaxed);
        let res = self.buffer.other_dropped.compare_exchange(
            false,
            true,
            Ordering::Relaxed,
            Ordering::Relaxed,
        );

        if res.is_err() {
            let buffer = unsafe { Box::from_raw(self.buffer as *mut _) };
            drop(buffer);
        }
    }
}

impl Drop for CharIteratorReceiver<'_> {
    fn drop(&mut self) {
        let res = self.buffer.other_dropped.compare_exchange(
            false,
            true,
            Ordering::Relaxed,
            Ordering::Relaxed,
        );

        if res.is_err() {
            let buffer =
                unsafe { Box::from_raw(self.buffer as *const UnsafeBuffer as *mut UnsafeBuffer) };
            drop(buffer);
        }
    }
}

struct UnsafeBuffer {
    buffer: Box<[u8]>,
    //readonly
    size: usize,
    //only modified by the sender and read by both
    write_pos: AtomicUsize,
    //only modified by the receiver and read by both
    read_pos: AtomicUsize,
    end: AtomicBool,
    other_dropped: AtomicBool, //modified by either sender or receiver and read by the other (compare_exchange)
}

impl UnsafeBuffer {
    fn new(size: usize) -> Self {
        Self {
            buffer: vec![0; size].into_boxed_slice(),
            size,
            write_pos: AtomicUsize::new(0),
            read_pos: AtomicUsize::new(usize::MAX), //no one will have a script that is 17,179,869,184GB long (17 exabytes)
            end: AtomicBool::new(false),
            other_dropped: AtomicBool::new(false),
        }
    }
}

struct CharIterator;

impl CharIterator {
    #[allow(clippy::new_ret_no_self)]
    fn new<'send, 'recv>() -> Option<(CharIteratorSender<'send>, CharIteratorReceiver<'recv>)> {
        let buffer = Box::new(UnsafeBuffer::new(1024));
        let mut buffer = NonNull::new(Box::into_raw(buffer))?;

        let sender = CharIteratorSender {
            buffer: unsafe { buffer.as_mut() },
        };

        let receiver = CharIteratorReceiver {
            pos: Position {
                pos: 0,
                line: 1,
                column: 1,
            },
            buffer: unsafe { buffer.as_ref() },
        };

        Some((sender, receiver))
    }

    fn from_string<'recv>(s: String) -> anyhow::Result<CharIteratorReceiver<'recv>> {
        CharIteratorReceiver::try_from(s)
    }
}

impl<'a> TryFrom<String> for CharIteratorReceiver<'a> {
    type Error = anyhow::Error;
    fn try_from(s: String) -> Result<CharIteratorReceiver<'a>, Self::Error> {
        let len = s.len();
        let buffer = s.into_bytes();
        let buffer = buffer.into_boxed_slice();
        let buffer = Box::new(UnsafeBuffer {
            buffer,
            read_pos: AtomicUsize::new(0),
            write_pos: AtomicUsize::new(len),
            size: len + 1, // +1 because we need to point write_pos to the next byte, however it's not allocated
            end: AtomicBool::new(true),
            other_dropped: AtomicBool::new(true), //we don't have the other side
        });
        let Some(buffer) = NonNull::new(Box::into_raw(buffer)) else {
            return Err(anyhow::anyhow!(
                "Failed to allocate buffer for CharIteratorReceiver"
            ));
        };

        let receiver = CharIteratorReceiver {
            pos: Position {
                pos: 0,
                line: 1,
                column: 1,
            },
            buffer: unsafe { buffer.as_ref() },
        };

        Ok(receiver)
    }
}

impl<'a> TryFrom<&str> for CharIteratorReceiver<'a> {
    type Error = anyhow::Error;
    fn try_from(s: &str) -> Result<CharIteratorReceiver<'a>, Self::Error> {
        let buffer = Box::new(UnsafeBuffer {
            buffer: s.to_string().into_bytes().into_boxed_slice(),
            size: s.len() + 1, // +1 because we need to point write_pos to the next byte, however it's not allocated
            read_pos: AtomicUsize::new(0),
            write_pos: AtomicUsize::new(s.len()),
            end: AtomicBool::new(true),
            other_dropped: AtomicBool::new(true), // we don't have the other side
        });
        let Some(buffer) = NonNull::new(Box::into_raw(buffer)) else {
            return Err(anyhow::anyhow!(
                "Failed to allocate buffer for CharIteratorReceiver"
            ));
        };

        let receiver = CharIteratorReceiver {
            pos: Position {
                pos: 0,
                line: 1,
                column: 1,
            },
            buffer: unsafe { buffer.as_ref() },
        };

        Ok(receiver)
    }
}

impl CharIteratorReceiver<'_> {
    #[inline(always)]
    fn next_with_pos(&mut self, read_pos: usize) -> Option<u8> {
        loop {
            if read_pos == self.buffer.write_pos.load(Ordering::Relaxed) {
                if self.buffer.end.load(Ordering::Relaxed) {
                    return None;
                } else {
                    std::hint::spin_loop();
                }
            } else {
                let byte = self.buffer.buffer[read_pos];
                self.buffer
                    .read_pos
                    .store((read_pos + 1) % self.buffer.size, Ordering::Relaxed);
                return Some(byte);
            }
        }
    }
}

impl Iterator for CharIteratorReceiver<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        let read_pos = self.buffer.read_pos.load(Ordering::Relaxed);

        self.next_with_pos(read_pos % usize::MAX)
    }
}

impl CharIteratorSender<'_> {
    pub fn push(&mut self, byte: u8) {
        let write_pos = self.buffer.write_pos.load(Ordering::Relaxed);
        self.push_with_pos(byte, write_pos);
    }

    #[inline(always)]
    fn push_with_pos(&mut self, byte: u8, write_pos: usize) {
        if write_pos == self.buffer.read_pos.load(Ordering::Relaxed) {
            if self.buffer.end.load(Ordering::Relaxed) {
                return;
            }

            std::hint::spin_loop();
            self.push_with_pos(byte, write_pos);
        } else {
            self.buffer.buffer[write_pos] = byte;
            self.buffer
                .write_pos
                .store((write_pos + 1) % self.buffer.size, Ordering::Relaxed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_iterator() {
        let (mut sender, mut receiver) = CharIterator::new().unwrap();
        sender.push(b'a');
        sender.push(b'b');
        sender.push(b'c');
        assert_eq!(receiver.next(), Some(b'a'));
        assert_eq!(receiver.next(), Some(b'b'));
        assert_eq!(receiver.next(), Some(b'c'));
    }

    #[test]
    fn test_char_iterator_from_string() {
        let mut receiver = CharIterator::from_string("abc".to_string()).unwrap();
        assert_eq!(receiver.next(), Some(b'a'));
        assert_eq!(receiver.next(), Some(b'b'));
        assert_eq!(receiver.next(), Some(b'c'));
        assert_eq!(receiver.next(), None);
    }

    #[test]
    fn test_char_iterator_from_str() {
        let mut receiver = CharIterator::from_string("abc".to_string()).unwrap();
        assert_eq!(receiver.next(), Some(b'a'));
        assert_eq!(receiver.next(), Some(b'b'));
        assert_eq!(receiver.next(), Some(b'c'));
        assert_eq!(receiver.next(), None);
    }
}
