use sqlplot_rs_core::ResultLine;
use sqlplot_rs_derive::ResultLine;

#[derive(ResultLine)]
struct MyStruct {
    a: &'static str,
    b: u32,
    c: f64,
}

fn main() {
    let s = MyStruct {
        a: "a",
        b: 1,
        c: -5.5
    };

    let _: String = s.to_result_line();
}