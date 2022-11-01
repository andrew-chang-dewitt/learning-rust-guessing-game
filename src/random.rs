use rand::{rngs::ThreadRng, Rng};

use crate::constants::{MAX_SECRET, MIN_SECRET};

/// Generate a secret number between MIN_SECRET & MAX_SECRET
pub struct NumberGenerator {
    thread_rng: Option<ThreadRng>,
    max: usize,
    min: usize,
}

impl NumberGenerator {
    pub fn new(min: usize, max: usize) -> Self {
        NumberGenerator {
            thread_rng: None,
            max,
            min,
        }
    }

    pub fn gen_secret(&mut self) -> usize {
        self.get_rng().gen_range(&self.min, &self.max)
    }

    fn get_rng(&mut self) -> ThreadRng {
        match self.thread_rng {
            Some(instance) => instance,
            None => self.init_rng(),
        }
    }

    fn init_rng(&mut self) -> ThreadRng {
        self.thread_rng = Some(rand::thread_rng());
        self.get_rng()
    }
}

impl Default for NumberGenerator {
    fn default() -> Self {
        Self::new(MIN_SECRET, MIN_SECRET)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn gen_secret_returns_a_number_between_min_and_max() {
//         // TODO: finish this test?
//         assert!(false)
//     }
// }
