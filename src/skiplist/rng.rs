use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

pub struct SeedableRng {
    rng: StdRng,
    seed: u64,
}

impl SeedableRng {
    pub fn new(seed: u64) -> SeedableRng {
        SeedableRng {
            StdRng::seed_from_u64(seed),
            seed
        }
    }

    pub fn from_entropy() -> SeedableRng {
        let seed: u64 = rand::thread_rng().gen();
        SeedableRng::new(seed);
    }

    pub fn view_seed() -> u64 {
        self.seed
    }
}
