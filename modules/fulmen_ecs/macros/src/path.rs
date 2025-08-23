use proc_macro_crate::{FoundCrate, crate_name};

/// Attempts to return the correct path for the base crate
pub(crate) fn get_fulmen_ecs_path() -> syn::Path {
    match crate_name("fulmen_ecs") {
        Ok(FoundCrate::Itself) => syn::parse_str("crate").unwrap(),
        Ok(FoundCrate::Name(name)) => syn::parse_str(&name).unwrap(),
        Err(_) => syn::parse_str("fulmen_ecs").unwrap(),
    }
}
