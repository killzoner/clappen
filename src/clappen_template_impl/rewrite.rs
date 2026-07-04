// Rewriting: walk the impl body and (1) replace the `Base`/`Prefixed` tags with their concrete
// type paths, (2) prefix field reads off a tagged binding, (3) prefix struct-literal fields.

use syn::visit_mut::{self, VisitMut};
use syn::{
    Expr, ExprField, ExprStruct, FnArg, Ident, ImplItemFn, ItemImpl, Member, Pat, Path, Type,
};

use crate::clappen_template_impl::resolve::ResolvedTag;
use crate::helper;

const SELF_BINDING: &str = "self";

// rewrite the base/prefixed tags to their paths and prefix the relevant field accesses /
// struct-literal fields
pub(crate) fn substitute(
    mut item: ItemImpl,
    base: ResolvedTag,
    prefixed: ResolvedTag,
    fields: Vec<String>,
) -> ItemImpl {
    let mut subst = TemplateSubst::new(base, prefixed, fields, &item);
    subst.visit_item_impl_mut(&mut item);
    item
}

// a binding (`self` or a param) and the field prefix of the tag its type names
#[derive(Clone)]
struct Binding {
    name: String,
    prefix: String,
}

struct TemplateSubst {
    base: ResolvedTag,
    prefixed: ResolvedTag,
    fields: Vec<String>,
    // `self` (from the impl self type), shared by every method
    self_binding: Option<Binding>,
    // the current method's bindings: `self_binding` plus that method's typed params, rebuilt on
    // entering each method so a param name reused across methods resolves per method
    bindings: Vec<Binding>,
}

impl VisitMut for TemplateSubst {
    fn visit_path_mut(&mut self, path: &mut Path) {
        if let Some(tag) = self.tag_for_path(path) {
            *path = tag.path.clone();
            return;
        }
        visit_mut::visit_path_mut(self, path);
    }

    fn visit_expr_field_mut(&mut self, node: &mut ExprField) {
        visit_mut::visit_expr_field_mut(self, node);
        // `receiver.field`: prefix the field for the instantiation the receiver holds
        if let Expr::Path(receiver) = &*node.base
            && let Some(binding) = receiver.path.get_ident()
            && let Some(prefix) = self.prefix_for_binding(&binding.to_string())
            && let Some(renamed) = self.prefixed_member(prefix, &node.member)
        {
            node.member = Member::Named(renamed);
        }
    }

    fn visit_expr_struct_mut(&mut self, node: &mut ExprStruct) {
        // prefix the constructed fields before recursing: `node.path` still names the base/prefixed
        // tag here, and the recursion below rewrites it to a concrete type path via `visit_path_mut`
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
        visit_mut::visit_expr_struct_mut(self, node);
    }

    fn visit_impl_item_fn_mut(&mut self, node: &mut ImplItemFn) {
        // scope the bindings to this method: the shared `self` plus this method's typed params
        self.bindings = self.self_binding.clone().into_iter().collect();
        for arg in &node.sig.inputs {
            if let FnArg::Typed(pat_type) = arg
                && let Pat::Ident(pat_ident) = &*pat_type.pat
                && let Some(binding) = self.binding(pat_ident.ident.to_string(), &pat_type.ty)
            {
                self.bindings.push(binding);
            }
        }
        visit_mut::visit_impl_item_fn_mut(self, node);
    }
}

impl TemplateSubst {
    // the two resolved tags plus the shared `self` binding (from the impl self type). Each method's
    // param bindings are added per method in `visit_impl_item_fn_mut`, so `self.field`, a
    // `Self { .. }` / tag literal, and a param field read all resolve through the same lookup
    fn new(base: ResolvedTag, prefixed: ResolvedTag, fields: Vec<String>, item: &ItemImpl) -> Self {
        let mut subst = Self {
            base,
            prefixed,
            fields,
            self_binding: None,
            bindings: Vec::new(),
        };
        subst.self_binding = subst.binding(SELF_BINDING.to_string(), &item.self_ty);
        subst
    }

    // the resolved tag a path names, if it is a bare `base`/`prefixed` tag ident
    fn tag_for_path(&self, path: &Path) -> Option<&ResolvedTag> {
        [&self.base, &self.prefixed]
            .into_iter()
            .find(|tag| path.is_ident(&tag.ident))
    }

    // a binding for `name`, if `ty` names the base or prefixed tag. A leading `&`/`&mut` is peeled,
    // so `&Prefixed` binds like `Prefixed`; other wrappers (`Option<Prefixed>`, `Box<Prefixed>`) are
    // not recognized, so their field reads stay unprefixed.
    fn binding(&self, name: String, ty: &Type) -> Option<Binding> {
        let mut ty = ty;
        while let Type::Reference(reference) = ty {
            ty = &reference.elem;
        }
        let Type::Path(path) = ty else { return None };
        self.tag_for_path(&path.path).map(|tag| Binding {
            name,
            prefix: tag.field_prefix.clone(),
        })
    }

    // the field prefix recorded for a binding name (`self` or a param)
    fn prefix_for_binding(&self, name: &str) -> Option<&str> {
        self.bindings
            .iter()
            .find(|binding| binding.name == name)
            .map(|binding| binding.prefix.as_str())
    }

    // prefix to apply to a struct literal's fields, based on the constructed instantiation
    fn literal_field_prefix(&self, path: &Path) -> Option<&str> {
        if let Some(tag) = self.tag_for_path(path) {
            return Some(tag.field_prefix.as_str());
        }
        if path.is_ident("Self") {
            return self.prefix_for_binding(SELF_BINDING);
        }
        None
    }

