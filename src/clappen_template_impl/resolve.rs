// Addressing: turn a `Base`/`Prefixed` tag into the concrete type it names at a given nesting
// position, plus the field prefix that position uses.

use syn::{Ident, Path, parse_quote};

use crate::clappen_template_impl::attrs;
use crate::helper;
use crate::helper::prefix::{DefaultPrefix, FieldPrefix, NestedPrefix, StructPrefix};

// one resolved tag (Base or Prefixed): the tag ident, its concrete type path, and its field prefix
pub(crate) struct ResolvedTag {
    pub(crate) ident: Ident,
    pub(crate) path: Path,
    pub(crate) field_prefix: FieldPrefix,
}

impl ResolvedTag {
    // resolve one tag (Base or Prefixed) to its struct: go through the nesting chain to build the
    // module path + field prefix, the same way the struct generation builds them at each level
    pub(crate) fn new(
        start_prefix: StructPrefix,
        chain: &[attrs::ChainStep],
        default_prefix: &DefaultPrefix,
        struct_ident: &Ident,
        tag: Ident,
    ) -> Self {
        let mut struct_prefix = start_prefix;
        let mut modules: Vec<Ident> = Vec::new();

        for step in chain {
            // the field name once the parent struct has prefixed it
            let field_ident = FieldPrefix::new(&step.parent_default, &struct_prefix)
                .field_name(&step.field.to_string());
            modules.push(helper::macro_module_name(&field_ident));

            let nested =
                NestedPrefix::new(&step.command_prefix, &step.parent_default, &struct_prefix);
            struct_prefix = StructPrefix::from(&nested);
        }

        // this struct's own default_prefix applies on top of the prefix built from the chain
        let field_prefix = FieldPrefix::new(default_prefix, &struct_prefix);
        let type_ident = field_prefix.type_ident(&struct_ident.to_string());
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

    use crate::helper::{PrefixValue, parse_literal};

    fn ident(name: &str) -> Ident {
        Ident::new(name, Span::call_site())
    }

    fn struct_prefix(value: &str) -> StructPrefix {
        parse_literal(value)
    }

    #[test]
    fn new_flat_prefixes_the_ident() {
        let resolved = ResolvedTag::new(
            struct_prefix("svc"),
            &[],
            &DefaultPrefix::default(),
            &ident("ServerOptions"),
            ident("Prefixed"),
        );
        assert_eq!(resolved.field_prefix.value().as_deref(), Some("svc"));
        assert_eq!(resolved.ident.to_string(), "Prefixed");
        assert_eq!(
            resolved.path.to_token_stream().to_string(),
            "SvcServerOptions"
        );
    }

    #[test]
    fn new_without_a_prefix_keeps_the_bare_ident() {
        let resolved = ResolvedTag::new(
            StructPrefix::default(),
            &[],
            &DefaultPrefix::default(),
            &ident("ServerOptions"),
            ident("Base"),
        );
        assert_eq!(resolved.field_prefix.value().as_deref(), None);
        assert_eq!(resolved.path.to_token_stream().to_string(), "ServerOptions");
    }

    #[test]
    fn new_nested_builds_a_module_path() {
        let chain = vec![attrs::ChainStep {
            command_prefix: parse_literal("db"),
            field: ident("database"),
            parent_default: DefaultPrefix::default(),
        }];
        let resolved = ResolvedTag::new(
            StructPrefix::default(),
            &chain,
            &DefaultPrefix::default(),
            &ident("Db"),
            ident("Base"),
        );
        assert_eq!(resolved.field_prefix.value().as_deref(), Some("db"));
        assert_eq!(
            resolved.path.to_token_stream().to_string(),
            "__inner_database :: DbDb"
        );
    }
}
