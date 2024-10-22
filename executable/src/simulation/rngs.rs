use std::{borrow::Borrow, fmt::Display};

use egui::Slider;
use macros::{cfg_educe_debug, cfg_persistence_derive};
use rand::Rng;
use rand_distr::{Distribution, Uniform};

use crate::helpers::TempStateDataAccess;

macro_rules! declare_rng_wrapper_macro {
    ($macro_name: ident, mod $path: tt) => {
        #[macro_export]
        macro_rules! $macro_name {
            (struct $rng_name: ident) => {
                pub struct $rng_name(::$path::$rng_name);
            };
        }
    };
}

declare_rng_wrapper_macro!(create_rng_wrapper_pcg, mod rand_pcg);
declare_rng_wrapper_macro!(create_rng_wrapper_xoshiro, mod rand_xoshiro);
declare_rng_wrapper_macro!(create_rng_wrapper_xorshift, mod rand_xorshift);

macro_rules! declare_rng_wrappers {
    (
        pcg: $($pcg_rng: ident),+ ,;
        xoshiro: $($xoshiro_rng: ident),+ ,;
        xorshift: $($xorshift_rng: ident),+ ,;
    ) => {
        $(
            #[cfg(feature = "rng_pcg")]
            create_rng_wrapper_pcg!(struct $pcg_rng);
        )+
        $(
            #[cfg(feature = "rng_xoshiro")]
            create_rng_wrapper_xoshiro!(struct $xoshiro_rng);
        )+
        $(
            #[cfg(feature = "rng_xorshift")]
            create_rng_wrapper_xorshift!(struct $xorshift_rng);
        )+

        #[cfg_persistence_derive]
        #[derive(Clone)]
        #[cfg_educe_debug]
        pub enum WrappedRng {
            $(
                #[cfg(feature = "rng_pcg")]
                $pcg_rng(::rand_pcg::$pcg_rng),
            )+
            $(
                #[cfg(feature = "rng_xoshiro")]
                $xoshiro_rng(::rand_xoshiro::$xoshiro_rng),
            )+
            $(
                #[cfg(feature = "rng_xorshift")]
                $xorshift_rng(::rand_xorshift::$xorshift_rng),
            )+
        }

        #[derive(PartialEq, Clone, Copy)]
        #[repr(u8)]
        pub enum WrappedRngDiscriminants {
            $(
                #[cfg(feature = "rng_pcg")]
                $pcg_rng,
            )+
            $(
                #[cfg(feature = "rng_xoshiro")]
                $xoshiro_rng,
            )+
            $(
                #[cfg(feature = "rng_xorshift")]
                $xorshift_rng,
            )+
        }

        const RNG_CORE_UNIMPLEMENTED: &'static str = "I'm too lazy to do this properly without need, and unwilling to use the provided less efficient methods";

        impl rand::RngCore for WrappedRng {
            fn next_u32(&mut self) -> u32 {
                use WrappedRng as T;
                #[expect(clippy::pattern_type_mismatch, reason = "I'm lazy")]
                match self {
                    $(
                        #[cfg(feature = "rng_pcg")]
                        T::$pcg_rng(inner) => inner.next_u32(),
                    )+
                    $(
                        #[cfg(feature = "rng_xoshiro")]
                        T::$xoshiro_rng(inner) => inner.next_u32(),
                    )+
                    $(
                        #[cfg(feature = "rng_xorshift")]
                        T::$xorshift_rng(inner) => inner.next_u32(),
                    )+
                }
            }

            fn next_u64(&mut self) -> u64 {
                use WrappedRng as T;
                #[expect(clippy::pattern_type_mismatch, reason = "I'm lazy")]
                match self {
                    $(
                        #[cfg(feature = "rng_pcg")]
                        T::$pcg_rng(inner) => inner.next_u64(),
                    )+
                    $(
                        #[cfg(feature = "rng_xoshiro")]
                        T::$xoshiro_rng(inner) => inner.next_u64(),
                    )+
                    $(
                        #[cfg(feature = "rng_xorshift")]
                        T::$xorshift_rng(inner) => inner.next_u64(),
                    )+
                }
            }

            fn fill_bytes(&mut self, _dest: &mut [u8]) {
                unimplemented!("{RNG_CORE_UNIMPLEMENTED}")
            }


            fn try_fill_bytes(&mut self, _dest: &mut [u8]) -> Result<(), rand::Error> {
                unimplemented!("{RNG_CORE_UNIMPLEMENTED}")
            }
        }

        use rand::SeedableRng;

        impl WrappedRngDiscriminants {
            pub const VARIANTS: &'static [Self] = &[
            $(
                #[cfg(feature = "rng_pcg")]
                Self::$pcg_rng
            ),+,
            $(
                #[cfg(feature = "rng_xoshiro")]
                Self::$xoshiro_rng
            ),+,
            $(
                #[cfg(feature = "rng_xorshift")]
                Self::$xorshift_rng
            ),+
            ];

            pub fn seed_from_u64(&self, seed: u64) -> WrappedRng {
                use WrappedRngDiscriminants as D;
                use WrappedRng as T;
                match *self {
                    $(
                        #[cfg(feature = "rng_pcg")]
                        D::$pcg_rng => T::$pcg_rng(::rand_pcg::$pcg_rng::seed_from_u64(seed)),
                    )+
                    $(
                        #[cfg(feature = "rng_xoshiro")]
                        D::$xoshiro_rng => T::$xoshiro_rng(::rand_xoshiro::$xoshiro_rng::seed_from_u64(seed)),
                    )+
                    $(
                        #[cfg(feature = "rng_xorshift")]
                        D::$xorshift_rng => T::$xorshift_rng(::rand_xorshift::$xorshift_rng::seed_from_u64(seed)),
                    )+
                }
            }

            pub const fn display_name(&self) -> &'static str {
                use WrappedRngDiscriminants as D;
                match *self {
                    $(
                        #[cfg(feature = "rng_pcg")]
                        D::$pcg_rng => stringify!($pcg_rng),
                    )+
                    $(
                        #[cfg(feature = "rng_xoshiro")]
                        D::$xoshiro_rng => stringify!($xoshiro_rng),
                    )+
                    $(
                        #[cfg(feature = "rng_xorshift")]
                        D::$xorshift_rng => stringify!($xorshift_rng),
                    )+
                }
            }
        }
        impl From<&WrappedRng> for WrappedRngDiscriminants {
            fn from(value: &WrappedRng) -> Self {
                use WrappedRng as T;
                use WrappedRngDiscriminants as D;
                match value {
                    $(
                        #[cfg(feature = "rng_pcg")]
                        &T::$pcg_rng(_) => D::$pcg_rng
                    ),+,
                    $(
                        #[cfg(feature = "rng_xoshiro")]
                        &T::$xoshiro_rng(_) => D::$xoshiro_rng
                    ),+,
                    $(
                        #[cfg(feature = "rng_xorshift")]
                        &T::$xorshift_rng(_) => D::$xorshift_rng
                    ),+
                }
            }
        }
    }
}

