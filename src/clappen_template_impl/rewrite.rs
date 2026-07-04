// Two passes, because prefixing a field reads the tags and replacing a tag removes them:
// (1) `PrefixFields` prefixes the fields of tag-typed values, (2) `ReplaceTags` swaps the tags for
// their type paths.

use syn::visit_mut::{self, VisitMut};
use syn::{
    Arm, Block, Expr, ExprClosure, ExprField, ExprForLoop, ExprStruct, FnArg, Ident, ImplItemFn,
    ItemImpl, Local, Member, Pat, PatIdent, PatType, Path, Signature, Type,
};

use crate::clappen_template_impl::resolve::ResolvedTag;
use crate::helper::{PrefixValue, prefix::FieldPrefix};

const SELF_BINDING: &str = "self";

// the two tags, looked up by ident
pub(crate) struct Tags {
    pub base: ResolvedTag,
    pub prefixed: ResolvedTag,
}

impl Tags {
    // the tag a bare ident path names
    fn for_path(&self, path: &Path) -> Option<&ResolvedTag> {
        [&self.base, &self.prefixed]
            .into_iter()
            .find(|tag| path.is_ident(&tag.ident))
    }

    // the field prefix for a value of type `ty`: `&`/`&mut` are skipped, other wrappers
    // (`Option<Prefixed>`) are not tags
    fn prefix_for_type(&self, ty: &Type) -> Option<&FieldPrefix> {
        let mut ty = ty;
        while let Type::Reference(reference) = ty {
            ty = &reference.elem;
        }
        let Type::Path(path) = ty else { return None };

        self.for_path(&path.path).map(|tag| &tag.field_prefix)
    }
}

// a tag-typed name (`self`, a param, a local) and its field prefix
#[derive(Clone)]
struct Binding {
    name: String,
    prefix: FieldPrefix,
}

// the tag-typed names in scope: `self` in every method, the others scoped, so a name that a `let`,
// a closure param, a `for` pattern or a match arm reuses stops meaning the tag
struct Scope {
    in_scope: Vec<Binding>,
}

impl Scope {
    fn new(self_binding: Option<Binding>) -> Self {
        Self {
            in_scope: self_binding.into_iter().collect(),
        }
    }

    // a method's params, on top of `self`
    fn bind_params(&mut self, sig: &mut Signature, tags: &Tags) {
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

    // a tag-typed name binds its tag's prefix; every other name in `pat` stops prefixing
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

    // the field prefix of a name in scope
    fn prefix(&self, name: &str) -> Option<&FieldPrefix> {
        self.in_scope
            .iter()
            .find(|binding| binding.name == name)
            .map(|binding| &binding.prefix)
    }
}

// every name a pattern binds; syn's walk covers every pattern form
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
pub(crate) struct PrefixFields<'a> {
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
                    // a renamed shorthand `Self { url }` needs `renamed: url` to read `url`
                    field.colon_token.get_or_insert_default();
                    field.member = Member::Named(renamed);
                }
            }
        }
    }

    fn visit_impl_item_fn_mut(&mut self, node: &mut ImplItemFn) {
        // each method starts from `self` alone, so a param name reused across methods resolves
        // per method
        let outer = self.scope.snapshot();
        self.scope.bind_params(&mut node.sig, self.tags);
        visit_mut::visit_impl_item_fn_mut(self, node);
        self.scope.restore(outer);
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
        visit_mut::visit_block_mut(self, &mut node.body);
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
    // `self` gets the impl type's tag, so `self.field` and `Self { .. }` resolve like a param
    pub(crate) fn new(tags: &'a Tags, fields: Vec<Ident>, item: &ItemImpl) -> Self {
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

    // the field prefix for a struct literal's fields
    fn literal_field_prefix(&self, path: &Path) -> Option<&FieldPrefix> {
        if let Some(tag) = self.tags.for_path(path) {
            return Some(&tag.field_prefix);
        }
        if path.is_ident("Self") {
            return self.scope.prefix(SELF_BINDING);
        }

        None
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
            let prefixed = prefix.field_name(&ident.to_string());

            Ident::new(&prefixed, ident.span())
        })
    }
}

// pass 2: the tags become their type paths
pub(crate) struct ReplaceTags<'a> {
    pub tags: &'a Tags,
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
    use quote::format_ident;
    use syn::parse_quote;

    use crate::helper::parse_literal;
    use crate::helper::prefix::{DefaultPrefix, StructPrefix};

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
        prefix.map(PrefixValue::as_str)
    }

    fn tag(tag: &str, path: &str, prefix: &str) -> ResolvedTag {
        ResolvedTag {
            ident: format_ident!("{tag}"),
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

    #[test]
    fn prefix_for_type_maps_a_bare_or_referenced_tag_type_to_its_prefix() {
        let tags = tags();
        let prefix = |ty: Type| value(tags.prefix_for_type(&ty));

        assert_eq!(prefix(parse_quote!(Base)), Some(""));
        assert_eq!(prefix(parse_quote!(Prefixed)), Some("svc"));
        // a leading reference is ignored, so `&Prefixed` / `&mut Base` bind like the bare tag
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
}
