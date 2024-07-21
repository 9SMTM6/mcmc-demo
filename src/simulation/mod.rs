use rand::{Rng, SeedableRng};
use rand_distr::{StandardNormal, Uniform};

pub mod random_walk_metropolis_hastings;

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
pub struct SRngGaussianIter<Rng> {
    rng: Rng,
}

impl<R> Iterator for SRngGaussianIter<R>
where
    R: Rng + SeedableRng,
{
    type Item = f32;

    #[inline(always)]
    fn next(&mut self) -> Option<f32> {
        Some(self.rng.sample(StandardNormal))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, None)
    }
}

impl<R: Rng + SeedableRng> SRngGaussianIter<R> {
    pub fn new(seed: R::Seed) -> Self {
        Self {
            rng: R::from_seed(seed),
        }
    }

    fn unwrapped_next(&mut self) -> f32 {
        self.next().expect("infinite iterator")
    }
}

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
pub struct SRngPercIter<Rng> {
    rng: Rng,
    distr: Uniform<f32>,
}

impl<R> Iterator for SRngPercIter<R>
where
    R: Rng + SeedableRng,
{
    type Item = f32;

    #[inline(always)]
    fn next(&mut self) -> Option<f32> {
        Some(self.rng.sample(Uniform::new_inclusive(0.0, 1.0)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, None)
    }
}

impl<R: Rng + SeedableRng> SRngPercIter<R> {
    pub fn new(seed: R::Seed) -> Self {
        Self {
            rng: R::from_seed(seed),
            distr: Uniform::new_inclusive(0.0, 1.0),
        }
    }

    fn unwrapped_next(&mut self) -> f32 {
        self.next().expect("infinite iterator")
    }
}