// Module is only here to create a scope for that dead_code allow.
#[expect(
    dead_code,
    reason = "IDK why rust thinks all these variants are never constructed if they're all selectable."
)]
mod rng_wrappers {
    use macros::{cfg_educe_debug, cfg_persistence_derive};

    declare_rng_wrappers! {
        pcg:
            Pcg32,
            Pcg64,
            Pcg64Mcg,
        ;
        xoshiro:
            Xoshiro256Plus,
            Xoshiro128Plus,
            Xoroshiro128Plus,
            Xoroshiro128StarStar,
            Xoroshiro64Star,
            Xoroshiro64StarStar,
        ;
        xorshift:
            XorShiftRng,
        ;
    }
}

use rng_wrappers::*;

impl WrappedRngDiscriminants {
    pub const fn explanation(&self) -> &'static str {
        use WrappedRngDiscriminants as D;
        // TODO: these dont seem correct. Sampling with StandardNormal seems to call RngCore::next_u64
        match *self {
            #[cfg(feature = "rng_pcg")]
            D::Pcg32 => "Works okay pretty much everywhere",
            #[cfg(feature = "rng_pcg")]
            D::Pcg64 => "AFAIK strictly inferior to Pcg64Mcg except on 32bit platforms when generating f64, but since we generate f32, not helpful. Mostly here for completeness",
            #[cfg(feature = "rng_pcg")]
            D::Pcg64Mcg => "Better than Pcg32 on 64 bit platforms (which does NOT currently include the web!)",
            #[cfg(feature = "rng_xoshiro")]
            D::Xoshiro128Plus => "Recommended for f32 generation.",
            #[cfg(feature = "rng_xoshiro")]
            D::Xoshiro256Plus => "Recommended for f64 generation (which is not what we do).",
            #[cfg(feature = "rng_xoshiro")]
            D::Xoroshiro128Plus => "Recommended for f64 generation (which is not what we do). Smaller state than Xoshiro256Plus",
            #[cfg(feature = "rng_xoshiro")]
            D::Xoroshiro128StarStar => "Recommended for f64 generation (which is not what we do). Smaller state than Xoshiro256Plus. Better scrambling than Plus variant, but more expensive",
            #[cfg(feature = "rng_xoshiro")]
            D::Xoroshiro64Star => "Recommended for f32 generation. Smaller state.",
            #[cfg(feature = "rng_xoshiro")]
            D::Xoroshiro64StarStar => "Recommended for f32 generation. Smaller state. Better scrambling than Plus variant, but more expensive",
            #[cfg(feature = "rng_xorshift")]
            D::XorShiftRng => "Better than Pcg32 on 64 bit platforms (which does NOT currently include the web!)",
            // _ => "Look for this in the Rust Rand book/documentation",
        }
    }
}

