//! Since proc-macros require a separate crate, this is the crate to hold macros that must be implemented as proc-macros.
//!
//! Flags im still consiering to add:
//! cfg_not_wasm:
//! #[cfg(not(target_arch = "wasm32"))]
//! #[cfg(target_arch = "wasm32")]
//! #[cfg(feature = "backend_panel")]
//! #[cfg(feature = "more_debug_impls")]
//! #[cfg(not(feature = "more_debug_impls"))]
//! #[cfg(feature = "debounce_async_loops")]
//! #[cfg(feature = "rng_xoshiro")]
//! #[cfg(feature = "rng_pcg")]
//! #[cfg(feature = "rng_xorshift")]
//! #[cfg_attr(
//!     feature = "tracing",
//!     <attr>
//! )]
//! #[cfg(feature = "persistence")]
//!
//! search replace previous apprarances, including:
//! #[expect(clippy::shadow_unrelated, reason = "false positive, is related.")] = #[expect(clippy::shadow_unrelated, reason = "false positive")]
extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use shared::cfg_if_expr;

/// Expands to
///
/// ```rs
/// #[derive(serde::Deserialize, serde::Serialize)]
/// ```
///
/// If `feature = "persistence"`, otherwise it creates a fake derive to accept serde non-macro attributes
#[proc_macro_attribute]
pub fn cfg_persistence_derive(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    let attr = cfg_if_expr!(
        => [feature = "persistence"]
        quote!(#[derive(::serde::Deserialize, ::serde::Serialize)])
        => [not]
        quote!(#[derive(::macros::FakeSerde)])
    );
    let expanded = quote! {
        #attr
        #input
    };
    expanded.into()
}

/// This 'accepts' serde attributes, and does nothing else.
/// Its here to avoid compilation errors when serde is not applied because of conditional compilation
#[proc_macro_derive(FakeSerde, attributes(serde))]
pub fn fake_serde(_: TokenStream) -> TokenStream {
    Default::default()
}

/// Expands to
///
/// ```rs
/// #[derive(educe::Educe)]
/// #[educe(Debug)]
/// ```
///
/// If `feature = "more_debug_impls"`, otherwise it creates a fake derive to accept educe non-macro attributes
#[proc_macro_attribute]
pub fn cfg_educe_debug(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    let attr = cfg_if_expr!(
        => [feature = "more_debug_impls"]
        quote! {
            #[derive(::educe::Educe)]
            #[educe(Debug)]
        }
        => [not]
        quote! {
            #[derive(::macros::FakeEduce)]
        }
    );
    let expanded = quote!(
        #attr
        #input
    );
    expanded.into()
}

/// This 'accepts' educe attributes, and does nothing else.
/// Its here to avoid compilation errors when educe::Debug is not applied because of conditional compilation
#[proc_macro_derive(FakeEduce, attributes(educe))]
pub fn fake_educe(_: TokenStream) -> TokenStream {
    Default::default()
}
