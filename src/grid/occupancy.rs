pub struct Grid {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Option<usize>>, // maps to particle index
}
