use std::borrow::Borrow;

use egui::Slider;
use rand::{Rng, RngCore, SeedableRng};
// #[cfg(feature="rng_small")]
// use rand::rngs::SmallRng;
use rand_distr::{StandardNormal, Uniform};
use rand_pcg;
use strum::{EnumMessage, VariantArray};

macro_rules! declare_rng_wrapper_macro {
    ($macro_name: ident, mod $path: tt$(, feature = $feature: literal)?) => {
        #[macro_export]
        macro_rules! $macro_name {
            (struct $rng_name: ident) => {
                $(#[cfg(feature = $feature)])?
                pub struct $rng_name(::$path::$rng_name);
            };
        }
    }
}

declare_rng_wrapper_macro!(create_rng_wrapper_pcg, mod rand_pcg);
declare_rng_wrapper_macro!(create_rng_wrapper_xoshiro, mod rand_xoshiro, feature = "rng_xoshiro");
declare_rng_wrapper_macro!(create_rng_wrapper_xorshift, mod rand_xorshift, feature = "rng_xorshift");

macro_rules! declare_rng_wrappers {
    (
        pcg: $($pcg_rng: ident),+ ; 
        xoshiro: $($xoshiro_rng: ident),+ ; 
        xorshift: $($xorshift_rng: ident),+;
    ) => {
        $(
            create_rng_wrapper_pcg!(struct $pcg_rng);
        )+
        $(
            create_rng_wrapper_xoshiro!(struct $xoshiro_rng);
        )+
        $(
            create_rng_wrapper_xorshift!(struct $xorshift_rng);
        )+

        pub enum WrappedRng {
            $($pcg_rng(::rand_pcg::$pcg_rng),)+
            $($xoshiro_rng(::rand_xoshiro::$xoshiro_rng),)+
            $($xorshift_rng(::rand_xorshift::$xorshift_rng),)+
        }

        #[derive(PartialEq, Clone, Copy)]
        pub enum WrappedRngDiscriminants {
            $($pcg_rng,)+
            $($xoshiro_rng,)+
            $($xorshift_rng,)+
        }

        impl WrappedRngDiscriminants {
            pub const VARIANTS: [WrappedRngDiscriminants; 10] = [$(Self::$pcg_rng),+, $(Self::$xoshiro_rng),+, $(Self::$xorshift_rng),+];
            pub fn seed_from_u64(&self, seed: u64) -> WrappedRng {
                use WrappedRngDiscriminants as D;
                use WrappedRng as T;
                match *self {
                    $(D::$pcg_rng => T::$pcg_rng(::rand_pcg::$pcg_rng::seed_from_u64(seed)),)+
                    $(D::$xoshiro_rng => T::$xoshiro_rng(::rand_xoshiro::$xoshiro_rng::seed_from_u64(seed)),)+
                    $(D::$xorshift_rng => T::$xorshift_rng(::rand_xorshift::$xorshift_rng::seed_from_u64(seed)),)+
                }
            }

            pub fn display_name(&self) -> &'static str {
                use WrappedRngDiscriminants as D;
                match *self {
                    $(D::$pcg_rng => stringify!($pcg_rng),)+
                    $(D::$xoshiro_rng => stringify!($xoshiro_rng),)+
                    $(D::$xorshift_rng => stringify!($xorshift_rng),)+
                }
            }
        }
        impl From<&WrappedRng> for WrappedRngDiscriminants {
            fn from(value: &WrappedRng) -> Self {
                use WrappedRng as T;
                use WrappedRngDiscriminants as D;
                match value {
                    $(T::$pcg_rng(_) => D::$pcg_rng),+,
                    $(T::$xoshiro_rng(_) => D::$xoshiro_rng),+,
                    $(T::$xorshift_rng(_) => D::$xorshift_rng),+
                }
            }
        }
    }
}


declare_rng_wrappers!{
    pcg: Pcg32, Pcg64, Pcg64Mcg; 
    xoshiro: Xoshiro256Plus, Xoshiro128Plus, Xoroshiro128Plus, Xoroshiro128StarStar, Xoroshiro64Star, Xoroshiro64StarStar; 
    xorshift: XorShiftRng;
}

impl WrappedRngDiscriminants {
    pub fn explanation(&self) -> &'static str {
        use WrappedRngDiscriminants as D;
        match self {
            D::Pcg32 => "Works okay pretty much everywhere",
            D::Pcg64 => "AFAIK strictly inferior to Pcg64Mcg except on 32bit platforms when generating f64, but since we generate f32, not helpful. Mostly here for completeness",
            D::Pcg64Mcg => "Better than Pcg32 on 64 bit platforms (which does NOT currently include the web!)",
            D::XorShiftRng => "Better than Pcg32 on 64 bit platforms (which does NOT currently include the web!)",
            D::Xoshiro128Plus => "Recommended for f32 generation.",
            D::Xoshiro256Plus => "Recommended for f64 generation (which is not what we do).",
            D::Xoroshiro128Plus => "Recommended for f64 generation (which is not what we do). Smaller state than Xoshiro256Plus",
            D::Xoroshiro128StarStar => "Recommended for f64 generation (which is not what we do). Smaller state than Xoshiro256Plus. Better scrambling than Plus variant, but more expensive",
            D::Xoroshiro64Star => "Recommended for f32 generation. Smaller state.",
            D::Xoroshiro64StarStar => "Recommended for f32 generation. Smaller state. Better scrambling than Plus variant, but more expensive",
            _ => "Look for this in the Rust Rand book/documentation",
        }
    }
}


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

impl WrappedRngDiscriminants {
    pub fn selection_ui(&mut self, ui: &mut egui::Ui) {
        for ele in Self::VARIANTS.into_iter() {
            // if *ele == WrappedRngDiscriminants::Boxed {
            //     continue;
            // }
            ui.selectable_value(self, ele, ele.display_name())
                .on_hover_text(ele.explanation());
        }
    }
}

impl WrappedRng {
    pub fn settings_ui(&mut self, ui: &mut egui::Ui) {
        #[derive(Clone)]
        struct Settings {
            discr: WrappedRngDiscriminants,
            seed: u64,
        }
        let id = ui.id();
        let mut current_settings = ui.data(|type_map| {
            type_map.get_temp(id).unwrap_or(Settings {
                discr: WrappedRngDiscriminants::from(self.borrow()),
                seed: 42,
            })
        });

        current_settings.discr.selection_ui(ui);
        ui.add(Slider::new(&mut current_settings.seed, 0..=u64::MAX));

        ui.data_mut(|type_map| {
            type_map.insert_temp(id, current_settings);
        });

        if ui.button("apply").clicked() {
            todo!();
        //     *self = AdoptedRngs::from(current_settings.discr);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sizeof_rand() {
        // TODO: update to enum
        // size of largest prng + discriminant
        assert!(size_of::<WrappedRng>() <= 32 + 8)
    }
}
