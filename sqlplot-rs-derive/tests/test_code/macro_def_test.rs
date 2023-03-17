use sqlplot_rs_derive::ResultLine;

#[derive(ResultLine)]
struct MyStruct {
    a: String,
    b: u32,
    c: f64,
}

fn main() {}