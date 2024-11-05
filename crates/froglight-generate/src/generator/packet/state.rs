use syn::Ident;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct State<'item, 'field> {
    pub item: &'item Ident,
    pub field: &'field Ident,
}

impl<'i, 'f> State<'i, 'f> {
    /// Create a new [`State`]
    #[must_use]
    pub fn new(item: &'i Ident, field: &'f Ident) -> Self { Self { item, field } }

    /// Create a new [`State`], combining the item and field [`Ident`]s.
    #[must_use]
    pub fn combined(&self) -> Ident {
        Ident::new(&format!("{}_{}", self.item, self.field), self.item.span())
    }
}

impl<'i, 'f> State<'i, 'f> {
    /// Use the given item [`Ident`].
    ///
    /// Allows for passing down an item's [`Ident`] to it's children.
    #[must_use]
    pub fn with_item<'n>(&self, item: &'n Ident) -> State<'n, 'f> { State::new(item, self.field) }

    /// Use the given field [`Ident`].
    ///
    /// Allows for passing down a field's [`Ident`] to it's children.
    #[must_use]
    pub fn with_field<'n>(&self, field: &'n Ident) -> State<'i, 'n> { State::new(self.item, field) }
}
