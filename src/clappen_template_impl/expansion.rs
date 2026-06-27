// Codegen for `#[clappen_template_impl]`: split the impls tagged with it from the regular ones and
// build the per-prefix pieces inserted into the exported macro (see `clappen::create_template`).

use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{Ident, ItemImpl, ItemStruct, Path, Token};

use super::TEMPLATE_MARKER;
use crate::clappen_command::attrs::{Attributes as CommandAttributes, NestedAttributes};
use crate::clappen_struct::FIELD_ATTR_CLAPPEN_COMMAND;

// the per-prefix pieces `build` returns; `clappen::create_template` inserts them into the exported macro
pub(crate) struct Expansion {
    // impls without the `#[clappen_template_impl]` marker, handled like any clappen impl
    pub(crate) regular_impls: Vec<ItemImpl>,
    // `($prefix)` arm: this struct's own template impl, specialized for the prefix
    pub(crate) prefixed_self_apply: Vec<TokenStream>,
    // `($prefix)` arm: recurse into each flattened child by calling its macro
    pub(crate) prefixed_child_apply: Vec<TokenStream>,
    // `@__template` arm (this struct flattened in a parent): `self_apply` at the inherited chain
    pub(crate) chained_self_apply: Vec<TokenStream>,
    // `@__template` arm: `child_apply`, extending the inherited chain
    pub(crate) chained_child_apply: Vec<TokenStream>,
    // `()` arm: `child_apply` with an empty prefix, only when this struct has no template of its own
    pub(crate) base_child_apply: Vec<TokenStream>,
}

pub(crate) fn build(
    items_impl: &[&ItemImpl],
    struct_def: &ItemStruct,
    fields: &[TokenStream],
    default_prefix: &str,
) -> Expansion {
    // split regular vs template impls; the marker may override the `Base`/`Prefixed` tag idents
    // (left as `None` here and defaulted by the proc-macro)
    let mut regular_impls = Vec::new();
    let mut template_impls: Vec<(ItemImpl, Option<Ident>, Option<Ident>)> = Vec::new();
    for item in items_impl {
        let mut item = (*item).clone();
        let marker = item
            .attrs
            .iter()
            .find(|a| a.path().is_ident(TEMPLATE_MARKER))
            .cloned();
        item.attrs.retain(|a| !a.path().is_ident(TEMPLATE_MARKER));
        let Some(marker) = marker else {
            regular_impls.push(item);
            continue;
        };

        let mut base_tag = None;
        let mut prefixed_tag = None;
        let _ = marker.parse_nested_meta(|meta| {
            if meta.path.is_ident("base") {
                base_tag = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("prefixed") {
                prefixed_tag = Some(meta.value()?.parse()?);
            }
            Ok(())
        });
        template_impls.push((item, base_tag, prefixed_tag));
    }

    let struct_ident = struct_def.ident.clone();
    let nested_fields = collect_nested_fields(struct_def);

    // `self_apply` is this struct's own template impl, tagged for the proc-macro to rewrite later
    // (the same idea as `#[__clappen_impl]`). `child_apply` recurses into each flattened child by
    // calling its macro. Both are built once here, parameterized by the prefix and the chain, then
    // reused across the arms: the arms can't call one another, because a `#[macro_export]` macro
    // built by a proc-macro can't name itself across crates.
    let apply_pieces = |prefix: TokenStream, own_chain: TokenStream, recurse_chain: TokenStream| {
        let self_apply: Vec<TokenStream> = template_impls
            .iter()
            .map(|(e, base_tag, prefixed_tag)| {
                let tags = tag_attrs(base_tag, prefixed_tag);
                quote! {
                    #[clappen::__clappen_template_impl(prefix = #prefix, self_default = #default_prefix, item = #struct_ident, #tags chain = [ #own_chain ], prefixed_fields = [#(#fields)*])]
                    #e
                }
            })
            .collect();
        let child_apply: Vec<TokenStream> = nested_fields
            .iter()
            .map(|(apply, command_prefix, field_ident)| {
                quote! {
                    #apply!(@__template #prefix, chain = [ #recurse_chain (#command_prefix, #field_ident, #default_prefix) ]);
                }
            })
            .collect();
        (self_apply, child_apply)
    };

    // `@__template` arm: reached when this struct is flattened in a parent; the chain is the metavars
    // the arm matched
    let (chained_self_apply, chained_child_apply) = apply_pieces(
        quote! { $prefix },
        quote! { $( ($command_prefix, $field, $parent_default) ),* },
        quote! { $( ($command_prefix, $field, $parent_default), )* },
    );

    // `($prefix)` arm: top-level prefixed instantiation, so the chain starts empty
    let (prefixed_self_apply, prefixed_child_apply) =
        apply_pieces(quote! { $prefix }, quote! {}, quote! {});

    // `()` arm: a template-free parent still converts each flattened child to its base (empty
    // prefix). A parent with its own template gets the children's conversions from its prefixed
    // instantiations instead, so this is empty then.
    let base_child_apply = if template_impls.is_empty() {
        apply_pieces(quote! { "" }, quote! {}, quote! {}).1
    } else {
        Vec::new()
    };

    Expansion {
        regular_impls,
        prefixed_self_apply,
        prefixed_child_apply,
        chained_self_apply,
        chained_child_apply,
        base_child_apply,
    }
}

// flattened fields that drive the sibling templated impls: (apply macro, command prefix, field name)
fn collect_nested_fields(struct_def: &ItemStruct) -> Vec<(Path, String, Ident)> {
    struct_def
        .fields
        .iter()
        .filter_map(|field| {
            let field_ident = field.ident.clone()?;
            let attr = field
                .attrs
                .iter()
                .find(|a| a.path().is_ident(FIELD_ATTR_CLAPPEN_COMMAND))?;
            let metas = attr
                .parse_args_with(Punctuated::<NestedAttributes, Token![,]>::parse_terminated)
                .ok()?;
            let cmd: CommandAttributes = metas.into_iter().collect::<Vec<_>>().try_into().ok()?;
            Some((cmd.apply, cmd.prefix, field_ident))
        })
        .collect()
}

// optional `base_tag = .., prefixed_tag = ..,` overrides forwarded to the template proc-macro
fn tag_attrs(base: &Option<Ident>, prefixed: &Option<Ident>) -> TokenStream {
    let base = base.as_ref().map(|t| quote! { base_tag = #t, });
    let prefixed = prefixed.as_ref().map(|t| quote! { prefixed_tag = #t, });
    quote! { #base #prefixed }
}