impl Display for WrappedRngDiscriminants {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.display_name())
    }
}

// TODO: actually remove the enum in here and use the raw RNG. Should be possible.
// But I've spend enough time on this for now, so I'll get to it whenever I do.
#[cfg_persistence_derive]
#[derive(Clone)]
#[cfg_educe_debug]
pub struct RngIter<Distr: Distribution<f32>> {
    pub rng: WrappedRng,
    distr: Distr,
}

impl<T: Default + Distribution<f32>> Default for RngIter<T> {
    fn default() -> Self {
        Self {
            rng: WrappedRngDiscriminants::Pcg64Mcg.seed_from_u64(42),
            distr: Default::default(),
        }
    }
}

impl<Distr: Distribution<f32>> Iterator for RngIter<Distr> {
    type Item = f32;
    #[inline(always)]
    fn next(&mut self) -> Option<f32> {
        Some(self.rng.sample(&self.distr))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, None)
    }
}

impl<Distr: Distribution<f32>> RngIter<Distr> {
    // pub const fn new(rng: WrappedRng, distr: Distr) -> Self {
    //     Self { rng, distr }
    // }

    pub fn unwrapped_next(&mut self) -> f32 {
        self.next().expect("infinite iterator")
    }
}

#[cfg_persistence_derive]
#[derive(Clone, Default)]
#[cfg_educe_debug]
pub struct Percentage;

impl Distribution<f32> for Percentage {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f32 {
        rng.sample(Uniform::new_inclusive(0.0, 1.0))
    }
}

/// Recreated just to implement default
#[cfg_persistence_derive]
#[derive(Clone, Default)]
#[cfg_educe_debug]
pub struct StandardNormal;

impl Distribution<f32> for StandardNormal {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f32 {
        rng.sample(rand_distr::StandardNormal)
    }
}

impl WrappedRngDiscriminants {
    pub fn selection_ui(&mut self, ui: &mut egui::Ui) {
        for ele in Self::VARIANTS.iter() {
            ui.selectable_value(self, *ele, ele.display_name())
                .on_hover_text(ele.explanation());
        }
    }
}

impl WrappedRng {
    pub fn settings_ui(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        #[derive(Clone, Copy)]
        struct Settings {
            discr: WrappedRngDiscriminants,
            seed: u64,
        }

        let current_settings = ui.temp_ui_state::<Settings>().with_id(id).get();

        if let Some(mut current_settings) = current_settings {
            current_settings.discr.selection_ui(ui);
            // If I set this to u64::MAX to provide all options, its not realistically possible to select many values.
            ui.add(Slider::new(&mut current_settings.seed, 0..=300).text("Seed"));

            if ui.button("apply").clicked() {
                *self = current_settings.discr.seed_from_u64(current_settings.seed);
                ui.temp_ui_state::<Settings>().with_id(id).remove();
            } else {
                ui.temp_ui_state::<Settings>()
                    .with_id(id)
                    .set_or_create(current_settings);
            }
        } else {
            let current_rng_setting = WrappedRngDiscriminants::from(self.borrow());
            ui.label("Current RNG:");
            if ui.button(format!("{current_rng_setting}")).clicked() {
                ui.temp_ui_state::<Settings>().with_id(id).create(Settings {
                    discr: WrappedRngDiscriminants::from(self.borrow()),
                    seed: 42,
                });
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sizeof_rand() {
        // size of largest prng + discriminant + alignment (I think)
        assert!(size_of::<WrappedRng>() <= 32 + 8 + 8);
    }
}
