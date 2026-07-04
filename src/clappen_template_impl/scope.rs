// The tag-typed names at each point of pass 1, so it knows whose fields to prefix.

use syn::{
    Expr, FnArg, GenericArgument, Ident, ImplItem, ItemImpl, Pat, Path, PathArguments, Signature,
    Type,
};

use crate::clappen_template_impl::resolve::Tags;
use crate::helper::prefix::FieldPrefix;

// a tag-typed name and its field prefix
#[derive(Clone)]
struct Binding {
    key: BindingKey,
    prefix: FieldPrefix,
}

// the key of a binding
#[derive(Clone, PartialEq)]
enum BindingKey {
    // `self`, also read by `Self`
    SelfValue,
    // the impl's `type X = Tag;`, read by `Self::X`
    Assoc(Ident),
    // a param, a `let` or a closure param
    Local(Ident),
}

// the tag-typed names of the current method, up to its end:
// - `self`, and the impl's `type X = Tag;` as `Self::X`
// - each `name: Tag` param, `let` or closure param
// - an untyped binding (`let value = Fallback::default();`, `Some(value) =>`) changes nothing,
//   so a tag-typed name cannot be reused for another type
#[derive(Clone, Default)]
pub(crate) struct Scope {
    in_scope: Vec<Binding>,
}

impl Scope {
    // `self` gets the impl type's tag, so `self.field` and `Self { .. }` resolve like a param
    pub(crate) fn for_impl(item: &ItemImpl, tags: &Tags) -> Self {
        let mut scope = Self::default();
        if let Some(prefix) = scope.type_prefix(&item.self_ty, tags) {
            scope.bind_key(BindingKey::SelfValue, prefix.clone());
        }
        for impl_item in &item.items {
            if let ImplItem::Type(assoc) = impl_item
                && let Some(prefix) = scope.type_prefix(&assoc.ty, tags)
            {
                scope.bind_key(BindingKey::Assoc(assoc.ident.clone()), prefix.clone());
            }
        }

        scope
    }

    // a method: the impl's `self` and `Self::X`, then its params
    pub(crate) fn for_method(&self, sig: &Signature, tags: &Tags) -> Self {
        let mut scope = self.clone();
        scope.bind_params(sig, tags);

        scope
    }

    // a `let` or a closure param: only a typed one binds
    pub(crate) fn bind(&mut self, pat: &Pat, tags: &Tags) {
        if let Pat::Type(pat_type) = pat {
            self.bind_typed(&pat_type.pat, &pat_type.ty, tags);
        }
    }

    // the prefix of a tag, or of `Self` / `Self::X` when they name a tag
    pub(crate) fn path_prefix<'s>(
        &'s self,
        path: &Path,
        tags: &'s Tags,
    ) -> Option<&'s FieldPrefix> {
        match tags.for_path(path) {
            Some(tag) => Some(&tag.field_prefix),
            None => self.self_prefix(path),
        }
    }

    // the field prefix of a name in scope that an expression reads: `value` in `value.url`
    pub(crate) fn expr_prefix(&self, expr: &Expr) -> Option<&FieldPrefix> {
        let Expr::Path(path) = expr else {
            return None;
        };
        self.prefix(path.path.get_ident()?)
    }

    // the field prefix of a name read in the code
    pub(crate) fn prefix(&self, ident: &Ident) -> Option<&FieldPrefix> {
        let key = if ident == "self" {
            BindingKey::SelfValue
        } else {
            BindingKey::Local(ident.clone())
        };
        self.key_prefix(&key)
    }

    // the params of a method
    fn bind_params(&mut self, sig: &Signature, tags: &Tags) {
        for arg in &sig.inputs {
            if let FnArg::Typed(pat_type) = arg {
                self.bind_typed(&pat_type.pat, &pat_type.ty, tags);
            }
        }
    }

    // `name: Tag` binds the tag's prefix; `(a, b): (Prefixed, u8)` and `[a, b]: [Prefixed; 2]`
    // bind each element
    fn bind_typed(&mut self, pat: &Pat, ty: &Type, tags: &Tags) {
        match (pat, ty) {
            (Pat::Ident(pat_ident), ty) => {
                if let Some(prefix) = self.type_prefix(ty, tags) {
                    self.bind_key(BindingKey::Local(pat_ident.ident.clone()), prefix.clone());
                }
            }
            (Pat::Tuple(pats), Type::Tuple(types)) => {
                for (pat, ty) in pats.elems.iter().zip(&types.elems) {
                    self.bind_typed(pat, ty, tags);
                }
            }
            (Pat::Slice(pats), Type::Array(array)) => {
                for pat in &pats.elems {
                    self.bind_typed(pat, &array.elem, tags);
                }
            }
            (Pat::Slice(pats), Type::Slice(slice)) => {
                for pat in &pats.elems {
                    self.bind_typed(pat, &slice.elem, tags);
                }
            }
            (pat, Type::Reference(reference)) => self.bind_typed(pat, &reference.elem, tags),
            _ => {}
        }
    }

    // in place of an earlier binding with that key
    fn bind_key(&mut self, key: BindingKey, prefix: FieldPrefix) {
        self.in_scope.retain(|binding| binding.key != key);
        self.in_scope.push(Binding { key, prefix });
    }

    // the prefix of a type, with the wrappers `inner_type` skips: `&Prefixed`, `Box<Self>`
    fn type_prefix<'s>(&'s self, ty: &Type, tags: &'s Tags) -> Option<&'s FieldPrefix> {
        match inner_type(ty) {
            Type::Path(type_path) => self.path_prefix(&type_path.path, tags),
            _ => None,
        }
    }

    // `Self` and `Self::X` resolve through the impl's own bindings
    fn self_prefix(&self, path: &Path) -> Option<&FieldPrefix> {
        let mut segments = path.segments.iter().map(|segment| &segment.ident);
        match (segments.next(), segments.next(), segments.next()) {
            (Some(first), None, _) if first == "Self" => self.key_prefix(&BindingKey::SelfValue),
            (Some(first), Some(assoc), None) if first == "Self" => {
                self.key_prefix(&BindingKey::Assoc(assoc.clone()))
            }
            _ => None,
        }
    }

    // the field prefix of a key in scope
    fn key_prefix(&self, key: &BindingKey) -> Option<&FieldPrefix> {
        self.in_scope
            .iter()
            .find(|binding| &binding.key == key)
            .map(|binding| &binding.prefix)
    }
}

