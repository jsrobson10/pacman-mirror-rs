
pub struct IterIterator<I,U,T> where I: Iterator<Item = T>, U: Clone {
    iters: Vec<(I, U)>,
    at: usize,
}

impl<I,U,T> IterIterator<I,U,T> where I: Iterator<Item = T>, U: Clone {
    pub fn new(iters: Vec<(I, U)>) -> Self {
        Self { iters, at: 0 }
    }
}

impl<I,U,T> Iterator for IterIterator<I,U,T> where I: Iterator<Item = T>, U: Clone {
    type Item = (T, U);

    fn next(&mut self) -> Option<Self::Item> {
        while self.iters.len() > 0 {
            let at = self.at % self.iters.len();
            let item = &mut self.iters[at];
            if let Some(value) = item.0.next() {
                self.at += 1;
                return Some((value, item.1.clone()));
            }
            self.iters.remove(at);
        }
        None
    }
}

