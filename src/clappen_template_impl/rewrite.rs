// The rewrite has two passes, because pass 2 removes the tags that pass 1 reads:
// (1) `PrefixFields` finds the prefix of a value from its tag (`value: Prefixed` gives `svc`),
//     then prefixes the fields of that value. It changes no type.
// (2) `ReplaceTags` replaces each tag with its type path.
// One pass can do both jobs only if each step reads its tag before the walk replaces it.
//
// With `Prefixed` as `SvcServerOptions` (prefix `svc`), `Base` as `ServerOptions` (no prefix) and
// a field `url`, the template is:
//
//     impl From<Prefixed> for Base {
//         fn from(value: Prefixed) -> Self { Self { url: value.url } }
//     }
//
// After pass 1, `value.url` is `value.svc_url`. `Self` is `Base`, so `Self { url: .. }` keeps
// `url`:
//
//     impl From<Prefixed> for Base {
//         fn from(value: Prefixed) -> Self { Self { url: value.svc_url } }
//     }
//
// After pass 2, no tag is left:
//
//     impl From<SvcServerOptions> for ServerOptions {
//         fn from(value: SvcServerOptions) -> Self { Self { url: value.svc_url } }
//     }

use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};
use quote::{ToTokens, quote};
use syn::punctuated::Punctuated;
use syn::visit_mut::{self, VisitMut};
use syn::{
    Attribute, Block, Expr, ExprClosure, ExprField, ExprStruct, Ident, ImplItemFn, ItemFn,
    ItemImpl, Local, Macro, Member, Meta, PatStruct, Path, Token,
};

use crate::clappen_template_impl::{resolve::Tags, scope::Scope};
use crate::helper::{PrefixValue, prefix::FieldPrefix};

// pass 1: prefix the field names, while the tags are still readable
pub(crate) struct PrefixFields<'a> {
    tags: &'a Tags,
    fields: Vec<Ident>,
    scope: Scope,
}

// the examples use the names of the example at the top of the file
impl VisitMut for PrefixFields<'_> {
    // `value.url` becomes `value.svc_url`, and `(*value).url` the same
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

    // each impl, the template or one in a method body (`impl Label { .. }`), binds its own `self`
    fn visit_item_impl_mut(&mut self, node: &mut ItemImpl) {
        let scope = Scope::for_impl(node, self.tags);
        self.visit_in(scope, |visitor| {
            visit_mut::visit_item_impl_mut(visitor, node)
        });
    }

    // a `fn url_of(v: &Prefixed)` in a method body binds its own params only
    fn visit_item_fn_mut(&mut self, node: &mut ItemFn) {
        let scope = Scope::for_fn(&node.sig, self.tags);
        self.visit_in(scope, |visitor| visit_mut::visit_item_fn_mut(visitor, node));
    }

    // `fn a(value: Prefixed)` prefixes `value.url`, `fn b(value: Base)` in the same impl does not:
    // each method starts from `self` alone
    fn visit_impl_item_fn_mut(&mut self, node: &mut ImplItemFn) {
        let scope = self.scope.for_method(&node.sig, self.tags);
        self.visit_in(scope, |visitor| {
            visit_mut::visit_impl_item_fn_mut(visitor, node)
        });
    }

    // after `let other: &Prefixed = ..;` or `let other = value.clone();`, `other.url` becomes
    // `other.svc_url`
    fn visit_local_mut(&mut self, node: &mut Local) {
        // the initializer still reads the earlier binding: `let value: Base = value.into();`
        visit_mut::visit_local_mut(self, node);
        self.scope.bind_local(node, self.tags);
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

    // `#[instrument(fields(url = %self.url))]` becomes `#[instrument(fields(url = %self.svc_url))]`
    fn visit_attribute_mut(&mut self, node: &mut Attribute) {
        rewrite_attribute(self, node);
    }
}

impl<'a> PrefixFields<'a> {
    pub(crate) fn new(tags: &'a Tags, fields: Vec<Ident>) -> Self {
        Self {
            tags,
            fields,
            scope: Scope::default(),
        }
    }

    // walks a node with `scope`, then puts back the outer scope
    fn visit_in(&mut self, scope: Scope, visit: impl FnOnce(&mut Self)) {
        let outer = std::mem::replace(&mut self.scope, scope);
        visit(self);
        self.scope = outer;
    }

    // `url` becomes `svc_url`, and a renamed shorthand `Self { url }` gets `svc_url: url` to read
    // `url`
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

impl RewriteToken for PrefixFields<'_> {
    // in a body syn cannot parse, `value.url` becomes `value.svc_url`, but not `other.value.url`
    // or the method call `value.url()`
    fn rewrite_token(&self, tokens: &[TokenTree], i: usize) -> Option<TokenStream> {
        let [
            rest @ ..,
            TokenTree::Ident(name),
            TokenTree::Punct(dot),
            TokenTree::Ident(field),
        ] = &tokens[..=i]
        else {
            return None;
        };
        let after_dot = matches!(rest.last(), Some(TokenTree::Punct(p)) if p.as_char() == '.');
        let method_call = matches!(
            tokens.get(i + 1),
            Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis
        );
        if dot.as_char() != '.' || after_dot || method_call {
            return None;
        }

        let prefix = self.scope.prefix(&name.to_string())?;
        let renamed = self.prefixed_member(prefix, &Member::Named(field.clone()))?;
        Some(renamed.into_token_stream())
    }
}

