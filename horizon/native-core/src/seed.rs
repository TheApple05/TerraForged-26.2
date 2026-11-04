// MIT ReTerraForged Seed.java, preservation baseline 29239f1f164d19dbfcccaca4a8277d64e784207c.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Seed {
    root: i32,
    value: i32,
}

impl Seed {
    pub fn new(value: i32) -> Self {
        Self { root: value, value }
    }
    pub fn next_seed(&mut self) -> i32 {
        let value = self.value;
        self.value = self.value.wrapping_add(1);
        value
    }
    pub fn get(&self) -> i32 {
        self.value
    }
    pub fn root(&self) -> i32 {
        self.root
    }
    pub fn split(&self) -> Self {
        Self::new(self.root)
    }
    pub fn offset(&self, offset: i32) -> Self {
        Self::new(self.root.wrapping_add(offset))
    }
}
