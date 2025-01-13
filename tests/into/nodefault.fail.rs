#[clappen::clappen(export = generate, gen_into)]
mod generate{
    struct SimpleStruct{}
}
// This macro doesn't work with no argument when gen_into is provided
fn main() {
    generate!();
}
