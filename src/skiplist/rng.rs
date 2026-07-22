use rand::{SeedableRng};
use rand::rngs::StdRng;
use rand::RngExt;

pub struct SeededRng {
    rng: StdRng,
    seed: u64,
}

impl SeededRng {
    pub fn new(seed: u64) -> SeededRng {
        SeededRng {
            rng: StdRng::seed_from_u64(seed),
            seed,
        }
    }

    pub fn from_entropy() -> SeededRng {
        let seed: u64 = rand::rng().random();
        SeededRng::new(seed)
    }

    pub fn view_seed(&self) -> u64 {
        self.seed
    }
}
