use std::{fmt::Display};
use owning_ref::ArcRef;


#[derive(Debug)]
pub struct Cursor {
    src: ArcRef<str>,
    idx: usize,
}
impl Cursor {
    pub fn new(src: ArcRef<str>, idx: usize) -> Self {
        Self { src, idx }
    }
    fn get_text_pos(&self) -> (usize, usize) {
        let (mut row, mut col) = (1, 0);
        for ch in self.src.chars() {
            match ch {
                '\r' => {
                    col = 0;
                }
                '\n' => {
                    row += 1;
                    col = 0;
                }
                _ => {
                    col += 1;
                }
            }
        }
        (row, col)
    }
}
impl Display for Cursor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (row, col) = self.get_text_pos();
        write!(f, "{row}:{col}:\n{}\n", self.src[self.idx..].lines().next().unwrap_or("").trim_end())?;
        for _ in 1..col {
            write!(f, "~")?;
        }
        write!(f, "^")?;
        Ok(())
    }
}

