#![allow(dead_code)]
#![allow(unused)]
use core::panic;
use std::cmp;

const BUF_INIT_SIZE: usize = 32;

#[derive(Debug, Clone)]
pub struct GapBuffer {
    data: Box<[u8]>,
    gap_start: usize,
    gap_end: usize,
    cursor: usize,
}

/// Updating the gap to the cursor at every movement is very costly, so it should only be updated when
/// a critical operation is done: insertion and deletion
impl GapBuffer {
    pub fn new() -> Self {
        Self {
            data: Box::new([0; BUF_INIT_SIZE]),
            gap_start: 0,
            gap_end: BUF_INIT_SIZE,
            cursor: 0,
        }
    }

    pub fn get_text(&self) -> Result<String, std::string::FromUtf8Error> {
        let capacity = self.data.len() + self.gap_end - self.gap_start;
        let mut buffer = Vec::<u8>::with_capacity(capacity);
        //eprintln!("gap_start: {}. buflen: {}. gap_end: {}", self.gap_start, self.buffer_len(), self.gap_end);
        buffer.extend_from_slice(&self.data[0..self.gap_start]);
        buffer.extend_from_slice(&self.data[self.gap_end..self.data.len()]);
        String::from_utf8(buffer)
    }

    /// Inserts a string slice into the gap
    /// if the gap is big enough
    pub fn insert(&mut self, slice: &[u8]) {
        if self.gap_len() > slice.len() {
            let insert_space = &mut self.data[self.gap_start..self.gap_start + slice.len()];
            insert_space.copy_from_slice(slice);
            self.gap_start += slice.len();
        } else {
            self.grow_gap(slice);
        }
    }

    pub fn insert_char(&mut self, char: u8) {
        if self.gap_end - self.gap_start == 0 {
            self.grow_gap(&[char]);
        }
        self.data[self.gap_start] = char;
        self.gap_start += 1;
    }

    pub fn delete_backwards(&mut self, nchars: usize) {
        assert!(nchars <= self.gap_start);
        self.gap_start -= nchars;
    }

    pub fn delete_forwards(&mut self, nchars: usize) {
        assert!(nchars <= self.gap_end);
        self.gap_end += nchars;
    }
    
    /// the cursor behaves in a smaller buffer, the text without gap buffer
    pub fn move_gap_to_cursor(&mut self, cursor: usize) {
        if self.gap_start == cursor {
            return;
        }
        if self.buffer_len() - self.gap_len() < cursor {
            panic!("cursor out of bounds");
        }
        let new_gap_start = cursor;
        let new_gap_end = cursor + self.gap_len();

        let buffer = if self.gap_start > new_gap_start {
            self.move_gap_backwards(new_gap_start, new_gap_end)
        } else {
            self.move_gap_forwards(new_gap_start, new_gap_end)
        };

        self.gap_start = new_gap_start;
        self.gap_end = new_gap_end;
        self.data = buffer.into_boxed_slice(); 
    }

    pub fn clear_buffer_text(&mut self) {
        self.gap_start = 0;
        self.gap_end = self.buffer_len();
    }

    fn move_gap_backwards(&mut self, new_gap_start: usize, new_gap_end: usize) -> Vec<u8> {
        let left = &self.data[0..new_gap_start];
        let swap = &self.data[new_gap_start..self.gap_start];
        let right = &self.data[self.gap_end..self.buffer_len()];
        let mut buffer = Vec::<u8>::with_capacity(self.buffer_len());
        buffer.extend_from_slice(left);
        buffer.resize(new_gap_end - new_gap_start, 0);
        buffer.extend_from_slice(swap);
        buffer.extend_from_slice(right);
        buffer
    }

