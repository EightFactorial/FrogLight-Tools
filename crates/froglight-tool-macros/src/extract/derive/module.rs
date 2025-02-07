use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[derive(FromDeriveInput)]
#[darling(attributes(module))]
struct DependencyMacro {
    #[darling(default)]
    path: Option<syn::Path>,
    #[darling(default)]
    name: Option<String>,
    function: syn::Path,
}

pub(crate) fn derive_module(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse2(input).unwrap();

    let DependencyMacro { path, name, function } =
        DependencyMacro::from_derive_input(&input).unwrap();
    let DeriveInput { ident, .. } = input;

    let path = path.unwrap_or_else(|| syn::parse_quote!(froglight_extract));
    let name = name.unwrap_or_else(|| ident.to_string());

    quote! {
        impl #ident {
            /// The name of the associated [`ExtractModule`](#path::module::ExtractModule).
            pub const MODULE_NAME: &'static str = #name;
        }

        #path::inventory::submit! {
            #path::module::ExtractModule::new(#name, |v, d| Box::pin(#function(v, d)))
        }
    }
}
