extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_quote};

pub(crate) fn derive_component(mut ast: DeriveInput) -> TokenStream {
    let struct_name = &ast.ident;
    let fecs_path: syn::Path = crate::get_fulmen_ecs_path();

    // TODO: Add configuration options to be able to change the default storage type.

    // Generics boilerplate
    ast.generics
        .make_where_clause()
        .predicates
        .push(parse_quote! { Self: Send + Sync + 'static });
    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();

    // Generate impl block
    quote! {
        impl #impl_generics #fecs_path::component::Component for #struct_name #type_generics #where_clause {
            const STORAGE_TYPE: #fecs_path::component::StorageType = #fecs_path::component::StorageType::Table;
        }
    }.into()
}

pub(crate) fn derive_resource(mut ast: DeriveInput) -> TokenStream {
    let struct_name = &ast.ident;
    let fecs_path: syn::Path = crate::get_fulmen_ecs_path();

    // Generics boilerplate
    ast.generics
        .make_where_clause()
        .predicates
        .push(parse_quote! { Self: Send + Sync + 'static });
    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();

    // Generate impl block
    quote! {
        impl #impl_generics #fecs_path::resource::Resource for #struct_name #type_generics #where_clause {
        }
    }.into()
}
