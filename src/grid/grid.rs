// src/grid/grid.rs
pub struct Grid {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Option<usize>>, // Maps to particle index
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![None; width * height],
        }
    }

pub fn index(&self, x: usize, y: usize) -> Option<usize> {
    (x < self.width && y < self.height).then(|| y * self.width + x)
}

pub fn is_occupied(&self, x: usize, y: usize) -> bool {
    (x < self.width && y < self.height) && self.cells[y * self.width + x].is_some()
}

    pub fn get(&self, x: usize, y: usize) -> Option<usize> {
        self.index(x, y).and_then(|i| self.cells[i])
    }

    pub fn set(&mut self, x: usize, y: usize, particle_idx: Option<usize>) {
        if let Some(i) = self.index(x, y) {
            self.cells[i] = particle_idx;
        }
    }

pub fn clear(&mut self) {
    self.cells.fill(None);
}
}
