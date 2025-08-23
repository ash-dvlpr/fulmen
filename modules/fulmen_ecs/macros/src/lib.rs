extern crate proc_macro;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

// --- Modules
mod path;
mod component;

// --- API Flattening
pub(crate) use path::get_fulmen_path;

/// Implement the `Component` trait.
#[proc_macro_derive(Component)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    component::derive_component(ast).into()
}

/// Implement the `Resource` trait.
#[proc_macro_derive(Resource)]
pub fn derive_resource(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    component::derive_resource(ast).into()
}
