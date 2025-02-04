use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[derive(FromDeriveInput)]
#[darling(attributes(dep))]
struct DependencyMacro {
    #[darling(default)]
    path: Option<syn::Path>,
    #[darling(default)]
    retrieve: Option<syn::Path>,
}

pub(crate) fn derive_dependency(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse2(input).unwrap();

    let DependencyMacro { path, retrieve } = DependencyMacro::from_derive_input(&input).unwrap();
    let path = path.unwrap_or_else(|| syn::parse_quote!(froglight_dependency));

    let DeriveInput { ident, .. } = input;

    if let Some(retrieve) = retrieve {
        quote! {
            impl #path::container::Dependency for #ident {}
            impl #path::container::Retrievable for #ident {
                #[inline]
                async fn retrieve(deps: &mut #path::container::DependencyContainer) -> anyhow::Result<Self> {
                    #retrieve(deps).await
                }
            }
        }
    } else {
        quote! {
            impl #path::container::Dependency for #ident {}
        }
    }
}
