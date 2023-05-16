use std::collections::{HashSet, VecDeque};

/// Represents a worker threads view of an argument, with its current assignment for this thread.
pub struct ArgumentView {
    iteration: usize,
    value: bool,
    positive_watched: HashSet<usize>,
    negative_watched: HashSet<usize>,
}

impl ArgumentView {
    pub fn new() -> ArgumentView {
        ArgumentView {
            iteration: 0,
            value: true,
            positive_watched: HashSet::new(),
            negative_watched: HashSet::new()
        }
    }

    #[inline]
    pub fn add_watched_in(&mut self, id: usize, sign: bool) {
        if sign {
            self.positive_watched.insert(id);
        }
        else {
            self.negative_watched.insert(id);
        }
    }

    #[inline]
    pub fn remove_watched_in(&mut self, id: usize, sign: bool) {
        if sign {
            self.positive_watched.remove(&id);
        }
        else {
            self.negative_watched.remove(&id);
        }
    }

    pub fn set_value(&mut self, value: bool, iteration: usize, clauses_to_check: &mut VecDeque<usize>) {
        self.iteration = iteration;
        self.value = value;

        let hash_map = match value {
            true => &mut self.negative_watched,
            false => &mut self.positive_watched
        };

        for entry in hash_map.iter() {
            clauses_to_check.push_back(*entry)
        }
    }

    #[inline]
    pub fn get_value(&self, iteration: usize) -> Option<bool> {
        if self.iteration == iteration {
            Some(self.value)
        }
        else {
            None
        }
    }
}