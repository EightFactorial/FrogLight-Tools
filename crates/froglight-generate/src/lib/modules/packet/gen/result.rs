use std::{
    convert::Infallible,
    ops::{ControlFlow, FromResidual, Try},
};

use derive_more::derive::From;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Attribute, Type};

use super::{state::Item, State};

#[derive(From)]
pub(crate) enum Result {
    Void,
    Item { kind: Type, attrs: Vec<Attribute> },
    Err(anyhow::Error),
}

#[allow(dead_code)]
impl Result {
    /// Create a new [`Result::Item`] with the given [`Type`].
    #[must_use]
    pub(crate) const fn item_from(kind: Type) -> Self { Self::item_from_attrs(kind, Vec::new()) }

    /// Create a new [`Result::Item`] with the given [`Type`] and
    /// [`Attributes`](Attribute).
    #[must_use]
    pub(crate) const fn item_from_attrs(kind: Type, attrs: Vec<Attribute>) -> Self {
        Self::Item { kind, attrs }
    }

    /// Create a new [`Result::Item`] from the given [`String`].
    #[must_use]
    pub(crate) fn item_from_str(kind: impl AsRef<str>) -> Self {
        Self::item_from(syn::parse_str(kind.as_ref()).expect("Result: Invalid ItemTokens type"))
    }

    /// Create a new [`Result::Item`] from the given [`State<Item>`].
    #[must_use]
    pub(crate) fn item_from_state(state: State<Item>) -> Self {
        Self::item_from_tokens(state.item().to_token_stream())
    }

    /// Create a new [`Result::Item`] from the given [`TokenStream`].
    #[must_use]
    pub(crate) fn item_from_tokens(tokens: TokenStream) -> Self {
        Self::item_from(syn::parse2(tokens).expect("Result: Invalid ItemTokens type"))
    }

    /// Append the given [`Attribute`] to the [`Result::Item`].
    ///
    /// Does nothing if the result is not [`Result::Item`].
    #[must_use]
    pub(crate) fn with_attr(mut self, attr: Attribute) -> Self {
        if let Self::Item { ref mut attrs, .. } = &mut self {
            attrs.push(attr);
        }
        self
    }

    /// Append the given [`Attributes`](Attribute) to the [`Result::Item`].
    ///
    /// Does nothing if the result is not [`Result::Item`].
    #[must_use]
    pub(crate) fn with_attrs(mut self, attrs: impl IntoIterator<Item = Attribute>) -> Self {
        if let Self::Item { attrs: ref mut item_attrs, .. } = &mut self {
            item_attrs.extend(attrs);
        }
        self
    }

    /// Append the given [`TokenStream`] as an [`Attribute`] to the
    /// [`Result::Item`].
    ///
    /// Does nothing if the result is not [`Result::Item`].
    #[must_use]
    pub(crate) fn with_attr_tokens(self, tokens: TokenStream) -> Self {
        self.with_attr(syn::parse_quote!(#tokens))
    }
}

impl Result {
    /// Create a new [`Result::Item`] that represents an unsupported type.
    #[must_use]
    pub(crate) fn unsupported() -> Self { Self::item_from_str("Unsupported") }

    /// Map a [`Result::Item`] with the given function.
    ///
    /// Does nothing if the result is not [`Result::Item`].
    ///
    /// # Example
    /// ```rust
    /// use froglight_generate::generator::packet::gen::Result;
    /// ```
    #[must_use]
    pub(crate) fn map_item(self, fun: impl FnOnce(String) -> String) -> Self {
        match self {
            Self::Item { kind: Type::Path(mut type_path), attrs } => {
                // Map the returned type with the given function.
                if let Some(ident) = type_path.path.get_ident() {
                    let new_type = fun(ident.to_string());
                    type_path = syn::parse_str(&new_type).expect("Result: Invalid ItemMap type");
                }

                Self::Item { kind: Type::Path(type_path), attrs }
            }
            _ => self,
        }
    }
}

impl Try for Result {
    type Output = Self;
    type Residual = anyhow::Error;

    fn from_output(output: Self::Output) -> Self { output }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Result::Err(error) => ControlFlow::Break(error),
            _ => ControlFlow::Continue(self),
        }
    }
}
impl FromResidual<anyhow::Error> for Result {
    fn from_residual(err: anyhow::Error) -> Self { Self::Err(err) }
}
impl FromResidual<std::result::Result<Infallible, anyhow::Error>> for Result {
    fn from_residual(res: std::result::Result<Infallible, anyhow::Error>) -> Self {
        Self::Err(res.unwrap_err())
    }
}
