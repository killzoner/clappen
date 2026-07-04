// The tag-typed names at each point of pass 1, so it knows whose fields to prefix.

use syn::{
    Expr, FnArg, GenericArgument, Ident, ImplItem, ItemImpl, Local, Pat, Path, PathArguments,
    Signature, Type, UnOp,
};

use crate::clappen_template_impl::resolve::Tags;
use crate::helper::prefix::FieldPrefix;

const SELF_BINDING: &str = "self";

// the field prefix for a value of type `ty`, from the tags alone
fn prefix_for_type<'t>(ty: &Type, tags: &'t Tags) -> Option<&'t FieldPrefix> {
    let Type::Path(type_path) = inner_type(ty) else {
        return None;
    };

    tags.for_path(&type_path.path).map(|tag| &tag.field_prefix)
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

// the name a field read or a `let` copies from: `value` in `value`, `(*value)`, `&value` or
// `value.clone()`
fn bound_name(expr: &Expr) -> Option<&Ident> {
    match expr {
        Expr::Path(path) => path.path.get_ident(),
        Expr::Paren(paren) => bound_name(&paren.expr),
        Expr::Reference(reference) => bound_name(&reference.expr),
        Expr::Unary(unary) if matches!(unary.op, UnOp::Deref(_)) => bound_name(&unary.expr),
        Expr::MethodCall(call) if call.method == "clone" && call.args.is_empty() => {
            bound_name(&call.receiver)
        }
        _ => None,
    }
}

// a tag-typed name (`self`, a param, a local) and its field prefix
#[derive(Clone)]
struct Binding {
    name: String,
    prefix: FieldPrefix,
}

// the tag-typed names of the current method: `self` and the impl's `type X = Tag;` (as
// `Self::X`), then each `name: Tag` param, `let` or closure param up to the end of the method; an
// untyped binding (`let value = other;`, `Some(value) =>`) changes nothing, so a tag-typed name
// cannot be reused for another type
#[derive(Clone, Default)]
pub(crate) struct Scope {
    in_scope: Vec<Binding>,
}

impl Scope {
    // `self` gets the impl type's tag, so `self.field` and `Self { .. }` resolve like a param
    pub(crate) fn for_impl(item: &ItemImpl, tags: &Tags) -> Self {
        let mut scope = Self::default();
        if let Some(prefix) = prefix_for_type(&item.self_ty, tags) {
            scope.bind_name(SELF_BINDING.to_string(), prefix.clone());
        }
        for impl_item in &item.items {
            if let ImplItem::Type(assoc) = impl_item
                && let Some(prefix) = prefix_for_type(&assoc.ty, tags)
            {
                scope.bind_name(format!("Self::{}", assoc.ident), prefix.clone());
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

    // a `fn` in a method body: its params only
    pub(crate) fn for_fn(sig: &Signature, tags: &Tags) -> Self {
        let mut scope = Self::default();
        scope.bind_params(sig, tags);

        scope
    }

    // the params of a method or of a `fn` in a method body
    fn bind_params(&mut self, sig: &Signature, tags: &Tags) {
        for arg in &sig.inputs {
            if let FnArg::Typed(pat_type) = arg {
                self.bind_typed(&pat_type.pat, &pat_type.ty, tags);
            }
        }
    }

    // `let other: &Prefixed = ..;` binds by type, `let other = value.clone();` copies the binding
    // of `value`
    pub(crate) fn bind_local(&mut self, local: &Local, tags: &Tags) {
        if let Pat::Ident(pat_ident) = &local.pat
            && let Some(init) = &local.init
            && let Some(prefix) = self.expr_prefix(&init.expr)
        {
            self.bind_name(pat_ident.ident.to_string(), prefix.clone());
        }
        self.bind(&local.pat, tags);
    }

    // a `let` or a closure param: only a typed one binds
    pub(crate) fn bind(&mut self, pat: &Pat, tags: &Tags) {
        if let Pat::Type(pat_type) = pat {
            self.bind_typed(&pat_type.pat, &pat_type.ty, tags);
        }
    }

    // `name: Tag` binds the tag's prefix; `(a, b): (Prefixed, u8)` and `[a, b]: [Prefixed; 2]`
    // bind each element
    fn bind_typed(&mut self, pat: &Pat, ty: &Type, tags: &Tags) {
        match (pat, ty) {
            (Pat::Ident(pat_ident), ty) => {
                if let Some(prefix) = self.type_prefix(ty, tags) {
                    self.bind_name(pat_ident.ident.to_string(), prefix);
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
            _ => {}
        }
    }

    // in place of an earlier binding of that name
    fn bind_name(&mut self, name: String, prefix: FieldPrefix) {
        self.in_scope.retain(|binding| binding.name != name);
        self.in_scope.push(Binding { name, prefix });
    }

    // the prefix of a type, with the wrappers `inner_type` skips: `&Prefixed`, `Box<Self>`
    fn type_prefix(&self, ty: &Type, tags: &Tags) -> Option<FieldPrefix> {
        match inner_type(ty) {
            Type::Path(type_path) => self.path_prefix(&type_path.path, tags).cloned(),
            _ => None,
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

    // `Self` and `Self::X` resolve through the impl's own bindings
    fn self_prefix(&self, path: &Path) -> Option<&FieldPrefix> {
        let mut segments = path.segments.iter().map(|segment| &segment.ident);
        match (segments.next(), segments.next(), segments.next()) {
            (Some(first), None, _) if first == "Self" => self.prefix(SELF_BINDING),
            (Some(first), Some(assoc), None) if first == "Self" => {
                self.prefix(&format!("Self::{assoc}"))
            }
            _ => None,
        }
    }

    // the field prefix of the name an expression reads (see `bound_name`)
    pub(crate) fn expr_prefix(&self, expr: &Expr) -> Option<&FieldPrefix> {
        self.prefix(&bound_name(expr)?.to_string())
    }

    // the field prefix of a name in scope
    pub(crate) fn prefix(&self, name: &str) -> Option<&FieldPrefix> {
        self.in_scope
            .iter()
            .find(|binding| binding.name == name)
            .map(|binding| &binding.prefix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    use crate::clappen_template_impl::resolve::test_helpers::{field_prefix, tags, value};

    // a base binding `base_arg`, a prefixed binding `prefixed_arg`, and `self` bound to the base
    // (so `Self` uses the empty prefix)
    fn scope() -> Scope {
        Scope {
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
        }
    }

    #[test]
    fn prefix_for_type_maps_a_bare_or_referenced_tag_type_to_its_prefix() {
        let tags = tags();
        let prefix = |ty: Type| value(prefix_for_type(&ty, &tags));

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

        // `prefixed_arg` is a prefixed binding, `base_arg` a base binding
        assert_eq!(value(scope.prefix("prefixed_arg")), Some("svc"));
        assert_eq!(value(scope.prefix("base_arg")), Some(""));
        // a name that is not in scope yields nothing
        assert_eq!(value(scope.prefix("unknown")), None);
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
