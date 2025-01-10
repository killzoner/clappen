#[allow(dead_code)]
#[cfg(test)]
mod tests {
    #[clappen::clappen(export = nested, gen_into)]
    mod nested {
        #[derive(PartialEq, Debug)]
        pub struct MyStruct {}
    }

    #[clappen::clappen(export = prefixed_struct_generator, gen_into)]
    mod m1 {
        #[derive(PartialEq, Debug)]
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
        prefixed_struct_generator!("second");
        let a = ServerOptions {
            url: "url a".into(),
            say_hello: Some(false),
            nested_default: MyStruct {},
            nested: __inner_TestMyStruct::TestMyStruct {},
            nested1: __inner_Test1MyStruct::Test1MyStruct {},
            nested2: __inner_Test2MyStruct::Test2MyStruct {},
        };

        assert_eq!(a.a_function(), "url: url a, say_hello: Some(false)");
        a.another_function();
        a.a_third_function_in_second_impl_block();

        let b = SecondServerOptions {
            second_url: "url b".into(),
            second_say_hello: Some(true),
            second_nested_default: MyStruct {},
            second_nested: __inner_SecondTestMyStruct::SecondTestMyStruct {},
            second_nested1: __inner_SecondTest1MyStruct::SecondTest1MyStruct {},
            second_nested2: __inner_SecondTest2MyStruct::SecondTest2MyStruct {},
        };

        assert_eq!(b.a_function(), "url: url b, say_hello: Some(true)");
        b.another_function();
        b.a_third_function_in_second_impl_block();

        let c = SecondServerOptions {
            second_url: "url a".into(),
            second_say_hello: Some(false),
            second_nested_default: MyStruct {},
            second_nested: __inner_SecondTestMyStruct::SecondTestMyStruct {},
            second_nested1: __inner_SecondTest1MyStruct::SecondTest1MyStruct {},
            second_nested2: __inner_SecondTest2MyStruct::SecondTest2MyStruct {},
        };
        let c_into: ServerOptions = c.into();
        assert_eq!(c_into, a);
    }
}


