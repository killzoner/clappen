use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Ident, Path, Token, Type, parse_quote};

use crate::clappen_command::{FIELD_ATTR_CLAPPEN_COMMAND, FIELD_ATTR_CLAPPEN_COMMAND_APPLY};
use crate::helper::{
    self, PREFIX_ATTR,
    prefix::{CommandPrefix, DefaultPrefix, NestedPrefix, StructPrefix},
};

// the macro applied, optionally with `$crate` notation
pub(crate) struct ApplyPath {
    dollar: Option<Token![$]>,
    path: Path,
}

impl Parse for ApplyPath {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            dollar: input.parse()?,
            path: input.parse()?,
        })
    }
}

impl ToTokens for ApplyPath {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { dollar, path } = self;
        tokens.extend(quote! { #dollar #path });
    }
}

// One `name = value` pair. `syn::meta::parser` would need owned tokens and lose the error span.
enum NestedAttribute {
    Apply(ApplyPath),
    Prefix(CommandPrefix),
}

impl Parse for NestedAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let keyword: Ident = input.parse()?;
        // Advance the iterator so that we can skip the '=' token
        let _eq_token: Token![=] = input.parse()?;

        match keyword.to_string().as_str() {
            FIELD_ATTR_CLAPPEN_COMMAND_APPLY => Ok(NestedAttribute::Apply(input.parse()?)),
            PREFIX_ATTR => Ok(NestedAttribute::Prefix(input.parse()?)),
            _ => Err(syn::Error::new(keyword.span(), "unknown attribute")),
        }
    }
}

pub(crate) struct Attributes {
    pub apply: ApplyPath,
    pub prefix: CommandPrefix,
}

impl Parse for Attributes {
    fn parse(input: ParseStream) -> Result<Self> {
        let span = input.span();

        let fields = Punctuated::<NestedAttribute, Token![,]>::parse_terminated(input)?;

        let mut apply = None;
        let mut prefix = CommandPrefix::default();

        for field in fields {
            match field {
                NestedAttribute::Apply(e) => apply = Some(e),
                NestedAttribute::Prefix(e) => prefix = e,
            }
        }

        let apply = apply.ok_or_else(|| {
            syn::Error::new(
                span,
                format!(
                    "'{FIELD_ATTR_CLAPPEN_COMMAND_APPLY}' must be specified when #[{FIELD_ATTR_CLAPPEN_COMMAND}] is provided"
                ),
            )
        })?;

        Ok(Attributes { apply, prefix })
    }
}

// a nested struct: its macro call and its type
pub(crate) struct NestedStruct {
    pub new_macro_call: TokenStream,
    pub new_type: Type,
}

impl Attributes {
    pub(crate) fn nested_macro_call(
        &self,
        default_prefix: &DefaultPrefix,
        struct_prefix: &StructPrefix,
        field_ident: &Ident,
        field_type: &Type,
    ) -> Result<NestedStruct> {
        let apply = &self.apply;
        let nested_prefix = NestedPrefix::new(&self.prefix, default_prefix, struct_prefix);
        let module_name = helper::macro_module_name(&field_ident.to_string());
        let new_type = Self::new_full_type_definition(&module_name, &nested_prefix, field_type)?;

        Ok(NestedStruct {
            new_macro_call: quote! {
                    pub(crate) mod #module_name {
                        #apply!(#nested_prefix);
                    }
            },
            new_type,
        })
    }

    fn new_full_type_definition(
        module_name: &Ident,
        nested_prefix: &NestedPrefix,
        field_type: &Type,
    ) -> Result<Type> {
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
            _ => return Err(syn::Error::new(field_type.span(), "unknown attribute")),
        };

        let field_type = nested_prefix.type_ident(&field_type.to_string());

        Ok(parse_quote! {
            #module_name::#field_type
        })
    }
}
