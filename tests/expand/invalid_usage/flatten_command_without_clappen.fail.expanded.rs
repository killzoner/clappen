fn main() {
    /// Macros used for nested struct definition : []
    pub struct NestedMyStruct {}
    #[allow(
        dead_code,
        unreachable_code,
        unused_variables,
        unused_braces,
        unused_qualifications,
    )]
    #[allow(
        clippy::style,
        clippy::complexity,
        clippy::pedantic,
        clippy::restriction,
        clippy::perf,
        clippy::deprecated,
        clippy::nursery,
        clippy::cargo,
        clippy::suspicious_else_formatting,
        clippy::almost_swapped,
        clippy::redundant_locals,
    )]
    #[automatically_derived]
    impl clap::FromArgMatches for NestedMyStruct {
        fn from_arg_matches(
            __clap_arg_matches: &clap::ArgMatches,
        ) -> ::std::result::Result<Self, clap::Error> {
            Self::from_arg_matches_mut(&mut __clap_arg_matches.clone())
        }
        fn from_arg_matches_mut(
            __clap_arg_matches: &mut clap::ArgMatches,
        ) -> ::std::result::Result<Self, clap::Error> {
            #![allow(deprecated)]
            let v = NestedMyStruct {};
            ::std::result::Result::Ok(v)
        }
        fn update_from_arg_matches(
            &mut self,
            __clap_arg_matches: &clap::ArgMatches,
        ) -> ::std::result::Result<(), clap::Error> {
            self.update_from_arg_matches_mut(&mut __clap_arg_matches.clone())
        }
        fn update_from_arg_matches_mut(
            &mut self,
            __clap_arg_matches: &mut clap::ArgMatches,
        ) -> ::std::result::Result<(), clap::Error> {
            #![allow(deprecated)]
            ::std::result::Result::Ok(())
        }
    }
    #[allow(
        dead_code,
        unreachable_code,
        unused_variables,
        unused_braces,
        unused_qualifications,
    )]
    #[allow(
        clippy::style,
        clippy::complexity,
        clippy::pedantic,
        clippy::restriction,
        clippy::perf,
        clippy::deprecated,
        clippy::nursery,
        clippy::cargo,
        clippy::suspicious_else_formatting,
        clippy::almost_swapped,
        clippy::redundant_locals,
    )]
    #[automatically_derived]
    impl clap::Args for NestedMyStruct {
        fn group_id() -> Option<clap::Id> {
            Some(clap::Id::from("NestedMyStruct"))
        }
        fn augment_args<'b>(__clap_app: clap::Command) -> clap::Command {
            {
                let __clap_app = __clap_app
                    .group(
                        clap::ArgGroup::new("NestedMyStruct")
                            .multiple(true)
                            .args({
                                let members: [clap::Id; 0usize] = [];
                                members
                            }),
                    );
                __clap_app
            }
        }
        fn augment_args_for_update<'b>(__clap_app: clap::Command) -> clap::Command {
            {
                let __clap_app = __clap_app
                    .group(
                        clap::ArgGroup::new("NestedMyStruct")
                            .multiple(true)
                            .args({
                                let members: [clap::Id; 0usize] = [];
                                members
                            }),
                    );
                __clap_app
            }
        }
    }
    (/*ERROR*/)
}
