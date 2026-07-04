// Rewriting runs two passes over the impl block, because prefixing a field name must read the tags
// and replacing a tag destroys them:
// (1) `PrefixFields` prefixes the field reads off a tagged binding and the fields of a tag struct
//     literal, while `Base` / `Prefixed` still name the tags.
// (2) `ReplaceTags` replaces the tag idents with their concrete type paths.

use syn::visit_mut::{self, VisitMut};
use syn::{
    Arm, Block, Expr, ExprClosure, ExprField, ExprForLoop, ExprStruct, FnArg, Ident, ImplItemFn,
    ItemImpl, Local, Member, Pat, PatIdent, PatType, Path, Signature, Type,
};

use crate::clappen_template_impl::resolve::ResolvedTag;
use crate::helper::{PrefixValue, prefix::FieldPrefix};

const SELF_BINDING: &str = "self";

// rewrite the base/prefixed tags to their paths and prefix the relevant field accesses /
// struct-literal fields
pub(crate) fn substitute(
    mut item: ItemImpl,
    base: ResolvedTag,
    prefixed: ResolvedTag,
    fields: Vec<Ident>,
) -> ItemImpl {
    let tags = Tags { base, prefixed };

    let mut prefix_fields = PrefixFields::new(&tags, fields, &item);
    prefix_fields.visit_item_impl_mut(&mut item);
    ReplaceTags { tags: &tags }.visit_item_impl_mut(&mut item);

    item
}

// the two tags an impl can name, looked up by the ident a path or a type uses
struct Tags {
    base: ResolvedTag,
    prefixed: ResolvedTag,
}

impl Tags {
    // the resolved tag a path names, if it is a bare `base`/`prefixed` tag ident
    fn for_path(&self, path: &Path) -> Option<&ResolvedTag> {
        [&self.base, &self.prefixed]
            .into_iter()
            .find(|tag| path.is_ident(&tag.ident))
    }

    // the field prefix a value of type `ty` carries, if `ty` names a tag. A leading `&`/`&mut` is
    // peeled, so `&Prefixed` counts as `Prefixed`; other wrappers (`Option<Prefixed>`,
    // `Box<Prefixed>`) are not recognized, so their field reads stay unprefixed.
    fn prefix_for_type(&self, ty: &Type) -> Option<&FieldPrefix> {
        let mut ty = ty;
        while let Type::Reference(reference) = ty {
            ty = &reference.elem;
        }
        let Type::Path(path) = ty else { return None };

        self.for_path(&path.path).map(|tag| &tag.field_prefix)
    }
}

// a binding (`self`, a param, a local) and the field prefix of the tag its type names
#[derive(Clone)]
struct Binding {
    name: String,
    prefix: FieldPrefix,
}

// the tag-typed names in scope where the visitor stands. `self` comes from the impl self type and
// is shared by every method; every other name is scoped, so a `let`, a closure param, a `for`
// pattern or a match arm that reuses a name replaces what that name meant.
struct Scope {
    self_binding: Option<Binding>,
    in_scope: Vec<Binding>,
}

impl Scope {
    fn new(self_binding: Option<Binding>) -> Self {
        let mut scope = Self {
            self_binding,
            in_scope: Vec::new(),
        };
        scope.reset();

        scope
    }

    // back to `self` alone, which every method shares
    fn reset(&mut self) {
        self.in_scope = self.self_binding.clone().into_iter().collect();
    }

    // enter a method: only `self` and that method's params are in scope, so a param name reused
    // across methods resolves per method
    fn enter_fn(&mut self, sig: &mut Signature, tags: &Tags) {
        self.reset();
        for arg in &mut sig.inputs {
            if let FnArg::Typed(pat_type) = arg {
                self.bind_typed(pat_type, tags);
            }
        }
    }

    // the bindings to put back when the nested scope ends
    fn snapshot(&self) -> Vec<Binding> {
        self.in_scope.clone()
    }

    fn restore(&mut self, outer: Vec<Binding>) {
        self.in_scope = outer;
    }

