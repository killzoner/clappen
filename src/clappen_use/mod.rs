use syn::parse::Parse;
use syn::ExprArray;

use crate::helper::parse_prefixes;

#[derive(Default, Clone)]
pub(crate) struct Attributes {
    pub prefixes: Vec<String>,
}

impl Parse for Attributes {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let arr: ExprArray = input.parse()?;
        let prefixes = parse_prefixes(arr)?;
        Ok(Self{prefixes})
    }
}
