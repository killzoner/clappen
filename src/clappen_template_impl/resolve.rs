// Turns a tag into the type it names at a nesting position, and that position's field prefix.

use syn::{Ident, Path, parse_quote};

use crate::clappen_template_impl::attrs;
use crate::helper::{
    self,
    prefix::{DefaultPrefix, FieldPrefix, NestedPrefix, StructPrefix},
};

// a tag, with the type path and the field prefix it resolved to
pub(crate) struct ResolvedTag {
    pub ident: Ident,
    pub path: Path,
    pub field_prefix: FieldPrefix,
}

impl ResolvedTag {
    // walks the chain the way the struct generation does at each level
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
            let field_name = FieldPrefix::new(&step.parent_default, &struct_prefix)
                .field_name(&step.field.to_string());
            modules.push(helper::macro_module_name(&field_name));

            let nested_prefix =
                NestedPrefix::new(&step.command_prefix, &step.parent_default, &struct_prefix);
            struct_prefix = StructPrefix::from(&nested_prefix);
        }

        // this struct's own default_prefix goes on top of the chain's prefix
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

// the two tags, looked up by ident
pub(crate) struct Tags {
    pub base: ResolvedTag,
    pub prefixed: ResolvedTag,
}

impl Tags {
    // the tag a one-segment path names, with or without generic args (`Prefixed<T>`)
    pub(crate) fn for_path(&self, path: &Path) -> Option<&ResolvedTag> {
        match (
            &path.leading_colon,
            path.segments.first(),
            path.segments.len(),
        ) {
            (None, Some(segment), 1) => self.for_ident(&segment.ident),
            _ => None,
        }
    }

    pub(crate) fn for_ident(&self, ident: &Ident) -> Option<&ResolvedTag> {
        [&self.base, &self.prefixed]
            .into_iter()
            .find(|tag| *ident == tag.ident)
    }
}

// the tags and field prefixes that the `resolve`, `scope` and `rewrite` tests share
#[cfg(test)]
pub(crate) mod test_helpers {
    use quote::format_ident;

    use super::{ResolvedTag, Tags};
    use crate::helper::prefix::{DefaultPrefix, FieldPrefix, StructPrefix};
    use crate::helper::{PrefixValue, parse_literal};

    // a field prefix as the parser builds one; an empty prefix is the absent one
    pub(crate) fn field_prefix(value: &str) -> FieldPrefix {
        let struct_prefix = match value {
            "" => StructPrefix::default(),
            value => parse_literal(value),
        };

        FieldPrefix::new(&DefaultPrefix::default(), &struct_prefix)
    }

    // a resolved field prefix as a plain string, for readable assertions
    pub(crate) fn value(prefix: Option<&FieldPrefix>) -> Option<&str> {
        prefix.map(PrefixValue::as_str)
    }

    pub(crate) fn tag(tag: &str, path: &str, prefix: &str) -> ResolvedTag {
        ResolvedTag {
            ident: format_ident!("{tag}"),
            path: syn::parse_str(path).unwrap(),
            field_prefix: field_prefix(prefix),
        }
    }

    // `Base` is the unprefixed `ServerOptions`, `Prefixed` the `svc`-prefixed `SvcServerOptions`
    pub(crate) fn tags() -> Tags {
        Tags {
            base: tag("Base", "ServerOptions", ""),
            prefixed: tag("Prefixed", "SvcServerOptions", "svc"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_helpers::{tags, value};
    use super::*;
    use quote::{ToTokens, format_ident};

    use crate::helper::{PrefixValue, parse_literal};

    #[test]
    fn for_path_matches_tag_idents() {
        let tags = tags();
        let prefix = |path: Path| value(tags.for_path(&path).map(|tag| &tag.field_prefix));

        assert_eq!(prefix(parse_quote!(Prefixed)), Some("svc"));
        assert_eq!(prefix(parse_quote!(Base)), Some(""));
        assert_eq!(prefix(parse_quote!(Other)), None);
    }

    #[test]
    fn new_flat_prefixes_the_ident() {
        let resolved = ResolvedTag::new(
            parse_literal("svc"),
            &[],
            &DefaultPrefix::default(),
            &format_ident!("ServerOptions"),
            format_ident!("Prefixed"),
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
            &format_ident!("ServerOptions"),
            format_ident!("Base"),
        );
        assert_eq!(resolved.field_prefix.value().as_deref(), None);
        assert_eq!(resolved.path.to_token_stream().to_string(), "ServerOptions");
    }

    #[test]
    fn new_nested_builds_a_module_path() {
        let chain = vec![attrs::ChainStep {
            command_prefix: parse_literal("db"),
            field: format_ident!("nested"),
            parent_default: DefaultPrefix::default(),
        }];
        let resolved = ResolvedTag::new(
            StructPrefix::default(),
            &chain,
            &DefaultPrefix::default(),
            &format_ident!("MyStruct"),
            format_ident!("Base"),
        );
        assert_eq!(resolved.field_prefix.value().as_deref(), Some("db"));
        assert_eq!(
            resolved.path.to_token_stream().to_string(),
            "__inner_nested :: DbMyStruct"
        );
    }
}