    // `pat` introduces names: a tag-typed name binds to that tag's prefix, and every other name the
    // pattern covers un-binds, so a shadowed name stops prefixing
    fn bind(&mut self, pat: &mut Pat, tags: &Tags) {
        match pat {
            Pat::Type(pat_type) => self.bind_typed(pat_type, tags),
            pat => self.shadow(pat),
        }
    }

    fn bind_typed(&mut self, pat_type: &mut PatType, tags: &Tags) {
        if let Pat::Ident(pat_ident) = &*pat_type.pat
            && let Some(prefix) = tags.prefix_for_type(&pat_type.ty)
        {
            let binding = Binding {
                name: pat_ident.ident.to_string(),
                prefix: prefix.clone(),
            };
            self.shadow_name(&binding.name);
            self.in_scope.push(binding);
            return;
        }

        self.shadow(&mut pat_type.pat);
    }

    fn shadow(&mut self, pat: &mut Pat) {
        let mut bound = BoundNames::default();
        bound.visit_pat_mut(pat);
        for name in bound.names {
            self.shadow_name(&name);
        }
    }

    fn shadow_name(&mut self, name: &str) {
        self.in_scope.retain(|binding| binding.name != name);
    }

    // the field prefix recorded for a binding name (`self`, a param, a local)
    fn prefix(&self, name: &str) -> Option<&FieldPrefix> {
        self.in_scope
            .iter()
            .find(|binding| binding.name == name)
            .map(|binding| &binding.prefix)
    }
}

// every name a pattern binds. syn's own walk covers each pattern form, including a struct pattern's
// fields and an `ident @ subpattern`.
#[derive(Default)]
struct BoundNames {
    names: Vec<String>,
}

impl VisitMut for BoundNames {
    fn visit_pat_ident_mut(&mut self, node: &mut PatIdent) {
        self.names.push(node.ident.to_string());
        visit_mut::visit_pat_ident_mut(self, node);
    }
}

// pass 1: prefix the field names, while the tags are still readable
struct PrefixFields<'a> {
    tags: &'a Tags,
    fields: Vec<Ident>,
    scope: Scope,
}

impl VisitMut for PrefixFields<'_> {
    fn visit_expr_field_mut(&mut self, node: &mut ExprField) {
        visit_mut::visit_expr_field_mut(self, node);
        // `receiver.field`: prefix the field for the instantiation the receiver holds
        if let Expr::Path(receiver) = &*node.base
            && let Some(binding) = receiver.path.get_ident()
            && let Some(prefix) = self.scope.prefix(&binding.to_string())
            && let Some(renamed) = self.prefixed_member(prefix, &node.member)
        {
            node.member = Member::Named(renamed);
        }
    }

    fn visit_expr_struct_mut(&mut self, node: &mut ExprStruct) {
        visit_mut::visit_expr_struct_mut(self, node);
        if let Some(prefix) = self.literal_field_prefix(&node.path) {
            for field in &mut node.fields {
                if let Some(renamed) = self.prefixed_member(prefix, &field.member) {
                    // a renamed shorthand field (`Self { url }`) must become `renamed: url` so it
                    // still reads the original binding
                    field.colon_token.get_or_insert_default();
                    field.member = Member::Named(renamed);
                }
            }
        }
    }

    fn visit_impl_item_fn_mut(&mut self, node: &mut ImplItemFn) {
        self.scope.enter_fn(&mut node.sig, self.tags);
        visit_mut::visit_impl_item_fn_mut(self, node);
    }

    fn visit_block_mut(&mut self, node: &mut Block) {
        // a `let` inside a block only shadows up to the end of that block
        let outer = self.scope.snapshot();
        visit_mut::visit_block_mut(self, node);
        self.scope.restore(outer);
    }

    fn visit_local_mut(&mut self, node: &mut Local) {
        // the initializer still reads the outer binding: `let value = value.clone();`
        visit_mut::visit_local_mut(self, node);
        self.scope.bind(&mut node.pat, self.tags);
    }

    fn visit_expr_closure_mut(&mut self, node: &mut ExprClosure) {
        let outer = self.scope.snapshot();
        for input in &mut node.inputs {
            self.scope.bind(input, self.tags);
        }
        visit_mut::visit_expr_closure_mut(self, node);
        self.scope.restore(outer);
    }

    fn visit_expr_for_loop_mut(&mut self, node: &mut ExprForLoop) {
        // the iterated expression runs before the loop binds its own pattern
        self.visit_expr_mut(&mut node.expr);

        let outer = self.scope.snapshot();
        self.scope.bind(&mut node.pat, self.tags);
        self.visit_block_mut(&mut node.body);
        self.scope.restore(outer);
    }

    fn visit_arm_mut(&mut self, node: &mut Arm) {
        let outer = self.scope.snapshot();
        self.scope.bind(&mut node.pat, self.tags);
        visit_mut::visit_arm_mut(self, node);
        self.scope.restore(outer);
    }
}

