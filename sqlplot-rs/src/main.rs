use sqlplot_rs_core::ResultLine;
use sqlplot_rs_derive::ResultLine;

#[allow(unused)]
#[derive(ResultLine)]
struct MyStruct {
    #[result(name = "a_token")]
    a: usize,
    #[skip]
    b: &'static str,
    c: f64,
}

fn main() {
    let a: MyStruct = MyStruct {
        a: 0,
        b: "()",
        c: 0.5,
    };

    println!("Hello, world! {}", a.to_result_line());
}
