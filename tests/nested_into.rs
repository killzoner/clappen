#[allow(dead_code)]
#[cfg(test)]
mod tests {
    #[clappen::clappen(export = nested, gen_into)]
    mod nested {
        pub struct MyStruct {}
    }

    #[clappen::clappen(export = prefixed_struct_generator, gen_into)]
    mod m1 {
        pub struct ServerOptions {
            /// A nested struct without a prefix.
            ///
            nested_default: MyStruct,

            /// A nested struct that needs a prefix.
            ///
            #[clappen_command(apply = nested, prefix = "test")]
            nested: MyStruct,
        }

    }

    prefixed_struct_generator!("second");
}
