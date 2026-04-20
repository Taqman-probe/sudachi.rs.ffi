mod analysis;
mod output;

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::PathBuf;
use std::str::FromStr;
//use serde::Serialize;
use sudachi::prelude::*;
use sudachi::config::Config;
use sudachi::dic::dictionary::JapaneseDictionary;
use analysis::{Analysis, AnalyzeNonSplitted, AnalyzeSplitted, SplitSentencesOnly};

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub enum SentenceSplitMode {
    /// Do both sentence splitting and analysis
    #[default]
    Default,
    /// Do only sentence splitting and not analysis
    Only,
    /// Do only analysis without sentence splitting
    None,
}

impl FromStr for SentenceSplitMode {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "yes" | "default" => Ok(SentenceSplitMode::Default),
            "no" | "none" => Ok(SentenceSplitMode::None),
            "only" => Ok(SentenceSplitMode::Only),
            _ => Err("invalid sentence split mode: allowed values - yes, default, no, none, only"),
        }
    }
}

struct Cli {
    print_all: bool,
    wakati: bool,
}

macro_rules! with_output {
    ($cli: expr, $exclude_pos: expr, $f: expr) => {
        if $cli.wakati {
            Box::new($f(output::Wakachi::new($exclude_pos)))
        } else {
            Box::new($f(output::Simple::new($cli.print_all, $exclude_pos)))
        }
    };
}

/// 解析結果を保持する構造体
#[repr(C)]
pub struct SudachiLib {
    path_buf: Option<PathBuf>,
    dict: JapaneseDictionary,
    mode: Mode,
    wakati: bool,
    print_all: bool,
    split_sentences: SentenceSplitMode,
    exclude_pos: Vec<String>, // ["記号", "助詞"] のような除外品詞リストのJSON
}

/// 辞書の初期化
#[unsafe(no_mangle)]
pub extern "C" fn init(
    config_path: *const c_char,
    mode_val: i8,
    is_wakati: i8,
    is_print_all: i8,
    split_sentences_val: i8,
    exclude_pos_json: *const c_char,
) -> *mut SudachiLib {
    let path_buf = if config_path.is_null() {
        None
    } else {
        let path_str = unsafe { CStr::from_ptr(config_path).to_str().unwrap() };
        Some(PathBuf::from(path_str))
    };

    let config = if config_path.is_null() {
        Config::new(None, None, None)
    } else {
        let path_str = unsafe { CStr::from_ptr(config_path).to_str().unwrap() };
        Config::new(Some(path_str.into()), None, None)
    }.expect("Failed to load config");

    let mode = match mode_val {
        0 => Mode::A,
        1 => Mode::B,
        _ => Mode::C,
    };

    let wakati = is_wakati != 0;
    let print_all = is_print_all != 0;

    let split_sentences = match split_sentences_val {
        1 => SentenceSplitMode::Only,
        2 => SentenceSplitMode::None,
        _ => SentenceSplitMode::Default
    };

    let exclude_pos = if exclude_pos_json.is_null() {
        vec![]
    } else {
        let exclude_pos_str = unsafe { CStr::from_ptr(exclude_pos_json).to_str().unwrap() };
        serde_json::from_str(&exclude_pos_str).expect("Faild to parse exclude_pos")
    };

    let dict = JapaneseDictionary::from_cfg(&config).expect("Failed to create dictionary");
    Box::into_raw(Box::new(SudachiLib { path_buf, dict, mode, wakati, print_all, split_sentences, exclude_pos }))
}

/// メインの解析関数
/// input_json: ["text1", "text2"] のようなJSON文字列
#[unsafe(no_mangle)]
pub extern "C" fn analyze(
    ptr: *mut SudachiLib,
    input_json: *const c_char,
    out_len: *mut usize
  ) -> *mut c_char {
    let lib = unsafe { &mut *ptr };
    let detail = Cli {wakati: lib.wakati, print_all: lib.print_all};
    let input_str = unsafe { CStr::from_ptr(input_json).to_str().unwrap() };
    if input_str.is_empty() {return std::ptr::null_mut()};
    let inputs: Vec<String> = serde_json::from_str(input_str).unwrap_or_default();

    let mut all_results = Vec::new();

    let mut analyzer: Box<dyn Analysis> = match lib.split_sentences {
        SentenceSplitMode::Only => Box::new(SplitSentencesOnly::new(&lib.dict)),
        SentenceSplitMode::Default => with_output!(detail, lib.exclude_pos.clone(), |o| {
            AnalyzeSplitted::new(o, &lib.dict, lib.mode, false)
        }),
        SentenceSplitMode::None => with_output!(detail, lib.exclude_pos.clone(), |o| {
            AnalyzeNonSplitted::new(o, &lib.dict, lib.mode, false)
        }),
    };

    for text in inputs {
        let lines: Vec<&str> = text
            .split(|c| c == '\n' || c == '\r')
            .filter(|s| !s.is_empty())
            .collect();
        let mut writer: Vec<String> = Vec::new();

        // tokenize and output results
        for no_eol in lines.iter() {
            analyzer.analyze(no_eol, &mut writer);
        }

        all_results.push(format!("[{}]", writer.join(",")));
    }

    let result_json = format!("[{}]", &all_results.join(","));
    let res_ptr = CString::new(result_json).unwrap().into_raw();

    // 文字列の長さを計算して、out_len が指す先に書き込む
    unsafe {
        if !out_len.is_null() {
            let len = CStr::from_ptr(res_ptr).to_bytes().len() + 1;
            *out_len = len ;
        }
    }
    res_ptr
}

