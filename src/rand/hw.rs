use core::{mem, ptr};
use rand_core::RngCore;
use x86_64::instructions::random::RdRand;

pub enum Trng {
    RdRand(RdRand),
    Jitter(JitterRng),
}

impl RngCore for Trng {
    fn next_u32(&mut self) -> u32 {
        match self {
            Trng::RdRand(rng) => loop {
                if let Some(rn) = rng.get_u32() {
                    break rn;
                }
            },
            Trng::Jitter(rng) => {
                rng.gen_entropy();
                (rng.data & 0xFFFF_FFFF) as u32
            }
        }
    }
    fn next_u64(&mut self) -> u64 {
        match self {
            Trng::RdRand(rng) => loop {
                if let Some(rn) = rng.get_u64() {
                    break rn;
                }
            },
            Trng::Jitter(rng) => {
                rng.gen_entropy();
                rng.data
            }
        }
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        let mut rn;
        for chunk in dest.chunks_mut(8) {
            rn = self.next_u64();
            for (i, byte) in chunk
                .iter_mut()
                .enumerate()
                .map(|(i, byte)| (i as u64 * 8, byte))
            {
                *byte = ((rn & (0xFF << i)) >> i) as u8;
            }
        }
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        #[allow(clippy::unit_arg)]
        Ok(self.fill_bytes(dest))
    }
}

// CPU jitter TRNG
// https://www.chronox.de/jent.html

// https://github.com/smuellerDD/jitterentropy-library/blob/master/jitterentropy.h#L52
pub struct JitterRng {
    data: u64,
    prev_time: u64,
    last_delta: u64,
    last_delta2: u64,

    osr: u64,

    mem: *mut u8,
    mem_location: u64,
    mem_blocks: u64,
    mem_block_size: u64,
    mem_access_loops: u64,

    apt_observations: u64,
    apt_count: u64,
    apt_base: u64,
    apt_base_set: bool,

    health_failure: bool,
}
unsafe impl Send for JitterRng {}
unsafe impl Sync for JitterRng {}

impl JitterRng {
    const DATA_SIZE_BITS: u64 = mem::size_of::<u64>() as u64 * 8;

    const MEMORY_BLOCKS: usize = 64;
    const MEMORY_BLOCK_SIZE: usize = 32;
    const MEMORY_ACCESS_LOOPS: u64 = 128;

    const APT_WINDOW_SIZE: u64 = 512;
    const APT_CUTOFF: u64 = 325;
    const APT_LSB: u64 = 16;
    const APT_WORD_MASK: u64 = Self::APT_LSB - 1;

    fn apt_reset(&mut self, delta_masked: u64) {
        self.apt_count = 0;
        self.apt_base = delta_masked;
        self.apt_observations = 0;
    }

    fn apt_insert(&mut self, delta_masked: u64) {
        if !self.apt_base_set {
            self.apt_base = delta_masked;
            self.apt_base_set = true;
            return;
        }

        if delta_masked == self.apt_base {
            self.apt_count += 1;

            if self.apt_count >= Self::APT_CUTOFF {
                self.health_failure = true;
            }
        }

        self.apt_observations += 1;

        if self.apt_observations >= Self::APT_WINDOW_SIZE {
            Self::apt_reset(self, delta_masked);
        }
    }

    #[inline]
    fn delta(prev: u64, next: u64) -> u64 {
        if prev < next {
            next - prev
        } else {
            u64::MAX - prev + 1 + next
        }
    }

    fn stuck(&mut self, current_delta: u64) -> bool {
        let delta2 = Self::delta(self.last_delta, current_delta);
        let delta3 = Self::delta(self.last_delta2, delta2);
        let delta_masked = current_delta & Self::APT_WORD_MASK;

        self.last_delta = current_delta;
        self.last_delta2 = delta2;

        self.apt_insert(delta_masked);

        if current_delta == 0 || delta2 == 0 || delta3 == 0 {
            return true;
        }

        false
    }

