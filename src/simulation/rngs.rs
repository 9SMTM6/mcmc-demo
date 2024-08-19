use std::borrow::Borrow;

use egui::Slider;
use rand::{Rng, RngCore, SeedableRng};
// #[cfg(feature="rng_small")]
// use rand::rngs::SmallRng;
use rand_distr::{StandardNormal, Uniform};
use rand_pcg::{Pcg32, Pcg64, Pcg64Mcg};
#[cfg(feature = "rng_xorshift")]
use rand_xorshift::XorShiftRng;
#[cfg(feature = "rng_xoshiro")]
use rand_xoshiro::{
    SplitMix64, Xoroshiro128Plus, Xoroshiro128StarStar, Xoroshiro64Star, Xoroshiro64StarStar, Xoshiro128Plus, Xoshiro256Plus,
};
use strum::{EnumMessage, VariantArray};

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone)]
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

    pub fn unwrapped_next(&mut self) -> f32 {
        self.next().expect("infinite iterator")
    }
}

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone)]
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
        Some(self.rng.sample(self.distr))
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

    pub fn unwrapped_next(&mut self) -> f32 {
        self.next().expect("infinite iterator")
    }
}

#[derive(strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::VariantArray, strum::Display, strum::EnumMessage))]
pub enum AdoptedRngs {
    #[strum_discriminants(strum(message = "Works okay pretty much everywhere"))]
    Pcg32(Pcg32),
    #[strum_discriminants(strum(
        message = "AFAIK strictly inferior to Pcg64Mcg except on 32bit platforms when generating f64, but since we generate f32, not helpful. Mostly here for completeness"
    ))]
    Pcg64(Pcg64),
    #[strum_discriminants(strum(
        message = "Better than Pcg32 on 64 bit platforms (which does NOT currently include the web!)"
    ))]
    Pcg64Mcg(Pcg64Mcg),
    #[cfg(feature = "rng_xorshift")]
    #[strum_discriminants(strum(
        message = "Better than Pcg32 on 64 bit platforms (which does NOT currently include the web!)"
    ))]
    XorShiftRng(XorShiftRng),
    #[cfg(feature = "rng_xoshiro")]
    SplitMix64(SplitMix64),
    // we select the xoshiro class algos according it its paper
    // https://vigna.di.unimi.it/ftp/papers/ScrambledLinear.pdf Chapter 5.3
    #[cfg(feature = "rng_xoshiro")]
    #[strum_discriminants(strum(
        message = "Recommended for f64 generation (which is not what we do)."
    ))]
    Xoshiro256Plus(Xoshiro256Plus),
    #[cfg(feature = "rng_xoshiro")]
    #[strum(message = "Recommended for f32 generation.")]
    Xoshiro128Plus(Xoshiro128Plus),
    #[cfg(feature = "rng_xoshiro")]
    #[strum_discriminants(strum(
        message = "Recommended for f64 generation (which is not what we do). Smaller state than Xoshiro256Plus"
    ))]
    Xoroshiro128Plus(Xoroshiro128Plus),
    #[cfg(feature = "rng_xoshiro")]
    #[strum_discriminants(strum(
        message = "Recommended for f64 generation (which is not what we do). Smaller state than Xoshiro256Plus. Better scrambling than Plus variant, but more expensive"
    ))]
    Xoroshiro128StarStar(Xoroshiro128StarStar),
    #[cfg(feature = "rng_xoshiro")]
    #[strum_discriminants(strum(message = "Recommended for f32 generation. Smaller state."))]
    Xoroshiro64Plus(Xoroshiro64Star),
    #[cfg(feature = "rng_xoshiro")]
    #[strum_discriminants(strum(
        message = "Recommended for f32 generation. Smaller state. Better scrambling than Plus variant, but more expensive"
    ))]
    Xoroshiro64StarStar(Xoroshiro64StarStar),
    // #[cfg(feature="rng_small")]
    // SmallRng(SmallRng),
    // feature rng/std_rng:
    // StdRng(StdRng),
    /// A wildcard, intended to hold RNGs with larger than 32 bits of state.
    /// We use RngCore for object safety. This object has wildcard implementations to the Rng trait.
    Boxed(Box<dyn RngCore>),
}

impl AdoptedRngsDiscriminants {
    pub fn selection_ui(&mut self, ui: &mut egui::Ui) {
        for ele in Self::VARIANTS.into_iter() {
            if *ele == AdoptedRngsDiscriminants::Boxed {
                continue;
            }
            ui.selectable_value(self, *ele, ele.to_string())
                .on_hover_text(ele.get_message().unwrap_or("Look for this on the Rust Rand book/documentation"));
        }
    }
}

impl AdoptedRngs {
    pub fn settings_ui(&mut self, ui: &mut egui::Ui) {
        #[derive(Clone)]
        struct Settings {
            discr: AdoptedRngsDiscriminants,
            seed: u64,
        }
        let id = ui.id();
        let mut current_settings = ui.data(|type_map| {
            type_map.get_temp(id).unwrap_or(Settings {
                discr: AdoptedRngsDiscriminants::from(self.borrow()),
                seed: 42,
            })
        });

        current_settings.discr.selection_ui(ui);
        ui.add(Slider::new(&mut current_settings.seed, 0..=u64::MAX));

        ui.data_mut(|type_map| {
            type_map.insert_temp(id, current_settings);
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sizeof_rand() {
        // TODO: update to enum
        // size of largest prng + discriminant
        assert!(size_of::<AdoptedRngs>() <= 32 + 8)
    }
}
