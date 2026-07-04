use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream, Result};
use syn::spanned::Spanned;
use syn::{Ident, Path, Token, Type};

use crate::helper::{
    self, PREFIX_ATTR,
    prefix::{CommandPrefix, DefaultPrefix, NestedPrefix, StructPrefix},
};

pub(crate) enum NestedAttribute {
    Apply(TokenStream),
    Prefix(CommandPrefix),
}

impl Parse for NestedAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let keyword: Ident = input.parse()?;
        // Advance the iterator so that we can skip the '=' token
        let _eq_token: Token![=] = input.parse()?;

        match keyword.to_string().as_str() {
            "apply" => {
                // parsed as a path, kept as tokens: `$crate::child` needs the `$`
                let dollar: Option<Token![$]> = input.parse()?;
                let path: Path = input.parse()?;
                Ok(NestedAttribute::Apply(quote! { #dollar #path }))
            }
            PREFIX_ATTR => Ok(NestedAttribute::Prefix(input.parse()?)),
            _ => Err(syn::Error::new(keyword.span(), "unknown attribute")),
        }
    }
}

pub(crate) struct Attributes {
    pub apply: TokenStream,
    pub prefix: CommandPrefix,
}

impl TryFrom<Vec<NestedAttribute>> for Attributes {
    type Error = ();

    fn try_from(fields: Vec<NestedAttribute>) -> std::result::Result<Self, Self::Error> {
        let mut apply = None;
        let mut prefix = None;

        for field in fields {
            match field {
                NestedAttribute::Apply(e) => apply = apply.or(Some(e)),
                NestedAttribute::Prefix(e) => prefix = prefix.or(Some(e)),
            }
        }

        Ok(Attributes {
            apply: apply.ok_or(())?,
            prefix: prefix.unwrap_or_default(),
        })
    }
}

impl Attributes {
    pub(crate) fn nested_macro_call(
        &self,
        default_prefix: &DefaultPrefix,
        struct_prefix: &StructPrefix,
        field_ident: &Ident,
        field_type: &Type,
    ) -> (TokenStream, TokenStream) {
        let apply = &self.apply;
        let nested_prefix = NestedPrefix::new(&self.prefix, default_prefix, struct_prefix);
        let module_name = helper::macro_module_name(&field_ident.to_string());
        let new_type_full_ref =
            Self::new_full_type_definition(&module_name, &nested_prefix, field_type);

        (
            quote! {
                    pub(crate) mod #module_name {
                        #apply!(@__struct #nested_prefix);
                    }
            },
            new_type_full_ref,
        )
    }

    fn new_full_type_definition(
        module_name: &Ident,
        nested_prefix: &NestedPrefix,
        field_type: &Type,
    ) -> TokenStream {
        // allow for fully qualified type notation, needed for $crate::something
        let field_type: Ident = match &field_type {
            Type::Path(e) => match e.path.segments.last() {
                Some(e) => e.ident.to_owned().clone(),
                None => {
                    return syn::Error::new(
                        field_type.span(),
                        format!("cannot get ident out of {}", field_type.to_token_stream()),
                    )
                    .into_compile_error();
                }
            },
            _ => {
                return syn::Error::new(field_type.span(), "unknown attribute")
                    .into_compile_error();
            }
        };

        let field_type = nested_prefix.type_ident(&field_type.to_string());

        quote! {
            #module_name::#field_type
        }
    }
}
