#[derive(Clone, Debug)]
pub struct Compactor<T: Ord> {
    pub data: Vec<T>,
    pub capacity: usize,
}
impl<T: Ord> Compactor<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: Default::default(),
            capacity,
        }
    }
}
impl<T: Ord> Compactor<T> {
    pub fn update(&mut self, element: T) {
        self.data.push(element);
    }
}
