mod hw;

use crate::sync::{Lazy, Mutex};
use core::mem;
use rand_core::{CryptoRng, Error, RngCore, SeedableRng};
use rand_hc::Hc128Rng;
use x86_64::instructions::random::RdRand;

pub static TRNG: Lazy<Mutex<hw::Trng>> = Lazy::new(|| {
    let rng = match RdRand::new() {
        Some(rng) => hw::Trng::RdRand(rng),
        None => hw::Trng::Jitter(hw::JitterRng::init(4)),
    };
    Mutex::new(rng)
});
impl CryptoRng for hw::Trng {}

pub static CSPRNG: Lazy<Mutex<Hc128Rng>> =
    Lazy::new(|| Mutex::new(Hc128Rng::from_rng(&mut *TRNG.lock()).unwrap()));

pub struct Csprng {
    rng: Hc128Rng,
    rounds: usize,
}
impl Csprng {
    const RESEED: usize = 2048;
}
impl CryptoRng for Csprng {}

impl RngCore for Csprng {
    fn next_u32(&mut self) -> u32 {
        let ret = self.rng.next_u32();

        self.rounds += mem::size_of::<u32>();
        if self.rounds >= Self::RESEED {
            self.rounds = 0;
            self.rng = Hc128Rng::from_rng(&mut *TRNG.lock()).unwrap();
        }

        ret
    }
    fn next_u64(&mut self) -> u64 {
        let ret = self.rng.next_u64();

        self.rounds += mem::size_of::<u64>();
        if self.rounds >= Self::RESEED {
            self.rounds = 0;
            self.rng = Hc128Rng::from_rng(&mut *TRNG.lock()).unwrap();
        }

        ret
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.rng.fill_bytes(dest);

        self.rounds += dest.len();
        if self.rounds >= Self::RESEED {
            self.rounds = 0;
            self.rng = Hc128Rng::from_rng(&mut *TRNG.lock()).unwrap();
        }
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        let ret = self.rng.try_fill_bytes(dest);

        self.rounds += dest.len();
        if self.rounds >= Self::RESEED {
            self.rounds = 0;
            self.rng = Hc128Rng::from_rng(&mut *TRNG.lock()).unwrap();
        }

        ret
    }
}

#[cfg(test)]
mod tests {
    use core::mem;
    use rand_core::RngCore;

    #[test_case]
    fn trng() {
        let mut rng = super::TRNG.lock();
        for _ in 0..32 {
            assert_ne!(rng.next_u64(), rng.next_u64());
        }
    }

    #[test_case]
    fn csprng() {
        let mut rng = super::CSPRNG.lock();
        for _ in 0..(super::Csprng::RESEED / mem::size_of::<u64>() + mem::size_of::<u64>()) {
            assert_ne!(rng.next_u64(), rng.next_u64());
        }
    }
}
