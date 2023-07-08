use serde::*;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Record(u64, u64, u128, usize, usize);

#[allow(dead_code)]
impl Record {
    #[inline]
    pub const fn new(position: u64, score: u64, millis: u128, rows: usize, columns: usize) -> Self {
        Self(position, score, millis, rows, columns)
    }

    #[inline]
    pub const fn position(&self) -> u64 {
        self.0
    }

    #[inline]
    pub fn set_position(&mut self, value: u64) {
        self.0 = value;
    }

    #[inline]
    pub const fn score(&self) -> u64 {
        self.1
    }

    #[inline]
    pub fn set_score(&mut self, value: u64) {
        self.1 = value;
    }

    #[inline]
    pub const fn millis(&self) -> u128 {
        self.2
    }

    #[inline]
    pub fn set_millis(&mut self, value: u128) {
        self.2 = value;
    }

    #[inline]
    pub const fn rows(&self) -> usize {
        self.3
    }

    #[inline]
    pub fn set_rows(&mut self, value: usize) {
        self.3 = value;
    }

    #[inline]
    pub const fn columns(&self) -> usize {
        self.4
    }

    #[inline]
    pub fn set_columns(&mut self, value: usize) {
        self.4 = value;
    }
}
