// The rewrite has two passes, because pass 2 removes the tags that pass 1 reads.
//
// With `Prefixed` as `SvcServerOptions` (prefix `svc`), `Base` as `ServerOptions` (no prefix) and
// a field `url`, the template is:
//
//     impl From<Prefixed> for Base {
//         fn from(value: Prefixed) -> Self { Self { url: value.url } }
//     }
//
// Pass 1, `RenameFields`, prefixes the fields of a value whose type is a tag (`value: Prefixed`).
// The tags stay for pass 2:
//
//     impl From<Prefixed> for Base {
//         fn from(value: Prefixed) -> Self { Self { url: value.svc_url } }
//     }
//
// Pass 2, `RenameTags`, replaces each tag with its type path, so no tag is left:
//
//     impl From<SvcServerOptions> for ServerOptions {
//         fn from(value: SvcServerOptions) -> Self { Self { url: value.svc_url } }
//     }

use quote::{ToTokens, quote};
use syn::punctuated::Punctuated;
use syn::visit_mut::{self, VisitMut};
use syn::{
    Block, Expr, ExprClosure, ExprField, ExprStruct, Ident, ImplItemFn, ItemImpl, Local, Macro,
    Member, PatStruct, Path, Token,
};

use crate::clappen_template_impl::{resolve::Tags, scope::Scope};
use crate::helper::{PrefixValue, prefix::FieldPrefix};

// pass 1: prefix the field names, while the tags are still readable
pub(crate) struct RenameFields<'a> {
    tags: &'a Tags,
    fields: Vec<Ident>,
    scope: Scope,
}

// the comments use the names of the example at the top of the file
impl<'a> RenameFields<'a> {
    pub(crate) fn new(tags: &'a Tags, fields: Vec<Ident>, item: &ItemImpl) -> Self {
        Self {
            tags,
            fields,
            scope: Scope::for_impl(item, tags),
        }
    }

    // `url` becomes `svc_url`, and a shorthand `Self { url }` becomes `Self { svc_url: url }`
    fn rename_field(
        &self,
        prefix: &FieldPrefix,
        member: &mut Member,
        colon: &mut Option<Token![:]>,
    ) {
        if let Some(renamed) = self.prefixed_member(prefix, member) {
            colon.get_or_insert_default();
            *member = Member::Named(renamed);
        }
    }

    // the prefixed name for a named field in the prefixed set, else `None`
    fn prefixed_member(&self, prefix: &FieldPrefix, member: &Member) -> Option<Ident> {
        if prefix.value().is_none() {
            return None;
        }
        let Member::Named(ident) = member else {
            return None;
        };

        self.fields.contains(ident).then(|| {
            let field_name = prefix.field_name(&ident.to_string());

            Ident::new(&field_name, ident.span())
        })
    }
}

impl VisitMut for RenameFields<'_> {
    // `value.url` becomes `value.svc_url`
    fn visit_expr_field_mut(&mut self, node: &mut ExprField) {
        visit_mut::visit_expr_field_mut(self, node);
        if let Some(prefix) = self.scope.expr_prefix(&node.base)
            && let Some(renamed) = self.prefixed_member(prefix, &node.member)
        {
            node.member = Member::Named(renamed);
        }
    }

    // `Prefixed { url }` becomes `Prefixed { svc_url: url }`, and `Self { url }` the same in an
    // impl for `Prefixed`
    fn visit_expr_struct_mut(&mut self, node: &mut ExprStruct) {
        visit_mut::visit_expr_struct_mut(self, node);
        if let Some(prefix) = self.scope.path_prefix(&node.path, self.tags) {
            for field in &mut node.fields {
                self.rename_field(prefix, &mut field.member, &mut field.colon_token);
            }
        }
    }

    // `let Prefixed { url, .. } = value;` becomes `let Prefixed { svc_url: url, .. } = value;`
    fn visit_pat_struct_mut(&mut self, node: &mut PatStruct) {
        visit_mut::visit_pat_struct_mut(self, node);
        if let Some(prefix) = self.scope.path_prefix(&node.path, self.tags) {
            for field in &mut node.fields {
                self.rename_field(prefix, &mut field.member, &mut field.colon_token);
            }
        }
    }

    // `fn a(value: Prefixed)` prefixes `value.url`, `fn b(value: Base)` in the same impl does not:
    // each method starts from `self` alone
    fn visit_impl_item_fn_mut(&mut self, node: &mut ImplItemFn) {
        let scope = self.scope.for_method(&node.sig, self.tags);
        let outer = std::mem::replace(&mut self.scope, scope);
        visit_mut::visit_impl_item_fn_mut(self, node);
        self.scope = outer;
    }

    // after `let other: &Prefixed = ..;`, `other.url` becomes `other.svc_url`
    fn visit_local_mut(&mut self, node: &mut Local) {
        // the initializer still reads the earlier binding: `let value: Base = value.into();`
        visit_mut::visit_local_mut(self, node);
        self.scope.bind(&node.pat, self.tags);
    }

    // `|other: &Prefixed| other.url` becomes `|other: &Prefixed| other.svc_url`
    fn visit_expr_closure_mut(&mut self, node: &mut ExprClosure) {
        for input in &node.inputs {
            self.scope.bind(input, self.tags);
        }
        visit_mut::visit_expr_closure_mut(self, node);
    }

    // `assert_eq!(value.url, ..)` becomes `assert_eq!(value.svc_url, ..)`
    fn visit_macro_mut(&mut self, node: &mut Macro) {
        visit_mut::visit_macro_mut(self, node);
        visit_macro_body(self, node);
    }
}

