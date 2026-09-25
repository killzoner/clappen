use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use syn::Result;
use syn::parse::{Parse, ParseStream};

use crate::helper::{
    DEFAULT_PREFIX_ATTR, PREFIX_ATTR, PrefixValue, camel_case, non_empty, parse_non_empty_literal,
    prefixed_ident, snake_case, snake_join,
};

// holds the raw value of a `default_prefix = ".."` attribute
#[derive(Default)]
pub(crate) struct DefaultPrefix {
    value: Option<String>,
}

impl Parse for DefaultPrefix {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            value: Some(parse_non_empty_literal(input, DEFAULT_PREFIX_ATTR)?),
        })
    }
}

impl PrefixValue for DefaultPrefix {
    fn value(&self) -> &Option<String> {
        &self.value
    }
}

// holds the raw value of a `prefix = ".."` attribute on a struct or an impl
#[derive(Default)]
pub(crate) struct StructPrefix {
    value: Option<String>,
}

impl Parse for StructPrefix {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            value: Some(parse_non_empty_literal(input, PREFIX_ATTR)?),
        })
    }
}

impl PrefixValue for StructPrefix {
    fn value(&self) -> &Option<String> {
        &self.value
    }
}

// holds the raw value of a `prefix = ".."` inside a field's `#[clappen_command(..)]`
#[derive(Default)]
pub(crate) struct CommandPrefix {
    value: Option<String>,
}

impl Parse for CommandPrefix {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            value: Some(parse_non_empty_literal(input, PREFIX_ATTR)?),
        })
    }
}

// snake case prefix for a struct's own ident and its field names
pub(crate) struct FieldPrefix {
    value: Option<String>,
}

impl FieldPrefix {
    pub(crate) fn new(default_prefix: &DefaultPrefix, struct_prefix: &StructPrefix) -> Self {
        Self {
            value: non_empty(snake_case(&snake_join(&[
                &default_prefix.value,
                &struct_prefix.value,
            ]))),
        }
    }

    // struct field name with its prefix: `<prefix>_<name>`
    pub(crate) fn field_name(&self, name: &str) -> String {
        let Some(prefix) = &self.value else {
            return name.to_string();
        };

        format!("{prefix}_{name}")
    }

    // type ident for a struct defined under this prefix
    pub(crate) fn type_ident(&self, base: &str) -> Ident {
        prefixed_ident(&self.value, base)
    }
}

impl PrefixValue for FieldPrefix {
    fn value(&self) -> &Option<String> {
        &self.value
    }
}

// camel case prefix passed one level down to a nested field's generated macro
pub(crate) struct NestedPrefix {
    value: Option<String>,
}

impl NestedPrefix {
    pub(crate) fn new(
        command_prefix: &CommandPrefix,
        default_prefix: &DefaultPrefix,
        struct_prefix: &StructPrefix,
    ) -> Self {
        Self {
            value: non_empty(camel_case(&snake_join(&[
                &command_prefix.value,
                &default_prefix.value,
                &struct_prefix.value,
            ]))),
        }
    }

    // type ident for a parent's reference to a nested child struct
    pub(crate) fn type_ident(&self, base: &str) -> Ident {
        prefixed_ident(&self.value, base)
    }
}

impl ToTokens for NestedPrefix {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.value.to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parent_and_child_name_the_same_type() {
        let cases = [
            (None, None, "Remote"),
            (Some("d"), None, "Remote"),
            (Some("d"), Some("test"), "Remote"),
            (Some("my_d"), Some("test"), "Remote"),
            (Some("log"), Some("test"), "HTTPServer"),
        ];

        for (child_default, command_prefix, struct_ident) in cases {
            let child_default = DefaultPrefix {
                value: child_default.map(str::to_string),
            };
            let command_prefix = CommandPrefix {
                value: command_prefix.map(str::to_string),
            };

            // what `child!()` names the struct: `DRemote`
            let base_name =
                FieldPrefix::new(&child_default, &StructPrefix::default()).type_ident(struct_ident);
            let nested = NestedPrefix::new(
                &command_prefix,
                &DefaultPrefix::default(),
                &StructPrefix::default(),
            );

            let parent_reference = nested.type_ident(&base_name.to_string());
            // the nested prefix crosses the macro boundary as a literal and comes back
            // as the child's own `prefix = ".."`
            let child_prefix = StructPrefix {
                value: nested.value.clone(),
            };
            let child_definition =
                FieldPrefix::new(&child_default, &child_prefix).type_ident(struct_ident);

            assert_eq!(
                parent_reference, child_definition,
                "default_prefix {:?}, prefix {:?}, {struct_ident}",
                child_default.value, command_prefix.value
            );
        }
    }
}
