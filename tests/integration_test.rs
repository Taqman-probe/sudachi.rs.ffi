use sudachi_ffi;
use std::ffi::{CStr, CString};
use std::ptr;

#[test]
fn test_full_lifecycle() {
    // 1. 初期化 (pub extern "C" fn init)
    let lib_ptr = sudachi_ffi::init(
        CString::new("./resources/sudachi.json").unwrap().as_ptr(),
        2,           // mode_val (Mode C)
        1,           // is_wakati (true)
        0,           // is_print_all (false)
        0,           // split_sentences_val (Default)
        ptr::null()  // exclude_pos_json
    );
    assert!(!lib_ptr.is_null());

    let input = CString::new(r#"["プロジェクトに関するお問い合わせ。"]"#).unwrap();
    let mut out_len: usize = 0;
    let res_ptr = sudachi_ffi::analyze(lib_ptr, input.as_ptr(), &mut out_len);
    
    assert!(!res_ptr.is_null());

    let result_cstr = unsafe { CStr::from_ptr(res_ptr) };
    let actual_bytes_with_nul = result_cstr.to_bytes_with_nul().len();
    assert_eq!(out_len, actual_bytes_with_nul, "out_len should include NUL terminator");

    let result_str = result_cstr.to_str().unwrap();
    assert!(serde_json::from_str::<Vec<Vec<String>>>(result_str).unwrap() == [["プロジェクト","に","関する","お","問い合わせ","。"]]);

    sudachi_ffi::free_string(res_ptr);
    sudachi_ffi::free_sudachi(lib_ptr);
}