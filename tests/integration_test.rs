use sudachi_ffi;
use std::ffi::{CStr, CString};
use std::ptr;

#[test]
fn test_full_lifecycle() {
    for is_multi in [0, 1] {
        // 1. 初期化 (pub extern "C" fn init)
        let lib_ptr = sudachi_ffi::init(
            CString::new("./resources/sudachi.json").unwrap().as_ptr(),
            2,           // mode_val (Mode C)
            1,           // is_wakati (true)
            0,           // is_print_all (false)
            0,           // split_sentences_val (Default)
            ptr::null(),  // exclude_pos_json
            is_multi, // is_multi (false)
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
}

#[test]
fn test_raw_full_lifecycle() {
    for is_multi in [0, 1] {
        // 1. 初期化 (pub extern "C" fn init)
        let lib_ptr = sudachi_ffi::init(
            CString::new("./resources/sudachi.json").unwrap().as_ptr(),
            2,           // mode_val (Mode C)
            1,           // is_wakati (true)
            0,           // is_print_all (false)
            0,           // split_sentences_val (Default)
            ptr::null(),  // exclude_pos_json
            is_multi, // is_multi (false)
        );
        assert!(!lib_ptr.is_null());

        let text_vec = vec!["プロジェクトに関するお問い合わせ。"];
        let mut input_data = Vec::new();
        for text in text_vec {
            let bytes = text.as_bytes();
            let len = bytes.len() as u32;
            input_data.extend_from_slice(&len.to_le_bytes()); // 長さをリトルエンディアンで追加
            input_data.extend_from_slice(bytes);             // テキスト本体
        }

        let mut out_len: usize = 0;
        let res_ptr = sudachi_ffi::analyze_raw(lib_ptr, input_data.as_ptr(), input_data.len(), &mut out_len);
        
        assert!(!res_ptr.is_null());

        let result_cstr = unsafe { CStr::from_ptr(res_ptr) };
        let actual_bytes_with_nul = result_cstr.to_bytes_with_nul().len();
        assert_eq!(out_len, actual_bytes_with_nul, "out_len should include NUL terminator");

        let result_str = result_cstr.to_str().unwrap();
        assert!(result_str == "プロジェクト に 関する お 問い合わせ 。\nEOS\n");

        sudachi_ffi::free_string(res_ptr);
        sudachi_ffi::free_sudachi(lib_ptr);
    }
}

// コールバックでデータを受け取るための共有バッファ
struct TestCallbackState {
    output: Vec<u8>,
}

// extern "C" なのでクロージャは使えず、静的な関数か関数ポインタが必要
extern "C" fn integration_callback(buffer: *const u8, len: usize, user_data: *mut std::ffi::c_void) {
    let state = unsafe { &mut *(user_data as *mut TestCallbackState) };
    let slice = unsafe { std::slice::from_raw_parts(buffer, len) };
    state.output.extend_from_slice(slice);
}

#[test]
fn test_callback_full_lifecycle() {
    for is_multi in [0, 1] {
        let lib_ptr = sudachi_ffi::init(
            CString::new("./resources/sudachi.json").unwrap().as_ptr(),
            2,
            1,
            0,
            0,
            ptr::null(),
            is_multi,
        );

        let mut state = TestCallbackState { output: Vec::new() };
        
        // 入力データの作成 (analyze_rawと同じ形式)
        let mut input_data = Vec::new();
        let text = "プロジェクトに関するお問い合わせ。";
        let bytes = text.as_bytes();
        input_data.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        input_data.extend_from_slice(bytes);

        // 実行
        let res = sudachi_ffi::analyze_callback(
            lib_ptr,
            input_data.as_ptr(),
            input_data.len(),
            integration_callback,
            &mut state as *mut _ as *mut std::ffi::c_void,
        );

        assert_eq!(res, 0);
        let result_str = String::from_utf8(state.output).unwrap();
        
        // callback形式は内部で Raw モードとして扱われるため、空白区切り + EOS
        assert!(result_str == "プロジェクト に 関する お 問い合わせ 。\nEOS\n");
        
        sudachi_ffi::free_sudachi(lib_ptr);
    }
}