use derive_more::derive::From;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Attribute, Type};

#[derive(From)]
pub enum Result {
    Void,
    Item { kind: Type, attrs: Vec<Attribute> },
    Err(anyhow::Error),
}

impl Result {
    /// Return `Unsupported`
    #[must_use]
    pub fn unsupported() -> Self {
        Self::Item { kind: Type::Path(syn::parse_str("Unsupported").unwrap()), attrs: vec![] }
    }

    /// Return a [`Result::Item`] with the given [`Type`].
    #[must_use]
    pub fn kind(kind: Type) -> Self { Self::Item { kind, attrs: Vec::new() } }

    /// Return a [`Result::Item`] with the given [`Type`].
    #[must_use]
    pub fn kind_str(kind: impl AsRef<str>) -> Self {
        Self::kind(syn::parse_str(kind.as_ref()).expect("Result: Invalid Type"))
    }

    /// Return a [`Result::Item`] with the given [`Type`].
    ///
    /// # Note
    /// Prefer using [`Result::kind_str`] where possible.
    #[must_use]
    pub fn kind_string(kind: impl ToString) -> Self { Self::kind_str(kind.to_string()) }
}

impl Result {
    /// Map the [`Result::Item`] with a function.
    ///
    /// Does nothing if the variant is not [`Result::Item`].
    #[must_use]
    pub fn map(self, mapper: impl FnOnce(String) -> String) -> Self {
        if let Self::Item { kind, attrs } = self {
            let kind = kind.into_token_stream().to_string();
            Self::Item { kind: syn::parse_str(&mapper(kind)).expect("Result: Invalid Type"), attrs }
        } else {
            self
        }
    }
}

#[expect(dead_code)]
impl Result {
    /// Push an [`Attribute`] into the [`Result::Item`].
    ///
    /// This does nothing if the variant is not [`Result::Item`].
    pub fn push_attr(&mut self, attr: Attribute) {
        if let Self::Item { attrs, .. } = self {
            attrs.push(attr);
        }
    }

    /// Chain the [`Result::Item`] with an [`Attribute`].
    ///
    /// This does nothing if the variant is not [`Result::Item`].
    #[must_use]
    pub fn with_attr(mut self, attr: Attribute) -> Self {
        self.push_attr(attr);
        self
    }

    /// Push multiple [`Attribute`]s into the [`Result::Item`].
    ///
    /// This does nothing if the variant is not [`Result::Item`].
    pub fn extend_attrs(&mut self, attrs: impl IntoIterator<Item = Attribute>) {
        if let Self::Item { attrs: item_attrs, .. } = self {
            item_attrs.extend(attrs);
        }
    }

    /// Push an [`Attribute`] into the [`Result::Item`].
    ///
    /// # Panics
    /// This will panic if the [`TokenStream`] is not a valid [`Attribute`].
    pub fn push_attr_tokens(&mut self, tokens: TokenStream) {
        if let Self::Item { attrs, .. } = self {
            attrs.push(syn::parse_quote!(#tokens));
        }
    }

    /// Chain the [`Result::Item`] with an [`Attribute`].
    ///
    /// # Panics
    /// This will panic if the [`TokenStream`] is not a valid [`Attribute`].
    #[must_use]
    pub fn with_attr_tokens(mut self, tokens: TokenStream) -> Self {
        self.push_attr_tokens(tokens);
        self
    }
}

// TODO: Figure out how to implement this properly
impl std::ops::Try for Result {
    type Output = Self;
    type Residual = anyhow::Error;

    fn from_output(output: Self::Output) -> Self { output }

    fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
        if let Self::Err(err) = self {
            std::ops::ControlFlow::Break(err)
        } else {
            std::ops::ControlFlow::Continue(self)
        }
    }
}

impl std::ops::FromResidual<anyhow::Error> for Result {
    fn from_residual(residual: anyhow::Error) -> Self { Self::Err(residual) }
}
impl std::ops::FromResidual<std::result::Result<std::convert::Infallible, anyhow::Error>>
    for Result
{
    fn from_residual(
        residual: std::result::Result<std::convert::Infallible, anyhow::Error>,
    ) -> Self {
        match residual {
            Ok(_) => Self::Void,
            Err(err) => Self::Err(err),
        }
    }
}
