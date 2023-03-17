
use sqlplot_rs_derive::ResultLine;

#[derive(ResultLine)]
struct MyStruct {
    a: &'static str,
    #[skip]
    b: u32,
    c: f64,
}

fn main() {}