/// メモリ解放用
#[unsafe(no_mangle)]
pub extern "C" fn free_string(s: *mut c_char) {
    unsafe {
        if s.is_null() { return; }
        let _ = CString::from_raw(s);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn free_sudachi(ptr: *mut SudachiLib) {
    unsafe {
        if ptr.is_null() { return; }
        let _ = Box::from_raw(ptr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    // ヘルパー関数
    fn run_analyze(lib_ptr: *mut SudachiLib, text: &str) -> String {
        let input_json = format!(r#"["{}"]"#, text);
        let c_input = CString::new(input_json).unwrap();
        let mut out_len: usize = 0;
        
        let res_ptr = analyze(lib_ptr, c_input.as_ptr(), &mut out_len);
        if res_ptr.is_null() {
            return String::from("null");
        }
        
        let result = unsafe { CStr::from_ptr(res_ptr).to_string_lossy().into_owned() };
        free_string(res_ptr);
        result
    }

    #[test]
    fn test_all_patterns() {
        let confing_path = CString::new("./resources/sudachi.json").unwrap();
        // split_sentences: Only (文分割のみ)
        let lib_only = init(confing_path.as_ptr(), 2, 0, 0, 1, ptr::null());
        let res1 = run_analyze(lib_only, "今日はいい天気。明日は雨？明後日は地震雷火事親父！");
        assert!(serde_json::from_str::<Vec<Vec<String>>>(&res1).unwrap() == [["今日はいい天気。明日は雨？", "明後日は地震雷火事親父！"]]);
        free_sudachi(lib_only);

        // wakati: true (わかち書き)
        let lib_wakati = init(confing_path.as_ptr(), 2, 1, 0, 0, ptr::null());
        let res2 = run_analyze(lib_wakati, "「この味がいいね」と君が言ったから七月六日はサラダ記念日（俵万智『サラダ記念日』より");
        assert!(serde_json::from_str::<Vec<Vec<String>>>(&res2).unwrap() == [["「", "この", "味", "が", "いい", "ね", "」", "と", "君", "が", "言っ", "た", "から", "七", "月", "六", "日", "は", "サラダ", "記念日", "（", "俵", "万智", "『", "サラダ", "記念日", "』", "より"]]);
        free_sudachi(lib_wakati);

        // wakati: false, print_all: false (標準構造)
        let lib_simple = init(confing_path.as_ptr(), 2, 0, 0, 0, ptr::null());
        let res3 = run_analyze(lib_simple, "記念日");
        assert!(res3.contains("記念日"));
        assert!(res3.contains("名詞"));
        assert!(res3.contains("surface")); 
        assert!(res3.contains("poses")); 
        assert!(res3.contains("normalized_form")); 
        assert!(!res3.contains("dictionary_form")); 
        assert!(!res3.contains("reading_form")); 
        free_sudachi(lib_simple);

        // wakati: false, print_all: true (詳細構造)
        let lib_detail = init(confing_path.as_ptr(), 2, 0, 1, 0, ptr::null());
        let res4 = run_analyze(lib_detail, "記念日");
        assert!(res4.contains("記念日"));
        assert!(res4.contains("名詞"));
        assert!(res4.contains("surface")); 
        assert!(res4.contains("poses")); 
        assert!(res4.contains("normalized_form")); 
        assert!(res4.contains("dictionary_form")); 
        assert!(res4.contains("reading_form")); 
        free_sudachi(lib_detail);

        // exclude_pos (品詞除外、"助詞" を除外する)
        let exclude_json = CString::new(r#"["助詞", "助動詞"]"#).unwrap();
        let lib_exclude = init(confing_path.as_ptr(), 2, 1, 0, 0, exclude_json.as_ptr());
        let res5 = run_analyze(lib_exclude, "君が言った");
        assert!(serde_json::from_str::<Vec<Vec<String>>>(&res5).unwrap() == [["君", "言っ"]]);
        free_sudachi(lib_exclude);
    }
}