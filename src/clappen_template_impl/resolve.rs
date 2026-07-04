// Addressing: turn a `Base`/`Prefixed` tag into the concrete type it names at a given nesting
// position, plus the field prefix that position uses.

use syn::{Ident, Path, parse_quote};

use crate::clappen_template_impl::attrs;
use crate::helper;

// one resolved tag (Base or Prefixed): the tag ident, its concrete type path, and its field prefix
pub(crate) struct ResolvedTag {
    pub(crate) ident: Ident,
    pub(crate) path: Path,
    pub(crate) field_prefix: String,
}

impl ResolvedTag {
    // resolve one tag (Base or Prefixed) to its struct: go through the nesting chain to build the
    // module path + field prefix, the same way the struct generation builds them at each level
    pub(crate) fn new(
        start_prefix: &str,
        chain: &[attrs::ChainStep],
        default_prefix: &str,
        struct_ident: &Ident,
        tag: Ident,
    ) -> Self {
        let mut struct_prefix = start_prefix.to_string();
        let mut modules: Vec<Ident> = Vec::new();

        for step in chain {
            // the field name once the parent struct has prefixed it
            let parent_field_prefix = helper::field_prefix(&step.parent_default, &struct_prefix);
            let field_ident = helper::prefixed_field(&parent_field_prefix, &step.field.to_string());
            modules.push(helper::macro_module_name(&field_ident));
            struct_prefix = helper::nested_step_prefix(
                &step.command_prefix,
                &step.parent_default,
                &struct_prefix,
            );
        }

        // this struct's own default_prefix applies on top of the prefix built from the chain
        let field_prefix = helper::field_prefix(default_prefix, &struct_prefix);
        let type_ident = helper::prefixed_ident(&field_prefix, &struct_ident.to_string());
        let path: Path = parse_quote!(#(#modules ::)* #type_ident);

        Self {
            ident: tag,
            path,
            field_prefix,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Span;
    use quote::ToTokens;

    fn ident(name: &str) -> Ident {
        Ident::new(name, Span::call_site())
    }

    #[test]
    fn new_flat_prefixes_the_ident() {
        let resolved = ResolvedTag::new("svc", &[], "", &ident("ServerOptions"), ident("Prefixed"));
        assert_eq!(resolved.field_prefix, "svc");
        assert_eq!(resolved.ident.to_string(), "Prefixed");
        assert_eq!(
            resolved.path.to_token_stream().to_string(),
            "SvcServerOptions"
        );
    }

    #[test]
    fn new_empty_prefix_keeps_the_bare_ident() {
        let resolved = ResolvedTag::new("", &[], "", &ident("ServerOptions"), ident("Base"));
        assert_eq!(resolved.field_prefix, "");
        assert_eq!(resolved.path.to_token_stream().to_string(), "ServerOptions");
    }

    #[test]
    fn new_nested_builds_a_module_path() {
        let chain = vec![attrs::ChainStep {
            command_prefix: "db".to_string(),
            field: ident("database"),
            parent_default: String::new(),
        }];
        let resolved = ResolvedTag::new("", &chain, "", &ident("Db"), ident("Base"));
        assert_eq!(resolved.field_prefix, "db");
        assert_eq!(
            resolved.path.to_token_stream().to_string(),
            "__inner_database :: DbDb"
        );
    }
}
