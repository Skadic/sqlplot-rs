use sqlplot_rs_derive::ResultLine;

#[derive(ResultLine)]
struct MyStruct {
    #[result(name="a_töken")]
    a: &'static str,
    b: u32,
    c: f64,
}

fn main() {}