    fn move_gap_forwards(&mut self, new_gap_start: usize, new_gap_end: usize) -> Vec<u8> {
        let left = &self.data[0..self.gap_start];
        let swap = &self.data[self.gap_end..new_gap_end];
        let right = &self.data[new_gap_end..self.gap_len()];
        let mut buffer = Vec::<u8>::with_capacity(self.buffer_len());
        buffer.extend_from_slice(left);
        buffer.extend_from_slice(swap);
        buffer.resize(new_gap_end - new_gap_start, 0);
        buffer.extend_from_slice(right);
        buffer
    }

    fn gap_len(&self) -> usize {
        self.gap_end - self.gap_start
    }

    pub fn buffer_len(&self) -> usize {
        self.data.len()
    }

    pub fn text_len(&self) -> usize {
        self.data.len() - self.gap_len()
    }
    
    fn grow_gap(&mut self, slice: &[u8]) {
        let slice_len = slice.len();
        let new_gap_size = self.compute_new_gap_size(slice_len);
        //eprintln!("gap_start: {}. datalen: {}. gap_end: {}", self.gap_start, self.data.len(), self.gap_end);
        let new_capacity = {
            let pre_gap = self.gap_start;
            let post_gap = self.data.len() - self.gap_end;
            pre_gap + new_gap_size + slice_len + post_gap
        };
        let mut buffer = Vec::<u8>::with_capacity(new_capacity);
        buffer.extend_from_slice(&self.data[0..self.gap_start]);
        buffer.extend_from_slice(&slice);
        buffer.resize(
            self.gap_start + slice_len + new_gap_size + self.data.len() - self.gap_end,
            0,
        );
        buffer.extend_from_slice(&self.data[self.gap_end..self.data.len()]);
        self.gap_start += slice_len;
        self.gap_end = self.gap_start + new_gap_size;
        self.data = buffer.into_boxed_slice();
    }

    fn compute_new_gap_size(&self, slice_len: usize) -> usize {
        let five_percent = self.data.len() * 0.05 as usize;
        let next_pow_of_two = (self.gap_len() + slice_len).next_power_of_two();
        cmp::max(five_percent, next_pow_of_two)
    }
}

#[cfg(test)]
mod tests {
    use super::GapBuffer;

    #[test]
    fn insert_char() {
        // [one way of doing things]
        let buffer = "one way of doing things";
        let mut gap_buffer = GapBuffer::new();
        gap_buffer.insert(buffer.as_bytes());
        println!(
            "{} - {} + {}",
            gap_buffer.buffer_len(),
            gap_buffer.gap_start,
            gap_buffer.gap_end,
        );
        gap_buffer.insert_char('C' as u8);
        let text = gap_buffer.get_text().unwrap();
        print!("{}", text);
        assert!(text.contains('C'));
    }

    #[test]
    fn delete_one_backwards() {
        // [one way of doing things]
        let buffer = "one way of doing things";
        let mut gap_buffer = GapBuffer::new();
        gap_buffer.insert(buffer.as_bytes());
        gap_buffer.move_gap_to_cursor(gap_buffer.buffer_len() - gap_buffer.gap_len());
        println!(
            "buffer len:{}. gap start: {}. gap end: {}.",
            gap_buffer.buffer_len(),
            gap_buffer.gap_start,
            gap_buffer.gap_end,
        );
        gap_buffer.delete_backwards(1);
        let text = gap_buffer.get_text().unwrap();
        print!("{}", text);
        assert_eq!(text, "one way of doing thing");
    }
    
    #[test]
    fn delete_one_forwards() {
        // [one way of doing things]
        let buffer = "one way of doing things";
        let mut gap_buffer = GapBuffer::new();
        gap_buffer.insert(buffer.as_bytes());
        gap_buffer.move_gap_to_cursor(0);
        println!(
            "buffer len:{}. gap start: {}. gap end: {}.",
            gap_buffer.buffer_len(),
            gap_buffer.gap_start,
            gap_buffer.gap_end,
        );
        gap_buffer.delete_forwards(1);
        let text = gap_buffer.get_text().unwrap();
        print!("{}", text);
        assert_eq!(text, "ne way of doing things");
    }
}
