
fn main(){
    #[clappen::clappen(export = prefix)]
    mod m1 {
        pub struct A {}
        pub struct B {}
    }
}
