pub mod diff_display;
pub mod multimodal_gaussian;
mod resolution_uniform;

pub use resolution_uniform::INITIAL_RENDER_SIZE;

#[macro_export]
#[allow(unknown_lints)] // not a lint of stable...
#[allow(edition_2024_expr_fragment_specifier)]
macro_rules! create_shader_module {
    ($shader_name:expr, $module_name: ident) => {
        #[allow(
            unused,
            elided_lifetimes_in_paths,
            clippy::approx_constant,
            clippy::module_name_repetitions,
            clippy::pattern_type_mismatch,
            clippy::unreadable_literal
        )]
        pub mod $module_name {
            include!(concat!(
                env!("OUT_DIR"),
                "/shaders_bindings/",
                $shader_name,
                ".rs"
            ));
        }
    };
    ($shader_name:expr) => {
        create_shader_module!($shader_name, shader_bindings);
    };
}

create_shader_module!("fullscreen_quad.vertex", fullscreen_quad);