impl<'a> PrefixFields<'a> {
    // `self` binds from the impl self type, so `self.field` and a `Self { .. }` literal resolve
    // through the same lookup as a param
    fn new(tags: &'a Tags, fields: Vec<Ident>, item: &ItemImpl) -> Self {
        let self_binding = tags.prefix_for_type(&item.self_ty).map(|prefix| Binding {
            name: SELF_BINDING.to_string(),
            prefix: prefix.clone(),
        });

        Self {
            tags,
            fields,
            scope: Scope::new(self_binding),
        }
    }

    // prefix to apply to a struct literal's fields, based on the constructed instantiation
    fn literal_field_prefix(&self, path: &Path) -> Option<&FieldPrefix> {
        if let Some(tag) = self.tags.for_path(path) {
            return Some(&tag.field_prefix);
        }
        if path.is_ident("Self") {
            return self.scope.prefix(SELF_BINDING);
        }

        None
    }

    // the `prefix`-joined name for a struct field member, if it is one of the prefixed named fields;
    // `None` for an absent prefix, a tuple field, or a name outside the prefixed set
    fn prefixed_member(&self, prefix: &FieldPrefix, member: &Member) -> Option<Ident> {
        if prefix.value().is_none() {
            return None;
        }
        let Member::Named(ident) = member else {
            return None;
        };

        self.fields.contains(ident).then(|| {
            let prefixed = prefix.field_name(&ident.to_string());

            Ident::new(&prefixed, ident.span())
        })
    }
}

// pass 2: the tag idents become the concrete type paths they resolved to
struct ReplaceTags<'a> {
    tags: &'a Tags,
}

