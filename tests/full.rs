// cargo expand --test full
#[allow(dead_code)]
#[cfg(test)]
mod tests {
    #[clappen::clappen(export = nested)]
    mod nested {
        pub struct MyStruct {}
    }

    nested!(); // define default nested struct

    #[clappen::clappen(export = prefixed_struct_generator)]
    mod m1 {
        pub struct ServerOptions {
            /// Address to connect to.
            ///
            url: String,

            /// Do you need to say hello?.
            ///
            say_hello: Option<bool>,

            /// A nested struct without a prefix.
            ///
            nested_default: MyStruct,

            /// A nested struct that needs a prefix.
            ///
            #[clappen_command(apply = nested, prefix = "test")]
            nested: MyStruct,

            /// A nested struct that needs another prefix.
            ///
            #[clappen_command(apply = nested, prefix = "test1")]
            nested1: MyStruct,

            /// A nested struct that needs yet another prefix.
            #[clappen_command(apply = nested, prefix = "test2")]
            nested2: MyStruct,
        }

        impl ServerOptions {
            fn a_function(&self) -> String {
                format!("url: {}, say_hello: {:?}", self.url, self.say_hello)
            }
            fn another_function(&self) {}
        }

        impl ServerOptions {
            fn a_third_function_in_second_impl_block(&self) {}
        }
    }

    #[test]
    fn it_works() {
        prefixed_struct_generator!();
        prefixed_struct_generator!("second");

        let a = ServerOptions {
            url: "url a".into(),
            say_hello: Some(false),
            nested_default: MyStruct {},
            nested: __inner_nested::TestMyStruct {},
            nested1: __inner_nested1::Test1MyStruct {},
            nested2: __inner_nested2::Test2MyStruct {},
        };

        assert_eq!(a.a_function(), "url: url a, say_hello: Some(false)");
        a.another_function();
        a.a_third_function_in_second_impl_block();

        let b = SecondServerOptions {
            second_url: "url b".into(),
            second_say_hello: Some(true),
            second_nested_default: MyStruct {},
            second_nested: __inner_second_nested::SecondTestMyStruct {},
            second_nested1: __inner_second_nested1::SecondTest1MyStruct {},
            second_nested2: __inner_second_nested2::SecondTest2MyStruct {},
        };

        assert_eq!(b.a_function(), "url: url b, say_hello: Some(true)");
        b.another_function();
        b.a_third_function_in_second_impl_block();
    }
}
