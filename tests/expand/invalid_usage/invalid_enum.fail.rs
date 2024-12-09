fn main() {
    #[clappen::clappen(export = test)]
    mod m2 {
        pub enum Foo {}
    }
}
