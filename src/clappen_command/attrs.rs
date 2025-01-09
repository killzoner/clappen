use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream, Result};
use syn::spanned::Spanned;
use syn::{Ident, LitStr, Path, Token, Type, TypePath};

use crate::helper::{camel_case, get_ident_from_path, macro_module_name, prefix};

#[derive(Clone)]
pub enum NestedAttributes {
    Apply(Path),
    Prefix(Option<String>),
}

impl Parse for NestedAttributes {
    fn parse(input: ParseStream) -> Result<Self> {
        let keyword: Ident = input.parse()?;
        // Advance the iterator so that we can skip the '=' token
        let _eq_token: Token![=] = input.parse()?;

        match &keyword {
            k if k == "apply" => Ok(NestedAttributes::Apply(input.parse()?)),
            k if k == "prefix" => {
                let val: LitStr = input.parse()?;
                let val = val.value();

                if val.is_empty() {
                    return Err(syn::Error::new(
                        keyword.span(),
                        "'prefix' attribute field  must not be empty when provided",
                    ));
                }

                Ok(NestedAttributes::Prefix(Some(val)))
            }
            e => Err(syn::Error::new(
                keyword.span(),
                format!("unknown attribute field '{}'", e),
            )),
        }
    }
}

#[derive(Clone)]
pub struct Attributes {
    pub apply: Path,
    pub prefix: String,
}

impl TryFrom<Vec<NestedAttributes>> for Attributes {
    type Error = ();

    fn try_from(fields: Vec<NestedAttributes>) -> std::result::Result<Self, Self::Error> {
        let macro_uses: Vec<_> = fields
            .iter()
            .flat_map(|e| match e {
                NestedAttributes::Apply(e) => Some(e),
                _ => None,
            })
            .collect();

        let macro_use = match macro_uses.first() {
            Some(e) => e,
            None => return Err(()),
        };

        let field_prefix: Vec<_> = fields
            .iter()
            .flat_map(|ref e| match e {
                NestedAttributes::Prefix(e) => e,
                _ => &None,
            })
            .collect();

        Ok(Attributes {
            apply: macro_use.to_owned().clone(),
            prefix: field_prefix
                .first()
                .map(|e| (*e).to_owned())
                .unwrap_or_default(),
        })
    }
}

impl Attributes {
    pub(crate) fn nested_macro_call(
        &self,
        default_prefix: &str,
        struct_prefix: &str,
        field_type: &Type,
    ) -> syn::Result<(TokenStream, TokenStream)> {
        let apply = &self.apply;
        let nested_prefix = camel_case(prefix(&[
            self.prefix.as_str(),
            default_prefix,
            struct_prefix,
        ]));
        let command_prefix = self.prefix.as_str();
        let (type_ident, _) = if let Type::Path(TypePath {path, .. }) = field_type{
            get_ident_from_path(path)?
        }else{
            return Err(syn::Error::new(field_type.span(), format!("Field type must be Path, found, {:?}", field_type)));
        };
        let module_name = macro_module_name(type_ident, &nested_prefix);
        let new_type_full_ref =
            Self::new_full_type_definition(&module_name, &nested_prefix, field_type)?;
        let default_prefix = if !default_prefix.is_empty() {quote! {#default_prefix}} else {quote! {}};
        let struct_prefix = if !struct_prefix.is_empty() {quote! {#struct_prefix}} else {quote! {}};
        Ok((
            quote! {
                #[allow(non_snake_case)]
                mod #module_name {
                    #apply!(@field #nested_prefix #struct_prefix #default_prefix #command_prefix);
                }
            },
            new_type_full_ref,
        ))
    }

    fn new_full_type_definition(
        module_name: &Ident,
        nested_prefix: &str,
        field_type: &Type,
    ) -> syn::Result<TokenStream> {
        // allow for fully qualified type notation, needed for $crate::something
        let field_type: Ident = match &field_type {
            Type::Path(e) => match e.path.segments.last() {
                Some(e) => e.ident.to_owned().clone(),
                None => {
                    return Err(syn::Error::new(
                        field_type.span(),
                        format!("cannot get ident out of {}", field_type.to_token_stream()),
                    ));
                }
            },
            _ => {
                return Err(syn::Error::new(field_type.span(), "unknown attribute"));
            }
        };

        let field_type = Ident::new(
            &camel_case(format!("{}{}", nested_prefix, field_type.to_token_stream())),
            Span::call_site(),
        );

        Ok(quote! {
            #module_name::#field_type
        })
    }

}