    fn loop_shuffle(&self, bits: u64, min: u64) -> u64 {
        let mut time = tsc() ^ self.data;

        let mut shuffle: u64 = 0;
        let mask = (1 << bits) - 1;
        for _ in 0..=((Self::DATA_SIZE_BITS + bits - 1) / bits) {
            shuffle ^= time & mask;
            time >>= bits;
        }

        shuffle + (1 << min)
    }

    fn lfsr_time(&mut self, time: u64, stuck: bool) {
        const MAX_FOLD_LOOP_BIT: u64 = 4;
        const MIN_FOLD_LOOP_BIT: u64 = 0;
        let lfsr_loop_cnt = self.loop_shuffle(MAX_FOLD_LOOP_BIT, MIN_FOLD_LOOP_BIT);

        let mut new = 0;
        for _ in 0..lfsr_loop_cnt {
            new = self.data;
            for i in 1..Self::DATA_SIZE_BITS {
                let mut tmp = time << (Self::DATA_SIZE_BITS - i);

                tmp ^= (new >> 63) & 1;
                tmp ^= (new >> 60) & 1;
                tmp ^= (new >> 55) & 1;
                tmp ^= (new >> 30) & 1;
                tmp ^= (new >> 27) & 1;
                tmp ^= (new >> 22) & 1;
                new <<= 1;
                new ^= tmp;
            }
        }

        if !stuck {
            self.data = new;
        }
    }

    fn memaccess(&mut self) {
        const MAX_ACC_LOOP_BIT: u64 = 7;
        const MIN_ACC_LOOP_BIT: u64 = 0;
        let acc_loop_cnt = self.loop_shuffle(MAX_ACC_LOOP_BIT, MIN_ACC_LOOP_BIT);

        let wrap = self.mem_block_size * self.mem_blocks;

        for _ in 0..(self.mem_access_loops + acc_loop_cnt) {
            let tmp_val: *mut u8 = unsafe { self.mem.offset(self.mem_location as isize) };
            unsafe { *tmp_val = (*tmp_val).wrapping_add(1) };

            self.mem_location = self.mem_location + self.mem_block_size - 1;
            self.mem_location %= wrap;
        }
    }

    fn measure_jitter(&mut self) -> bool {
        self.memaccess();

        let time = tsc();
        let current_delta = Self::delta(self.prev_time, time);
        self.prev_time = time;

        let stuck = self.stuck(current_delta);

        self.lfsr_time(current_delta, stuck);

        stuck
    }

    fn gen_entropy(&mut self) {
        self.measure_jitter();

        let mut k = 0;
        loop {
            if self.measure_jitter() {
                continue;
            }

            k += 1;
            if k >= Self::DATA_SIZE_BITS * self.osr {
                break;
            }
        }
    }

    pub fn init(osr: u64) -> Self {
        let mut ec = Self {
            data: 0,
            prev_time: 0,
            last_delta: 0,
            last_delta2: 0,

            osr: osr.max(1),

            mem: unsafe {
                alloc::alloc::alloc_zeroed(alloc::alloc::Layout::new::<
                    [[u8; Self::MEMORY_BLOCK_SIZE]; Self::MEMORY_BLOCKS],
                >())
            },
            mem_location: 0,
            mem_blocks: Self::MEMORY_BLOCKS as u64,
            mem_block_size: Self::MEMORY_BLOCK_SIZE as u64,
            mem_access_loops: Self::MEMORY_ACCESS_LOOPS,

            apt_observations: 0,
            apt_count: 0,
            apt_base: 0,
            apt_base_set: false,

            health_failure: false,
        };

        assert_ne!(ec.mem, ptr::null_mut());

        ec.gen_entropy();
        ec
    }
}

fn tsc() -> u64 {
    let low: u32;
    let high: u32;
    unsafe {
        asm!("rdtsc", lateout("eax") low, lateout("edx") high);
    }
    low as u64 | (high as u64) << 32
}

#[cfg(test)]
mod tests {
    #[test_case]
    fn jitter() {
        let mut rng = super::JitterRng::init(4);
        let mut last = rng.data;
        for _ in 0..32 {
            rng.gen_entropy();
            assert_ne!(last, rng.data);
            last = rng.data;
        }
        assert!(!rng.health_failure);
    }
}
