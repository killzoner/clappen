use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::visit_mut::{self, VisitMut};
use syn::{
    Expr, ExprField, ExprStruct, FnArg, Ident, ImplItem, ItemImpl, Member, Pat, Path, Type,
    parse_quote,
};

use super::{DEFAULT_BASE_TAG, DEFAULT_PREFIXED_TAG, attrs};
use crate::helper;

#[derive(Clone, Copy)]
enum Role {
    Base,
    Prefixed,
}

// one resolved tag (Base or Prefixed): the tag ident, its type path, and its field prefix
struct ResolvedTag {
    tag: Ident,
    path: Path,
    field_prefix: String,
}

struct TemplateSubst {
    base: ResolvedTag,
    prefixed: ResolvedTag,
    fields: Vec<String>,
    base_bindings: Vec<String>,
    prefixed_bindings: Vec<String>,
    self_role: Option<Role>,
}

impl VisitMut for TemplateSubst {
    fn visit_path_mut(&mut self, path: &mut Path) {
        if path.is_ident(&self.base.tag) {
            *path = self.base.path.clone();
            return;
        }
        if path.is_ident(&self.prefixed.tag) {
            *path = self.prefixed.path.clone();
            return;
        }
        visit_mut::visit_path_mut(self, path);
    }

    fn visit_expr_field_mut(&mut self, node: &mut ExprField) {
        visit_mut::visit_expr_field_mut(self, node);

        let (name, span) = match &node.member {
            Member::Named(member) => (member.to_string(), member.span()),
            _ => return,
        };
        if self.fields.iter().any(|f| f == &name)
            && let Some(prefix) = self.receiver_field_prefix(&node.base)
            && !prefix.is_empty()
        {
            node.member = Member::Named(Ident::new(&format!("{prefix}_{name}"), span));
        }
    }

    fn visit_expr_struct_mut(&mut self, node: &mut ExprStruct) {
        if let Some(prefix) = self.literal_field_prefix(&node.path)
            && !prefix.is_empty()
        {
            for field in &mut node.fields {
                let (name, span) = match &field.member {
                    Member::Named(member) => (member.to_string(), member.span()),
                    _ => continue,
                };
                if self.fields.iter().any(|f| f == &name) {
                    field.member = Member::Named(Ident::new(&format!("{prefix}_{name}"), span));
                }
            }
        }
        visit_mut::visit_expr_struct_mut(self, node);
    }
}

impl TemplateSubst {
    // prefix to apply to a field read, based on which instantiation the receiver holds
    fn receiver_field_prefix(&self, base: &Expr) -> Option<&str> {
        let Expr::Path(path) = base else { return None };
        let ident = path.path.get_ident()?.to_string();
        if self.prefixed_bindings.iter().any(|b| b == &ident) {
            Some(self.prefixed.field_prefix.as_str())
        } else if self.base_bindings.iter().any(|b| b == &ident) {
            Some(self.base.field_prefix.as_str())
        } else {
            None
        }
    }

    // prefix to apply to a struct literal's fields, based on the constructed instantiation
    fn literal_field_prefix(&self, path: &Path) -> Option<&str> {
        if path.is_ident(&self.base.tag) {
            return Some(self.base.field_prefix.as_str());
        }
        if path.is_ident(&self.prefixed.tag) {
            return Some(self.prefixed.field_prefix.as_str());
        }
        if path.is_ident("Self") {
            return match self.self_role {
                Some(Role::Base) => Some(self.base.field_prefix.as_str()),
                Some(Role::Prefixed) => Some(self.prefixed.field_prefix.as_str()),
                None => None,
            };
        }
        None
    }
}

fn tag_role(ty: &Type, base_tag: &Ident, prefixed_tag: &Ident) -> Option<Role> {
    if let Type::Path(path) = ty {
        if path.path.is_ident(base_tag) {
            return Some(Role::Base);
        }
        if path.path.is_ident(prefixed_tag) {
            return Some(Role::Prefixed);
        }
    }
    None
}

