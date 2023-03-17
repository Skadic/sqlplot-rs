use sqlplot_rs_derive::ResultLine;

#[derive(ResultLine)]
struct MyStruct {
    #[result(name="6atoken")]
    a: &'static str,
    b: u32,
    c: f64,
}
fn main() {}