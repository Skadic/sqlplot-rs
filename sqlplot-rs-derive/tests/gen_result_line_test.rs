use sqlplot_rs_core::ResultLine;
use sqlplot_rs_derive::ResultLine;

#[derive(ResultLine)]
struct MyStruct {
    a: &'static str,
    b: u32,
    c: f64,
}

#[test]
fn gen_result_line_test() {
    let s = MyStruct {
        a: "xyz",
        b: 1,
        c: -5.5
    };

    assert_eq!(s.to_result_line().as_str(), "RESULT a=\"xyz\" b=1 c=-5.5");
}