// `&Prefixed`, `Box<Prefixed>`, `Rc<Prefixed>` and `Arc<Prefixed>` read their fields like a
// `Prefixed`; other wrappers (`Option<Prefixed>`) do not
fn inner_type(ty: &Type) -> &Type {
    match ty {
        Type::Reference(reference) => inner_type(&reference.elem),
        Type::Path(type_path) => {
            if let Some(last) = type_path.path.segments.last()
                && ["Box", "Rc", "Arc"].iter().any(|name| last.ident == name)
                && let PathArguments::AngleBracketed(args) = &last.arguments
                && args.args.len() == 1
                && let Some(GenericArgument::Type(inner)) = args.args.first()
            {
                return inner_type(inner);
            }
            ty
        }
        ty => ty,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::format_ident;
    use syn::parse_quote;

    use crate::clappen_template_impl::resolve::test_helpers::{field_prefix, tags, value};

    // a base binding `base_arg`, a prefixed binding `prefixed_arg`, and `self` bound to the base
    // (so `Self` uses the empty prefix)
    fn scope() -> Scope {
        Scope {
            in_scope: vec![
                Binding {
                    key: BindingKey::Local(format_ident!("base_arg")),
                    prefix: field_prefix(""),
                },
                Binding {
                    key: BindingKey::Local(format_ident!("prefixed_arg")),
                    prefix: field_prefix("svc"),
                },
                Binding {
                    key: BindingKey::SelfValue,
                    prefix: field_prefix(""),
                },
            ],
        }
    }

    #[test]
    fn type_prefix_maps_a_bare_or_referenced_tag_type_to_its_prefix() {
        let tags = tags();
        let scope = Scope::default();
        let prefix = |ty: Type| value(scope.type_prefix(&ty, &tags));

        assert_eq!(prefix(parse_quote!(Base)), Some(""));
        assert_eq!(prefix(parse_quote!(Prefixed)), Some("svc"));
        // a leading reference is ignored, so `&Prefixed` / `&mut Base` bind like the bare tag
        assert_eq!(prefix(parse_quote!(&Prefixed)), Some("svc"));
        assert_eq!(prefix(parse_quote!(&mut Base)), Some(""));
        assert_eq!(prefix(parse_quote!(String)), None);
        // `Box`, `Rc` and `Arc` are skipped like a reference, other wrappers are not tags
        assert_eq!(prefix(parse_quote!(Box<Prefixed>)), Some("svc"));
        assert_eq!(prefix(parse_quote!(std::sync::Arc<&Prefixed>)), Some("svc"));
        assert_eq!(prefix(parse_quote!(Option<Prefixed>)), None);
        // generic args do not change the tag
        assert_eq!(prefix(parse_quote!(Prefixed<T>)), Some("svc"));
    }

    #[test]
    fn scope_prefix_looks_up_the_names_in_scope() {
        let scope = scope();
        let prefix = |name: &str| value(scope.prefix(&format_ident!("{name}")));

        // `prefixed_arg` is a prefixed binding, `base_arg` a base binding
        assert_eq!(prefix("prefixed_arg"), Some("svc"));
        assert_eq!(prefix("base_arg"), Some(""));
        // `self` reads the `self` binding
        assert_eq!(prefix("self"), Some(""));
        // a name that is not in scope yields nothing
        assert_eq!(prefix("unknown"), None);
    }

    #[test]
    fn path_prefix_covers_tags_and_self() {
        let tags = tags();
        let scope = scope();
        let prefix = |path: Path| value(scope.path_prefix(&path, &tags));

        assert_eq!(prefix(parse_quote!(Prefixed)), Some("svc"));
        assert_eq!(prefix(parse_quote!(Base)), Some(""));
        // `Self` resolves through the `self` binding (base -> empty here)
        assert_eq!(prefix(parse_quote!(Self)), Some(""));
        assert_eq!(prefix(parse_quote!(Other)), None);

        // with no `self` binding, `Self` yields nothing
        let empty = Scope::default();
        assert_eq!(value(empty.path_prefix(&parse_quote!(Self), &tags)), None);
    }
}
