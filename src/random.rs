use rand::{
    Rng,
    rngs::ThreadRng,
};

use crate::constants::*;

/// Generate a secret number between MIN_SECRET & MAX_SECRET
pub struct NumberGenerator {
    thread_rng: Option<ThreadRng>
}

impl NumberGenerator {
    pub fn new() -> Self {
        NumberGenerator {
            thread_rng: None,
        }
    }

    pub fn gen_secret(&mut self) -> u8 {
        self.get_rng().gen_range(MIN_SECRET, MAX_SECRET)
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

