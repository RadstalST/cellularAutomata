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
        if x < self.width && y < self.height {
            Some(y * self.width + x)
        } else {
            None
        }
    }

    pub fn is_occupied(&self, x: usize, y: usize) -> bool {
        self.index(x, y).map_or(false, |i| self.cells[i].is_some())
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
        for cell in self.cells.iter_mut() {
            *cell = None;
        }
    }
}