impl VisitMut for ReplaceTags<'_> {
    fn visit_path_mut(&mut self, path: &mut Path) {
        if let Some(tag) = self.tags.for_path(path) {
            *path = tag.path.clone();
            return;
        }
        visit_mut::visit_path_mut(self, path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Span;
    use quote::{format_ident, quote};
    use syn::parse_quote;

    use crate::helper::parse_literal;
    use crate::helper::prefix::{DefaultPrefix, StructPrefix};

    fn ident(name: &str) -> Ident {
        Ident::new(name, Span::call_site())
    }

    // a field prefix as the parser builds one; an empty prefix is the absent one
    fn field_prefix(value: &str) -> FieldPrefix {
        let struct_prefix = match value {
            "" => StructPrefix::default(),
            value => parse_literal(value),
        };

        FieldPrefix::new(&DefaultPrefix::default(), &struct_prefix)
    }

    // a resolved field prefix as a plain string, for readable assertions
    fn value(prefix: Option<&FieldPrefix>) -> Option<&str> {
        prefix.map(|prefix| prefix.value().as_deref().unwrap_or_default())
    }

    // a ResolvedTag with the given tag ident / type path / field prefix
    fn tag(tag: &str, path: &str, prefix: &str) -> ResolvedTag {
        ResolvedTag {
            ident: ident(tag),
            path: syn::parse_str(path).unwrap(),
            field_prefix: field_prefix(prefix),
        }
    }

    // `Base` is the unprefixed `ServerOptions`, `Prefixed` the `svc`-prefixed `SvcServerOptions`
    fn tags() -> Tags {
        Tags {
            base: tag("Base", "ServerOptions", ""),
            prefixed: tag("Prefixed", "SvcServerOptions", "svc"),
        }
    }

    // a pass 1 visitor with one prefixed field `url`, a base binding `base_arg`, a prefixed binding
    // `prefixed_arg`, and `self` bound to the base (so `Self` uses the empty prefix)
    fn prefix_fields(tags: &Tags) -> PrefixFields<'_> {
        PrefixFields {
            tags,
            fields: vec![format_ident!("url")],
            scope: Scope {
                self_binding: None,
                in_scope: vec![
                    Binding {
                        name: "base_arg".to_string(),
                        prefix: field_prefix(""),
                    },
                    Binding {
                        name: "prefixed_arg".to_string(),
                        prefix: field_prefix("svc"),
                    },
                    Binding {
                        name: SELF_BINDING.to_string(),
                        prefix: field_prefix(""),
                    },
                ],
            },
        }
    }

    // the whole rewrite over the tags above, with `url` as the only prefixed field
    fn assert_rewrites(item: ItemImpl, expected: ItemImpl) {
        let tags = tags();
        let rewritten = substitute(item, tags.base, tags.prefixed, vec![format_ident!("url")]);

        assert_eq!(
            quote!(#rewritten).to_string(),
            quote!(#expected).to_string()
        );
    }

    #[test]
    fn prefix_for_type_maps_a_bare_or_referenced_tag_type_to_its_prefix() {
        let tags = tags();
        let prefix = |ty: Type| value(tags.prefix_for_type(&ty));

        assert_eq!(prefix(parse_quote!(Base)), Some(""));
        assert_eq!(prefix(parse_quote!(Prefixed)), Some("svc"));
        // a leading reference is peeled, so `&Prefixed` / `&mut Base` bind like the bare tag
        assert_eq!(prefix(parse_quote!(&Prefixed)), Some("svc"));
        assert_eq!(prefix(parse_quote!(&mut Base)), Some(""));
        assert_eq!(prefix(parse_quote!(String)), None);
        // other wrappers are not recognized as tags
        assert_eq!(prefix(parse_quote!(Option<Prefixed>)), None);
        assert_eq!(prefix(parse_quote!(Box<Prefixed>)), None);
    }

    #[test]
    fn for_path_matches_tag_idents() {
        let tags = tags();
        let prefix = |path: Path| value(tags.for_path(&path).map(|tag| &tag.field_prefix));

        assert_eq!(prefix(parse_quote!(Prefixed)), Some("svc"));
        assert_eq!(prefix(parse_quote!(Base)), Some(""));
        assert_eq!(prefix(parse_quote!(Other)), None);
    }

    #[test]
    fn scope_prefix_looks_up_the_names_in_scope() {
        let tags = tags();
        let visitor = prefix_fields(&tags);

        // `prefixed_arg` is a prefixed binding, `base_arg` a base binding
        assert_eq!(value(visitor.scope.prefix("prefixed_arg")), Some("svc"));
        assert_eq!(value(visitor.scope.prefix("base_arg")), Some(""));
        // a name that is not in scope yields nothing
        assert_eq!(value(visitor.scope.prefix("unknown")), None);
    }

    #[test]
    fn literal_field_prefix_covers_tags_and_self() {
        let tags = tags();
        let visitor = prefix_fields(&tags);
        let prefix = |path: Path| value(visitor.literal_field_prefix(&path));

        assert_eq!(prefix(parse_quote!(Prefixed)), Some("svc"));
        assert_eq!(prefix(parse_quote!(Base)), Some(""));
        // `Self` resolves through the `self` binding (base -> empty here)
        assert_eq!(prefix(parse_quote!(Self)), Some(""));
        assert_eq!(prefix(parse_quote!(Other)), None);

        // with no `self` binding, `Self` yields nothing
        let mut visitor = prefix_fields(&tags);
        visitor.scope.shadow_name(SELF_BINDING);
        assert_eq!(
            value(visitor.literal_field_prefix(&parse_quote!(Self))),
            None
        );
    }

    #[test]
    fn prefixed_member_renames_only_known_named_fields() {
        let tags = tags();
        let visitor = prefix_fields(&tags);
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

    #[test]
    fn a_param_of_a_tag_type_prefixes_its_field_reads() {
        assert_rewrites(
            parse_quote! {
                impl From<Prefixed> for Base {
                    fn from(value: Prefixed) -> Self {
                        Self { url: value.url }
                    }
                }
            },
            parse_quote! {
                impl From<SvcServerOptions> for ServerOptions {
                    fn from(value: SvcServerOptions) -> Self {
                        Self { url: value.svc_url }
                    }
                }
            },
        );
    }

    #[test]
    fn a_shadowing_let_stops_the_prefixing_of_that_name() {
        assert_rewrites(
            parse_quote! {
                impl From<Prefixed> for Base {
                    fn from(value: Prefixed) -> Self {
                        // `value` is a `Fallback` from here on, so its field keeps its own name
                        let value = Fallback::default();
                        Self { url: value.url }
                    }
                }
            },
            parse_quote! {
                impl From<SvcServerOptions> for ServerOptions {
                    fn from(value: SvcServerOptions) -> Self {
                        let value = Fallback::default();
                        Self { url: value.url }
                    }
                }
            },
        );
    }

    #[test]
    fn a_let_shadows_up_to_the_end_of_its_own_block_only() {
        assert_rewrites(
            parse_quote! {
                impl From<Prefixed> for Base {
                    fn from(value: Prefixed) -> Self {
                        let inner = {
                            let value = Fallback::default();
                            value.url
                        };
                        Self { url: value.url }
                    }
                }
            },
            parse_quote! {
                impl From<SvcServerOptions> for ServerOptions {
                    fn from(value: SvcServerOptions) -> Self {
                        let inner = {
                            let value = Fallback::default();
                            value.url
                        };
                        Self { url: value.svc_url }
                    }
                }
            },
        );
    }

    #[test]
    fn a_let_of_a_tag_type_binds_that_tag() {
        assert_rewrites(
            parse_quote! {
                impl From<Base> for Prefixed {
                    fn from(value: Base) -> Self {
                        let value: Prefixed = build();
                        Prefixed { url: value.url }
                    }
                }
            },
            parse_quote! {
                impl From<ServerOptions> for SvcServerOptions {
                    fn from(value: ServerOptions) -> Self {
                        let value: SvcServerOptions = build();
                        SvcServerOptions { svc_url: value.svc_url }
                    }
                }
            },
        );
    }

    #[test]
    fn a_closure_param_shadows_or_binds_like_a_let() {
        assert_rewrites(
            parse_quote! {
                impl From<Prefixed> for Base {
                    fn from(value: Prefixed) -> Self {
                        let plain = |value: Fallback| value.url;
                        let tagged = |value: Prefixed| value.url;
                        Self { url: value.url }
                    }
                }
            },
            parse_quote! {
                impl From<SvcServerOptions> for ServerOptions {
                    fn from(value: SvcServerOptions) -> Self {
                        let plain = |value: Fallback| value.url;
                        let tagged = |value: SvcServerOptions| value.svc_url;
                        Self { url: value.svc_url }
                    }
                }
            },
        );
    }

    #[test]
    fn a_for_pattern_and_a_match_arm_shadow_for_their_own_body_only() {
        assert_rewrites(
            parse_quote! {
                impl From<Prefixed> for Base {
                    fn from(value: Prefixed) -> Self {
                        for value in value.url {
                            drop(value.url);
                        }
                        let url = match other {
                            Some(value) => value.url,
                            None => value.url,
                        };
                        Self { url }
                    }
                }
            },
            parse_quote! {
                impl From<SvcServerOptions> for ServerOptions {
                    fn from(value: SvcServerOptions) -> Self {
                        for value in value.svc_url {
                            drop(value.url);
                        }
                        let url = match other {
                            Some(value) => value.url,
                            None => value.svc_url,
                        };
                        Self { url }
                    }
                }
            },
        );
    }
}
