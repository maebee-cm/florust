pub struct CircularVec<T> {
    vec: Vec<T>,
    start: usize,
    end: usize,
    max_size: usize,
}

impl<T> CircularVec<T> {
    pub fn new(max_size: usize) -> CircularVec<T>
    where
        T: Clone,
    {
        CircularVec {
            vec: Vec::with_capacity(max_size),
            start: 0,
            end: 0,
            max_size
        }
    }

    pub fn append(&mut self, val: T) {
        self.vec[self.end] = val;
        self.increment_range();
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        let index = self.start + index;
        let index = if index >= self.max_size {
            index - self.max_size
        }
        else {
            index
        };

        self.vec.get(index)
    }

    fn increment_range(&mut self) {
        self.end += 1;

        if self.end == self.max_size {
            self.end = 0;
        }

        if self.end == self.start {
            self.start += 1;

            if self.start == self.max_size {
                self.start = 0;
            }
        }
    }
}