// pass 2: the tags become their type paths
pub(crate) struct ReplaceTags<'a> {
    pub tags: &'a Tags,
}

impl VisitMut for ReplaceTags<'_> {
    // `Prefixed` becomes `SvcServerOptions`, `Base::default` becomes `ServerOptions::default`, and
    // `Prefixed<T>` becomes `SvcServerOptions<T>`
    fn visit_path_mut(&mut self, path: &mut Path) {
        if path.leading_colon.is_none()
            && let Some(first) = path.segments.first()
            && let Some(tag) = self.tags.for_ident(&first.ident)
        {
            let mut segments = tag.path.segments.clone();
            if let Some(last) = segments.last_mut() {
                last.arguments = first.arguments.clone();
            }
            let rest = path.segments.iter().skip(1).cloned();
            path.segments = segments.into_iter().chain(rest).collect();
            path.leading_colon = tag.path.leading_colon;
        }
        visit_mut::visit_path_mut(self, path);
    }

    // `vec![Base::default()]` becomes `vec![ServerOptions::default()]`
    fn visit_macro_mut(&mut self, node: &mut Macro) {
        visit_mut::visit_macro_mut(self, node);
        visit_macro_body(self, node);
    }

    // `#[my_attr(Prefixed)]` becomes `#[my_attr(SvcServerOptions)]`
    fn visit_attribute_mut(&mut self, node: &mut Attribute) {
        rewrite_attribute(self, node);
    }
}

impl RewriteToken for ReplaceTags<'_> {
    // in a body syn cannot parse, `Base` becomes `ServerOptions`, but not `other::Base`
    fn rewrite_token(&self, tokens: &[TokenTree], i: usize) -> Option<TokenStream> {
        let TokenTree::Ident(ident) = &tokens[i] else {
            return None;
        };
        let after_path = match &tokens[..i] {
            [.., TokenTree::Punct(a), TokenTree::Punct(b)]
                if (a.as_char(), b.as_char()) == (':', ':') =>
            {
                true
            }
            [.., TokenTree::Punct(p)] => p.as_char() == '.',
            _ => false,
        };
        if after_path {
            return None;
        }

        Some(self.tags.for_ident(ident)?.path.to_token_stream())
    }
}

// how a pass rewrites one token of a body syn cannot parse, from the tokens around it
trait RewriteToken {
    fn rewrite_token(&self, tokens: &[TokenTree], i: usize) -> Option<TokenStream>;
}

// syn keeps a macro body as raw tokens: a body of comma-separated expressions (`assert_eq!(..)`)
// or of statements (`vec![x; n]`) is rewritten like the code around it, any other body
// (`matches!(x, Some(v) if ..)`) token by token
fn visit_macro_body<V: VisitMut + RewriteToken>(pass: &mut V, node: &mut Macro) {
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
        _ => node.tokens = rewrite_tokens(&*pass, &node.tokens),
    }
}

// an attribute's args are raw tokens, like a macro body syn cannot parse
fn rewrite_attribute(pass: &impl RewriteToken, node: &mut Attribute) {
    if let Meta::List(list) = &mut node.meta {
        list.tokens = rewrite_tokens(pass, &list.tokens);
    }
}

// each token gets `rewrite_token` with the tokens around it, nested groups included
fn rewrite_tokens(pass: &impl RewriteToken, tokens: &TokenStream) -> TokenStream {
    let tokens: Vec<TokenTree> = tokens.clone().into_iter().collect();
    tokens
        .iter()
        .enumerate()
        .map(|(i, token)| {
            pass.rewrite_token(&tokens, i)
                .unwrap_or_else(|| match token {
                    TokenTree::Group(group) => {
                        let stream = rewrite_tokens(pass, &group.stream());
                        let mut rewritten = Group::new(group.delimiter(), stream);
                        rewritten.set_span(group.span());
                        TokenTree::Group(rewritten).into()
                    }
                    token => token.clone().into(),
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::{format_ident, quote};
    use syn::parse_quote;

    use crate::clappen_template_impl::resolve::test_helpers::{field_prefix, tag, tags};

    #[test]
    fn replace_tags_rewrites_a_tag_as_the_first_segment_of_a_path() {
        let tags = Tags {
            base: tag("Base", "ServerOptions", ""),
            prefixed: tag("Prefixed", "__inner_nested::SvcServerOptions", "svc"),
        };
        let replaced = |mut path: Path| {
            ReplaceTags { tags: &tags }.visit_path_mut(&mut path);
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
        let visitor = PrefixFields::new(&tags, vec![format_ident!("url")]);
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