    // the `prefix`-joined name for a struct field member, if it is one of the prefixed named fields;
    // `None` for an empty prefix, a tuple field, or a name outside the prefixed set
    fn prefixed_member(&self, prefix: &str, member: &Member) -> Option<Ident> {
        if prefix.is_empty() {
            return None;
        }
        let Member::Named(ident) = member else {
            return None;
        };
        let name = ident.to_string();
        self.fields
            .contains(&name)
            .then(|| Ident::new(&helper::prefixed_field(prefix, &name), ident.span()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Span;
    use syn::parse_quote;

    fn ident(name: &str) -> Ident {
        Ident::new(name, Span::call_site())
    }

    // a ResolvedTag with the given tag ident / type path / field prefix
    fn tag(tag: &str, path: &str, field_prefix: &str) -> ResolvedTag {
        ResolvedTag {
            ident: ident(tag),
            path: syn::parse_str(path).unwrap(),
            field_prefix: field_prefix.to_string(),
        }
    }

    // a visitor with `Base`/`Prefixed` tags, one prefixed field `url`, a base binding `base_arg`, a
    // prefixed binding `prefixed_arg`, and `self` bound to the base (so `Self` uses the empty prefix)
    fn subst() -> TemplateSubst {
        TemplateSubst {
            base: tag("Base", "ServerOptions", ""),
            prefixed: tag("Prefixed", "SvcServerOptions", "svc"),
            fields: vec!["url".to_string()],
            self_binding: None,
            bindings: vec![
                Binding {
                    name: "base_arg".to_string(),
                    prefix: String::new(),
                },
                Binding {
                    name: "prefixed_arg".to_string(),
                    prefix: "svc".to_string(),
                },
                Binding {
                    name: "self".to_string(),
                    prefix: String::new(),
                },
            ],
        }
    }

    #[test]
    fn binding_maps_a_bare_or_referenced_tag_type_to_its_prefix() {
        let visitor = subst();
        let prefix = |ty: Type| visitor.binding("x".to_string(), &ty).map(|b| b.prefix);

        assert_eq!(prefix(parse_quote!(Base)).as_deref(), Some(""));
        assert_eq!(prefix(parse_quote!(Prefixed)).as_deref(), Some("svc"));
        // a leading reference is peeled, so `&Prefixed` / `&mut Base` bind like the bare tag
        assert_eq!(prefix(parse_quote!(&Prefixed)).as_deref(), Some("svc"));
        assert_eq!(prefix(parse_quote!(&mut Base)).as_deref(), Some(""));
        assert!(prefix(parse_quote!(String)).is_none());
        // other wrappers are not recognized as tags
        assert!(prefix(parse_quote!(Option<Prefixed>)).is_none());
        assert!(prefix(parse_quote!(Box<Prefixed>)).is_none());
    }

    #[test]
    fn tag_for_path_matches_tag_idents() {
        let visitor = subst();
        assert_eq!(
            visitor
                .tag_for_path(&parse_quote!(Prefixed))
                .unwrap()
                .field_prefix,
            "svc"
        );
        assert_eq!(
            visitor
                .tag_for_path(&parse_quote!(Base))
                .unwrap()
                .field_prefix,
            ""
        );
        assert!(visitor.tag_for_path(&parse_quote!(Other)).is_none());
    }

    #[test]
    fn prefix_for_binding_looks_up_recorded_bindings() {
        let visitor = subst();
        // `prefixed_arg` is a prefixed binding, `base_arg` a base binding
        assert_eq!(visitor.prefix_for_binding("prefixed_arg"), Some("svc"));
        assert_eq!(visitor.prefix_for_binding("base_arg"), Some(""));
        // an unrecorded binding yields nothing
        assert_eq!(visitor.prefix_for_binding("unknown"), None);
    }

    #[test]
    fn literal_field_prefix_covers_tags_and_self() {
        let visitor = subst();
        assert_eq!(
            visitor.literal_field_prefix(&parse_quote!(Prefixed)),
            Some("svc")
        );
        assert_eq!(visitor.literal_field_prefix(&parse_quote!(Base)), Some(""));
        // `Self` resolves through the `self` binding (base -> empty here)
        assert_eq!(visitor.literal_field_prefix(&parse_quote!(Self)), Some(""));
        assert_eq!(visitor.literal_field_prefix(&parse_quote!(Other)), None);

        // with no `self` binding, `Self` yields nothing
        let mut visitor = subst();
        visitor.bindings.retain(|binding| binding.name != "self");
        assert_eq!(visitor.literal_field_prefix(&parse_quote!(Self)), None);
    }

    #[test]
    fn prefixed_member_renames_only_known_named_fields() {
        let visitor = subst();

        // a prefixed field with a non-empty prefix
        let member: Member = parse_quote!(url);
        assert_eq!(
            visitor
                .prefixed_member("svc", &member)
                .map(|i| i.to_string()),
            Some("svc_url".to_string())
        );

        // empty prefix
        let member: Member = parse_quote!(url);
        assert!(visitor.prefixed_member("", &member).is_none());

        // a field not in the prefixed set
        let member: Member = parse_quote!(other);
        assert!(visitor.prefixed_member("svc", &member).is_none());

        // a tuple field
        let member: Member = parse_quote!(0);
        assert!(visitor.prefixed_member("svc", &member).is_none());
    }
}
