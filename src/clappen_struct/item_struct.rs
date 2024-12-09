use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Ident, ItemStruct, Token, Type};

use super::{
    ProcessItem, FIELD_ATTR_CLAPPEN_COMMAND, FIELD_ATTR_CLAPPEN_COMMAND_APPLY,
    FIELD_ATTR_CLAP_FLATTEN_COMMAND, FIELD_ATTR_CLAP_FLATTEN_COMMAND_FLATTEN,
};
use crate::clappen_command::attrs::NestedAttributes;
use crate::{clappen_command, helper};

impl ProcessItem for ItemStruct {
    fn process(
        &mut self,
        default_prefix: String,
        struct_prefix: String,
    ) -> syn::Result<TokenStream> {
        let mut nested_macro_uses: Vec<clappen_command::attrs::Attributes> = Vec::new();
        let mut nested_macro_calls: Vec<TokenStream> = Vec::new();

        let prefix = helper::prefix(&[default_prefix.as_str(), struct_prefix.as_str()]);

        // handle struct prefix
        if !prefix.is_empty() {
            let mut ident = self.ident.to_string();
            ident.insert_str(0, &helper::camel_case(prefix.clone()));

            self.ident = Ident::new(&ident, ident.span());
        }

        for field in self.fields.iter_mut() {
            // handle clappen_command arguments
            let mut command_clap_flatten = false;
            let mut clappen_command = false;
            let mut clappen_command_attributes: Option<clappen_command::attrs::Attributes> = None;

            // Check that we don't have a clap flatten without config
            for attr in &field.attrs {
                // parse #[command(flatten)] only, sub commands are still allowed
                if attr.path().is_ident(FIELD_ATTR_CLAP_FLATTEN_COMMAND) {
                    let _ = attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident(FIELD_ATTR_CLAP_FLATTEN_COMMAND_FLATTEN) {
                            command_clap_flatten = true;
                        }

                        Ok(())
                    });
                }

                // parse clappen_command arguments
                if attr.path().is_ident(FIELD_ATTR_CLAPPEN_COMMAND) {
                    clappen_command = true;

                    let meta: Punctuated<NestedAttributes, Token![,]> =
                        attr.parse_args_with(Punctuated::parse_terminated)?;
                    let meta: Vec<NestedAttributes> = meta.into_iter().collect();

                    let attrs: std::result::Result<clappen_command::attrs::Attributes, ()> =
                        meta.try_into();

                    clappen_command_attributes = attrs.ok();
                }
            }

            if command_clap_flatten && !clappen_command {
                return Err(syn::Error::new(
                    field.span(),
                    format!(
                        "'{}' must be specified when #[command(flatten)] is provided for clap",
                        FIELD_ATTR_CLAPPEN_COMMAND_APPLY,
                    ),
                ));
            }

            if clappen_command && clappen_command_attributes.is_none() {
                return Err(syn::Error::new(
                    field.span(),
                    format!(
                        "'{}' must be specified when #[{}] is provided",
                        FIELD_ATTR_CLAPPEN_COMMAND_APPLY, FIELD_ATTR_CLAPPEN_COMMAND,
                    ),
                ));
            }

            // handle fields prefix
            let mut ident = match &field.ident {
                Some(e) => e.to_string(),
                None => {
                    return Err(syn::Error::new(
                        field.span(),
                        "Ident field could not be parsed",
                    ))
                }
            };

            // field prefix
            if !prefix.is_empty() {
                ident.insert_str(0, format!("{}_", prefix).as_str());
            }

            field.ident = Some(Ident::new(&ident, ident.span()));

            // Handle nested field definitions with macro uses.
            if let (Some(command_attrs), Some(field_ident)) =
                (clappen_command_attributes, &field.ident)
            {
                let (new_macro_call, new_type_full) = command_attrs.nested_macro_call(
                    default_prefix.as_str(),
                    struct_prefix.as_str(),
                    field_ident,
                    &field.ty,
                );

                nested_macro_calls.push(new_macro_call);
                nested_macro_uses.push(command_attrs);

                // replace field type with new type
                let ty: Result<Type, syn::Error> = syn::parse2(new_type_full);
                match ty {
                    Ok(ty) => {
                        field.ty = ty;
                    }
                    Err(e) => {
                        return Err(syn::Error::new(
                            field.ty.span(),
                            format!(
                                "{}: unable to rewrite field '{}' type '{}' to new type",
                                e,
                                field.ident.to_token_stream(),
                                field.ty.to_token_stream(),
                            ),
                        ))
                    }
                }
            }
        }

        // Clean the attributes from the original struct
        self.fields.iter_mut().for_each(|field| {
            field
                .attrs
                .retain(|attr| !attr.path().is_ident(FIELD_ATTR_CLAPPEN_COMMAND));
        });

        let debug_nested_macro_uses: Vec<_> = nested_macro_uses
            .iter()
            .filter_map(|e: &clappen_command::attrs::Attributes| e.apply.get_ident())
            .map(|e| e.to_string())
            .collect();

        let debug_nested_macro_uses = debug_nested_macro_uses.join(",");
        let expanded = self;

        Ok(quote! {
            #(#nested_macro_calls)*
            #[doc=concat!(concat!(" Macros used for nested struct definition : [", #debug_nested_macro_uses, "]"))]
            #expanded
        })
    }
}
