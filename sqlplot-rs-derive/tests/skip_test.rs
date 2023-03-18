use sqlplot_rs_core::ResultLine;
use sqlplot_rs_derive::ResultLine;

#[allow(unused)]
#[derive(ResultLine)]
struct MyStruct {
    a: &'static str,
    #[skip]
    b: u32,
    c: f64,
}

#[test]
fn rename_test() {
    let s = MyStruct {
        a: "xyz",
        b: 1,
        c: -5.5
    };

    assert_eq!(s.to_result_line().as_str(), "RESULT a=\"xyz\" c=-5.5");
}