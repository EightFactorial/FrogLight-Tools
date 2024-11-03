use syn::{Field, File, ItemStruct};

/// A stack of parents that can be used to navigate the fields of a struct.
pub struct ParentStack {
    current: usize,
    parents: Vec<ItemStruct>,
}

impl ParentStack {
    pub fn new() -> Self { Self { current: 0, parents: Vec::new() } }

    pub fn push(&mut self, parent: ItemStruct) {
        self.parents.push(parent);
        self.current = self.parents.len() - 1;
    }

    pub fn pop(&mut self) -> Option<ItemStruct> {
        self.current = self.current.saturating_sub(1);
        self.parents.pop()
    }
}

impl ParentStack {
    /// Get the field by following the path given by `field_ref`.
    ///
    /// As previously generated structs are not stored in the [`ParentStack`],
    /// fields in those structs are accessed through the provided [`File`].
    pub fn get_field<'a>(
        &'a mut self,
        mut field_ref: &str,
        file: &'a mut File,
    ) -> anyhow::Result<&'a mut Field> {
        // Recurse upwards using "../" into the parents
        let mut parent_counter = 0usize;
        while let Some(stripped) = field_ref.strip_prefix("../") {
            field_ref = stripped;

            parent_counter += 1;
        }
        let mut parent = &mut self.parents[self.current - parent_counter];

        // Iterate down the "field/" in the file.
        let mut parent_ident = parent.ident.to_string();
        while let Some((field_name, stripped)) = field_ref.split_once('/') {
            field_ref = stripped;

            if let Some(parent_field) =
                parent.fields.iter_mut().find(|field| field.ident.as_ref().unwrap() == field_name)
            {
                if let syn::Type::Path(type_path) = &parent_field.ty {
                    let type_ident = type_path.path.segments.last().unwrap().ident.to_string();
                    if let Some(parent) = file
                        .items
                        .iter_mut()
                        .filter_map(|item| {
                            if let syn::Item::Struct(item_struct) = item {
                                Some(item_struct)
                            } else {
                                None
                            }
                        })
                        .find(|item_struct| item_struct.ident == type_ident)
                    {
                        parent_ident = parent.ident.to_string();
                    }
                }
            } else {
                anyhow::bail!("Field not found: {parent_ident} : {field_name}");
            }
        }

        // Find the parent that matches the `parent_ident`.
        if parent.ident != parent_ident {
            if let Some(matching) = file
                .items
                .iter_mut()
                .filter_map(|item| {
                    if let syn::Item::Struct(item_struct) = item {
                        Some(item_struct)
                    } else {
                        None
                    }
                })
                .find(|item_struct| item_struct.ident == parent_ident)
            {
                parent = matching;
            }
        }

        // Find the field, now that we have the correct parent.
        if let Some(field) =
            parent.fields.iter_mut().find(|field| field.ident.as_ref().unwrap() == field_ref)
        {
            Ok(field)
        } else {
            anyhow::bail!("Field not found: {field_ref}")
        }
    }
}

impl std::ops::Deref for ParentStack {
    type Target = ItemStruct;
    fn deref(&self) -> &Self::Target { &self.parents[self.current] }
}
impl std::ops::DerefMut for ParentStack {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.parents[self.current] }
}
