pub use sqlplot_rs_core::ResultLine;
pub use sqlplot_rs_derive::ResultLine;

mod parsing;

#[allow(unused)]
#[derive(ResultLine)]
struct MyStruct {
    #[result(name = "a_token")]
    a: usize,
    b: &'static str,
    #[skip]
    c: f64,
    d: char,
}

fn main() {
    let a: MyStruct = MyStruct {
        a: 0,
        b: "()",
        c: 0.5,
        d: 'd'
    };
    println!("Hello, world! {}", a.to_result_line());
}