// pass 2: the tags become their type paths
pub(crate) struct RenameTags<'a> {
    pub tags: &'a Tags,
}

impl VisitMut for RenameTags<'_> {
    // `Prefixed` becomes `SvcServerOptions`, `Base::default` becomes `ServerOptions::default`, and
    // `Prefixed<T>` becomes `SvcServerOptions<T>`
    fn visit_path_mut(&mut self, path: &mut Path) {
        if path.leading_colon.is_none()
            && let Some(first) = path.segments.first()
            && let Some(tag) = self.tags.for_ident(&first.ident)
        {
            let mut renamed = tag.path.clone();
            if let Some(last) = renamed.segments.last_mut() {
                last.arguments = first.arguments.clone();
            }
            renamed
                .segments
                .extend(path.segments.iter().skip(1).cloned());
            *path = renamed;
        }
        visit_mut::visit_path_mut(self, path);
    }

    // `vec![Base::default()]` becomes `vec![ServerOptions::default()]`
    fn visit_macro_mut(&mut self, node: &mut Macro) {
        visit_mut::visit_macro_mut(self, node);
        visit_macro_body(self, node);
    }
}

// syn keeps a macro body as raw tokens: a body of comma-separated expressions (`assert_eq!(..)`)
// or of statements (`vec![x; n]`) is rewritten like the code around it
fn visit_macro_body(pass: &mut impl VisitMut, node: &mut Macro) {
    match (
        node.parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated),
        node.parse_body_with(Block::parse_within),
    ) {
        (Ok(mut args), _) => {
            for arg in &mut args {
                pass.visit_expr_mut(arg);
            }
            node.tokens = args.into_token_stream();
        }
        (_, Ok(mut stmts)) => {
            for stmt in &mut stmts {
                pass.visit_stmt_mut(stmt);
            }
            node.tokens = quote!(#(#stmts)*);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::{format_ident, quote};
    use syn::parse_quote;

    use crate::clappen_template_impl::resolve::test_helpers::{field_prefix, tag, tags};

    #[test]
    fn rename_tags_rewrites_a_tag_as_the_first_segment_of_a_path() {
        let tags = Tags {
            base: tag("Base", "ServerOptions", ""),
            prefixed: tag("Prefixed", "__inner_nested::SvcServerOptions", "svc"),
        };
        let replaced = |mut path: Path| {
            RenameTags { tags: &tags }.visit_path_mut(&mut path);
            quote!(#path).to_string()
        };

        assert_eq!(
            replaced(parse_quote!(Base::default)),
            quote!(ServerOptions::default).to_string()
        );
        assert_eq!(
            replaced(parse_quote!(Prefixed::from)),
            quote!(__inner_nested::SvcServerOptions::from).to_string()
        );
        // a leading `::`: not a tag
        assert_eq!(
            replaced(parse_quote!(::Base::default)),
            quote!(::Base::default).to_string()
        );
        // generic args move to the type's own segment
        assert_eq!(
            replaced(parse_quote!(Base::<u8>::default)),
            quote!(ServerOptions::<u8>::default).to_string()
        );
        assert_eq!(
            replaced(parse_quote!(Prefixed<T>)),
            quote!(__inner_nested::SvcServerOptions<T>).to_string()
        );
    }

    #[test]
    fn prefixed_member_renames_only_known_named_fields() {
        let tags = tags();
        let visitor = RenameFields::new(
            &tags,
            vec![format_ident!("url")],
            &parse_quote!(impl Base {}),
        );
        let renamed = |prefix: &str, member: Member| {
            visitor
                .prefixed_member(&field_prefix(prefix), &member)
                .map(|ident| ident.to_string())
        };

        // a prefixed field with a non-empty prefix
        assert_eq!(
            renamed("svc", parse_quote!(url)),
            Some("svc_url".to_string())
        );
        // an empty prefix
        assert_eq!(renamed("", parse_quote!(url)), None);
        // a field outside the prefixed set
        assert_eq!(renamed("svc", parse_quote!(other)), None);
        // a tuple field
        assert_eq!(renamed("svc", parse_quote!(0)), None);
    }
}
