use sqlplot_rs_core::ResultLine;
use sqlplot_rs_derive::ResultLine;

#[derive(ResultLine)]
struct MyStruct {
    #[result(name="a_token")]
    a: &'static str,
    b: u32,
    #[result(name="my_c_token")]
    c: f64,
}

#[test]
fn rename_test() {
    let s = MyStruct {
        a: "xyz",
        b: 1,
        c: -5.5
    };

    assert_eq!(s.to_result_line().as_str(), "RESULT a_token=xyz b=1 my_c_token=-5.5");
}