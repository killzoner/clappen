use proc_macro2::TokenStream;
use quote::quote;

// an arm of the exported macro
#[derive(Clone, Copy)]
pub(crate) enum Arm {
    // `NAME!()`
    Base,
    // `NAME!("p")`
    Prefixed,
    // `NAME!(@__struct "p")`
    Struct,
    // `NAME!(@__template "p", chain = [ .. ])`
    Template,
}

impl Arm {
    // the `prefix = ..` key the internal attributes take
    pub(crate) fn attribute_prefix(self) -> TokenStream {
        match self {
            Arm::Base => TokenStream::new(),
            Arm::Prefixed => quote! { prefix = $prefix, },
            Arm::Struct | Arm::Template => quote! { $(prefix = $prefix,)? },
        }
    }

    // the leading slot of a nested call
    pub(crate) fn invocation_prefix(self) -> TokenStream {
        match self {
            Arm::Base => TokenStream::new(),
            Arm::Prefixed => quote! { $prefix, },
            Arm::Struct | Arm::Template => quote! { $($prefix,)? },
        }
    }

    // only the `@__template` arm forwards chain steps: the ones it matched
    pub(crate) fn chain(self) -> TokenStream {
        match self {
            Arm::Base | Arm::Prefixed | Arm::Struct => TokenStream::new(),
            Arm::Template => {
                quote! { $( ( $($command_prefix,)? $field $(, $parent_default)? ), )* }
            }
        }
    }
}