// pure substitution: rewrite the base/prefixed tags to their paths and prefix the
// relevant field accesses / struct-literal fields
fn substitute(
    mut item: ItemImpl,
    base: ResolvedTag,
    prefixed: ResolvedTag,
    fields: Vec<String>,
) -> TokenStream {
    let self_role = tag_role(&item.self_ty, &base.tag, &prefixed.tag);
    let mut base_bindings = Vec::new();
    let mut prefixed_bindings = Vec::new();
    match self_role {
        Some(Role::Base) => base_bindings.push("self".to_string()),
        Some(Role::Prefixed) => prefixed_bindings.push("self".to_string()),
        None => {}
    }
    for impl_item in &item.items {
        if let ImplItem::Fn(method) = impl_item {
            for arg in &method.sig.inputs {
                if let FnArg::Typed(pat_type) = arg
                    && let Pat::Ident(pat_ident) = &*pat_type.pat
                {
                    match tag_role(&pat_type.ty, &base.tag, &prefixed.tag) {
                        Some(Role::Base) => base_bindings.push(pat_ident.ident.to_string()),
                        Some(Role::Prefixed) => prefixed_bindings.push(pat_ident.ident.to_string()),
                        None => {}
                    }
                }
            }
        }
    }

    let mut subst = TemplateSubst {
        base,
        prefixed,
        fields,
        base_bindings,
        prefixed_bindings,
        self_role,
    };
    subst.visit_item_impl_mut(&mut item);

    quote! { #item }
}

// Entry point: validate the forwarded attributes and build the specialized impl.
pub(crate) fn expand(item: ItemImpl, attrs: attrs::Attributes) -> syn::Result<TokenStream> {
    let Some(struct_ident) = attrs.item else {
        return Err(syn::Error::new(
            Span::call_site(),
            "clappen template impl requires `item`",
        ));
    };

    let base_tag = attrs
        .base_tag
        .unwrap_or_else(|| Ident::new(DEFAULT_BASE_TAG, Span::call_site()));
    let prefixed_tag = attrs
        .prefixed_tag
        .unwrap_or_else(|| Ident::new(DEFAULT_PREFIXED_TAG, Span::call_site()));

    // the prefixed instantiation carries the literal prefix, the base the empty one
    let prefixed = resolve_tag(
        &attrs.prefix,
        &attrs.chain,
        &attrs.self_default,
        &struct_ident,
        prefixed_tag,
    );
    // an empty prefix is a flattened field resolving against its own base (a template-free
    // parent's `()` arm calls `@__template` with an empty prefix): the base ignores the chain.
    // A non-empty prefix is a prefixed instantiation whose base is the parent's nested base,
    // so it uses the chain like the prefixed side.
    let base_chain: &[attrs::ChainStep] = if attrs.prefix.is_empty() {
        &[]
    } else {
        &attrs.chain
    };
    let base = resolve_tag("", base_chain, &attrs.self_default, &struct_ident, base_tag);

    // debug doc: which instantiation this impl belongs to (prefix + nesting path)
    let nesting: Vec<&str> = attrs
        .chain
        .iter()
        .map(|s| s.command_prefix.as_str())
        .filter(|c| !c.is_empty())
        .collect();
    let doc = if nesting.is_empty() {
        format!(
            " Template impl for `{struct_ident}` (prefix '{}')",
            attrs.prefix
        )
    } else {
        format!(
            " Template impl for `{struct_ident}` (prefix '{}', nested via {})",
            attrs.prefix,
            nesting.join(".")
        )
    };

    let expanded = substitute(item, base, prefixed, attrs.prefixed_fields);
    Ok(quote! {
        #[doc = #doc]
        #expanded
    })
}

// resolve one tag (Base or Prefixed) to its struct: walk the nesting chain to build the module
// path + field prefix, the same way the struct generation builds them at each level
fn resolve_tag(
    top_prefix: &str,
    chain: &[attrs::ChainStep],
    self_default: &str,
    struct_ident: &Ident,
    tag: Ident,
) -> ResolvedTag {
    let mut struct_prefix = top_prefix.to_string();
    let mut modules: Vec<Ident> = Vec::new();

    for step in chain {
        // the field name once the parent struct has prefixed it
        let parent_field_prefix = helper::field_prefix(&step.parent_default, &struct_prefix);
        let field_ident = if parent_field_prefix.is_empty() {
            step.field.to_string()
        } else {
            format!("{parent_field_prefix}_{}", step.field)
        };
        modules.push(helper::macro_module_name(&field_ident));
        struct_prefix =
            helper::nested_step_prefix(&step.command_prefix, &step.parent_default, &struct_prefix);
    }

    // this struct's own default_prefix applies on top of the carried prefix
    let field_prefix = helper::field_prefix(self_default, &struct_prefix);
    let type_ident = helper::prefixed_ident(&field_prefix, &struct_ident.to_string());

    let path: Path = if modules.is_empty() {
        type_ident.into()
    } else {
        parse_quote!(#(#modules)::* :: #type_ident)
    };

    ResolvedTag {
        tag,
        path,
        field_prefix,
    }
}
