use proc_macro2::Span;
use syn::Ident;

/// A state machine for generating code.
#[derive(Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct State<S: StateType> {
    state: S,
}
pub trait StateType {}

impl State<Empty> {
    /// Create a new, empty [`State`].
    #[must_use]
    pub const fn new() -> Self { Self { state: Empty } }

    /// Create a new [`State`] with the given [`Item`](syn::Item).
    #[must_use]
    #[expect(clippy::unused_self)]
    pub fn with_item<I: AsIdent>(self, item: I) -> State<Item> {
        State { state: Item { tree: vec![item.as_ident()] } }
    }
}

impl State<Item> {
    /// Get the current [`Item`](syn::Item).
    #[must_use]
    pub fn item(&self) -> &Ident { self.state.tree.last().expect("Guaranteed non-empty") }

    /// Create a new [`State`] with the given [`Item`](syn::Item).
    #[must_use]
    pub fn with_item<I: AsIdent>(&self, item: I) -> State<Item> {
        let mut tree = self.state.tree.clone();
        tree.push(item.as_ident());
        State { state: Item { tree } }
    }

    /// Create a new [`State`] with the given target.
    #[must_use]
    pub fn with_target<I: AsIdent>(&self, target: I) -> State<Target<'_>> {
        State { state: Target { tree: &self.state.tree, target: target.as_ident() } }
    }
}

impl State<Target<'_>> {
    /// Get the current [`Item`](syn::Item).
    #[must_use]
    pub fn item(&self) -> &Ident { self.state.tree.last().expect("Guaranteed non-empty") }

    /// Create a new [`State`] with the given [`Item`](syn::Item).
    #[must_use]
    pub fn with_item<I: AsIdent>(&self, item: I) -> State<Item> {
        let mut tree = self.state.tree.to_vec();
        tree.push(item.as_ident());
        State { state: Item { tree } }
    }

    /// Create a new [`State`],
    /// turning the current [`Target`] into an [`Item`](syn::Item).
    #[must_use]
    pub fn create_item(&self) -> State<Item> {
        self.with_item(self.item().to_string() + "_" + self.target().to_string().as_str())
    }

    /// Get the current target.
    #[must_use]
    pub fn target(&self) -> &Ident { &self.state.target }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Empty;
impl StateType for Empty {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Item {
    tree: Vec<Ident>,
}
impl StateType for Item {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Target<'a> {
    tree: &'a [Ident],
    target: Ident,
}
impl StateType for Target<'_> {}

pub trait AsIdent {
    fn as_ident(&self) -> Ident;
}

impl AsIdent for Ident {
    fn as_ident(&self) -> Ident { self.clone() }
}

impl<T: AsRef<str>> AsIdent for &T {
    fn as_ident(&self) -> Ident { self.as_ref().as_ident() }
}
impl AsIdent for String {
    fn as_ident(&self) -> Ident { self.as_str().as_ident() }
}
impl AsIdent for &str {
    fn as_ident(&self) -> Ident { <str as AsIdent>::as_ident(self) }
}
impl AsIdent for str {
    fn as_ident(&self) -> Ident { Ident::new(self, Span::call_site()) }
}
