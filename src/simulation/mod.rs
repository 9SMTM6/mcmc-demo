use rand::{Rng, SeedableRng};
use rand_distr::{StandardNormal, Uniform};

pub mod random_walk_metropolis_hastings;

pub struct SRngGaussianIter<Rng>(rand_distr::DistIter<StandardNormal, Rng, f32>);

impl<R> Iterator for SRngGaussianIter<R>
where
    R: Rng + SeedableRng,
{
    type Item = f32;

    #[inline(always)]
    fn next(&mut self) -> Option<f32> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, None)
    }
}

impl<R: Rng + SeedableRng> SRngGaussianIter<R> {
    pub fn new(seed: R::Seed) -> Self {
        Self(R::from_seed(seed).sample_iter::<f32, _>(StandardNormal))
    }

    fn unwrapped_next(&mut self) -> f32 {
        self.next().expect("infinite iterator")
    }
}

pub struct SRngPercIter<Rng>(rand_distr::DistIter<Uniform<f32>, Rng, f32>);

impl<R> Iterator for SRngPercIter<R>
where
    R: Rng + SeedableRng,
{
    type Item = f32;

    #[inline(always)]
    fn next(&mut self) -> Option<f32> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, None)
    }
}

impl<R: Rng + SeedableRng> SRngPercIter<R> {
    pub fn new(seed: R::Seed) -> Self {
        Self(R::from_seed(seed).sample_iter::<f32, _>(Uniform::new_inclusive(0.0, 1.0)))
    }

    fn unwrapped_next(&mut self) -> f32 {
        self.next().expect("infinite iterator")
    }
}
