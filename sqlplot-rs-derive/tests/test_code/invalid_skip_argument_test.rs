use sqlplot_rs_derive::ResultLine;

#[allow(unused)]
#[derive(ResultLine)]
struct MyStruct {
    a: &'static str,
    #[skip(random_arg)]
    b: u32,
    c: f64,
}

fn main() {}