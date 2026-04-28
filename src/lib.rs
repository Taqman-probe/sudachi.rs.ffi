mod analysis;
mod output;

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::PathBuf;
use std::str::FromStr;
use rayon::prelude::*;
use sudachi::prelude::*;
use sudachi::config::Config;
use sudachi::dic::dictionary::JapaneseDictionary;
use analysis::{Analysis, AnalyzeNonSplitted, AnalyzeSplitted, SplitSentencesOnly};

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub enum SentenceSplitMode {
    // Do both sentence splitting and analysis
    #[default]
    Default,
    // Do only sentence splitting and not analysis
    Only,
    // Do only analysis without sentence splitting
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

struct OutputHelper {
    is_json: bool,
}

impl OutputHelper {
    fn new(method: &str) -> Self {
        Self { is_json: method == "JSON" }
    }
    fn write_start(&self, w: &mut Vec<u8>) {
        if self.is_json { w.push(b'['); }
    }
    fn write_end(&self, w: &mut Vec<u8>) {
        if self.is_json {
            // 直前がカンマなら消してから閉じる（空振りの掃除）
            if w.last() == Some(&b',') { w.pop(); }
            w.push(b']');
        }
    }
    fn write_separator(&self, w: &mut Vec<u8>) {
        if self.is_json {
            w.push(b',');
        }
    }
    fn write_chunk_separator(&self, w: &mut Vec<u8>) {
        if self.is_json {
            w.push(b',');
        } else {
            w.push(b'\n');
        }
    }
}

macro_rules! with_output {
    ($cli: expr, $exclude_pos: expr, $method: expr, $f: expr) => {
        if $cli.wakati {
            match $method {
                "JSON" => Box::new($f(output::WakachiJSON::new($exclude_pos))),
                "Raw" => Box::new($f(output::WakachiRaw::new($exclude_pos))),
                _ => panic!("Invalid method"),
             }
        } else {
            match $method {
                "JSON" => Box::new($f(output::SimpleJSON::new($cli.print_all, $exclude_pos))),
                "Raw" => Box::new($f(output::SimpleRaw::new($cli.print_all, $exclude_pos))),
                _ => panic!("Invalid method"),
            }
        }
    };
}

// 解析結果を保持する構造体
#[repr(C)]
pub struct SudachiLib {
    path_buf: Option<PathBuf>,
    dict: JapaneseDictionary,
    mode: Mode,
    wakati: bool,
    print_all: bool,
    split_sentences: SentenceSplitMode,
    exclude_pos: Vec<String>, // ["記号", "助詞"] のような除外品詞リストのJSON
    multi_thread: bool, // マルチスレッド化
}

///辞書の初期化
#[unsafe(no_mangle)]
pub extern "C" fn init(
    config_path: *const c_char,
    mode_val: i8,
    is_wakati: i8,
    is_print_all: i8,
    split_sentences_val: i8,
    exclude_pos_json: *const c_char,
    is_multi: i8,
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

    let multi_thread = is_multi != 0;

    Box::into_raw(Box::new(SudachiLib { 
        path_buf,
        dict,
        mode,
        wakati,
        print_all,
        split_sentences,
        exclude_pos,
        multi_thread
    }))
}

fn estimate_len(
    input_len: usize,
    lib: &SudachiLib,
    method: &str,
) -> usize {
    match method {
        "JSON" => {
            if lib.wakati {
                input_len * 4
            } else if lib.print_all {
                input_len * 75
            } else {
                input_len * 60
            }
        },
        "Raw" => {
            if lib.wakati {
                input_len * 4
            } else if lib.print_all {
                input_len * 35
            } else {
                input_len * 30
            }
        },
        _ => input_len
    }
}


fn analyze_single (
    inputs: Vec<&str>,
    lib: &SudachiLib,
    method: &str
) -> Vec<u8> {
    let output_helper = OutputHelper::new(method);
    let detail = Cli { wakati: lib.wakati, print_all: lib.print_all };
    let mut analyzer: Box<dyn Analysis> = match lib.split_sentences {
        SentenceSplitMode::Only => Box::new(SplitSentencesOnly::new(&lib.dict)),
        SentenceSplitMode::Default => with_output!(detail, lib.exclude_pos.clone(), method, |o| {
            AnalyzeSplitted::new(o, &lib.dict, lib.mode, false)
        }),
        SentenceSplitMode::None => with_output!(detail, lib.exclude_pos.clone(), method, |o| {
            AnalyzeNonSplitted::new(o, &lib.dict, lib.mode, false)
        }),
    };

    let total_input_len: usize = inputs.iter().map(|s| s.len()).sum();
    let mut writer: Vec<u8> = Vec::with_capacity(estimate_len(total_input_len, &lib, method));
    output_helper.write_start(&mut writer);
    inputs.into_iter().for_each(| text | {
        let lines: Vec<_> = text
            .split(|c| c == '\n' || c == '\r')
            .filter(|s| !s.is_empty())
            .collect();

        // tokenize and output results
        output_helper.write_start(&mut writer);
        for no_eol in lines.into_iter() {
            analyzer.analyze(no_eol, &mut writer, method);
            output_helper.write_separator(&mut writer);
        }
        output_helper.write_end(&mut writer);
        output_helper.write_chunk_separator(&mut writer);
    });
    output_helper.write_end(&mut writer);
    writer
}

fn analyze_multi(
    inputs: Vec<&str>,
    lib: &SudachiLib,
    method: &str,
) -> Vec<u8> {
    let output_helper = OutputHelper::new(method);
    let total_input_len: usize = inputs.iter().map(|s| s.len()).sum();
    let results: Vec<Vec<u8>> = inputs.into_par_iter().map_init(
        || {
            // --- 初期化クロージャ (スレッドごとに1回実行) ---
            let detail = Cli { wakati: lib.wakati, print_all: lib.print_all };
            
            // 各スレッド専用の analyzer を作成
            let analyzer: Box<dyn Analysis> = match lib.split_sentences {
                SentenceSplitMode::Only => Box::new(SplitSentencesOnly::new(&lib.dict)),
                SentenceSplitMode::Default => with_output!(detail, lib.exclude_pos.clone(), method, |o| {
                    AnalyzeSplitted::new(o, &lib.dict, lib.mode, false)
                }),
                SentenceSplitMode::None => with_output!(detail, lib.exclude_pos.clone(), method, |o| {
                    AnalyzeNonSplitted::new(o, &lib.dict, lib.mode, false)
                }),
            };
            analyzer
        },
        |analyzer, text| {
            let lines: Vec<_> = text
                .split(|c| c == '\n' || c == '\r')
                .filter(|s| !s.is_empty())
                .collect();

            let mut local_writer = Vec::with_capacity(estimate_len(text.len(), &lib, method));
            // tokenize and output results
            output_helper.write_start(&mut local_writer);
            for no_eol in lines.into_iter() {
                analyzer.analyze(no_eol, &mut local_writer, method);
                output_helper.write_separator(&mut local_writer);
            }
            output_helper.write_end(&mut local_writer);
            output_helper.write_chunk_separator(&mut local_writer);
            local_writer
    }).collect();

    let mut writer = Vec::with_capacity(estimate_len(total_input_len, &lib, method));
    output_helper.write_start(&mut writer);
    for res in results.iter() {
        writer.extend_from_slice(res);
    }
    output_helper.write_end(&mut writer);
    writer
}

// メインの解析関数
// input_json: ["text1", "text2"] のようなJSON文字列
#[unsafe(no_mangle)]
pub extern "C" fn analyze(
    ptr: *mut SudachiLib,
    input_json: *const c_char,
    out_len: *mut usize
  ) -> *mut c_char {
    let lib = unsafe { &mut *ptr };
    let input_str = unsafe { CStr::from_ptr(input_json).to_bytes() };
    if input_str.is_empty() {return std::ptr::null_mut()};
    let inputs: Vec<String> = serde_json::from_slice(input_str).unwrap();
    let input_refs: Vec<&str> = inputs.iter().map(|v| v.as_str()).collect();

    let all_results = if lib.multi_thread {
        analyze_multi(input_refs, lib, "JSON")
    } else {
        analyze_single(input_refs, lib, "JSON")
    };

    let res_ptr = CString::new(all_results).unwrap().into_raw();

    // 文字列の長さを計算して、out_len が指す先に書き込む
    unsafe {
        if !out_len.is_null() {
            let len = CStr::from_ptr(res_ptr).to_bytes().len() + 1;
            *out_len = len ;
        }
    }
    res_ptr
}

// バイナリ形式での解析関数
// input_data: u32(length1) + text1_bytes + u32(length2) + text2_bytes + ... の形式
// 各テキストの前に4バイトのu32（リトルエンディアン）で長さを指定
#[unsafe(no_mangle)]
pub extern "C" fn analyze_raw(
    ptr: *mut SudachiLib,
    input_data: *const u8,
    input_len: usize,
    out_len: *mut usize
  ) -> *mut c_char {
    let lib = unsafe { &mut *ptr };
    
    let input_bytes = unsafe {
        std::slice::from_raw_parts(input_data, input_len)
    };
    
    // バイナリデータをパース
    let mut inputs_owned: Vec<&str> = Vec::with_capacity(8132);
    let mut pos = 0;
    
    while pos < input_bytes.len() {
        if pos + 4 > input_bytes.len() {
            break; // 残りが4バイト未満の場合は終了
        }
        
        // 4バイトをu32として読む（リトルエンディアン）
        let len_bytes = [input_bytes[pos], input_bytes[pos+1], input_bytes[pos+2], input_bytes[pos+3]];
        let text_len = u32::from_le_bytes(len_bytes) as usize;
        pos += 4;
        
        if pos + text_len > input_bytes.len() {
            break; // テキストがバッファを超える場合は終了
        }
        
        // UTF-8文字列に変換
        if let Ok(text_str) = std::str::from_utf8(&input_bytes[pos..pos + text_len]) {
            inputs_owned.push(text_str);
        }
        pos += text_len;
    }
    
    if inputs_owned.is_empty() {
        return std::ptr::null_mut();
    }
    
    let mut all_results = if lib.multi_thread {
        analyze_multi(inputs_owned, lib, "Raw")
    } else {
        analyze_single(inputs_owned, lib, "Raw")
    };
    // callbackモードで複数回バッチ方式にする場合、anaryze_single/analyze_multi内でEOSを付けるとEOSがバッチ処理ごとに入ってしまう
    // 処理を共通化するとここでEOSを付与する形になる
    all_results.extend_from_slice(b"EOS\n");

    let res_ptr = CString::new(all_results).unwrap().into_raw();
    
    unsafe {
        if !out_len.is_null() {
            let len = CStr::from_ptr(res_ptr).to_bytes().len() + 1;
            *out_len = len;
        }
    }
    res_ptr
}

// コールバック関数の型定義
type SudachiCallback = extern "C" fn(buffer: *const u8, len: usize, user_data: *mut std::ffi::c_void);

// コールバックでの解析関数
// 入力はanaryze_rauと同じ形式
#[unsafe(no_mangle)]
pub extern "C" fn analyze_callback(
    ptr: *mut SudachiLib,
    input_data: *const u8,
    input_len: usize,
    callback: SudachiCallback,
    user_data: *mut std::ffi::c_void,
  ) -> i32 {
    let lib = unsafe { &mut *ptr };
    
    let input_bytes = unsafe {
        std::slice::from_raw_parts(input_data, input_len)
    };
    
    // バイナリデータをパース
    let mut inputs_owned: Vec<&str> = Vec::with_capacity(8192);
    let mut pos = 0;
    
    while pos < input_bytes.len() {
        if pos + 4 > input_bytes.len() {
            break; // 残りが4バイト未満の場合は終了
        }
        
        // 4バイトをu32として読む（リトルエンディアン）
        let len_bytes = [input_bytes[pos], input_bytes[pos+1], input_bytes[pos+2], input_bytes[pos+3]];
        let text_len = u32::from_le_bytes(len_bytes) as usize;
        pos += 4;
        
        if pos + text_len > input_bytes.len() {
            break; // テキストがバッファを超える場合は終了
        }
        
        // UTF-8文字列に変換
        if let Ok(text_str) = std::str::from_utf8(&input_bytes[pos..pos + text_len]) {
            inputs_owned.push(text_str);
        }
        pos += text_len;
    }
    
    if inputs_owned.is_empty() { return 1; }

    let num_inputs = inputs_owned.len();
    if num_inputs == 0 { return 1; }

    let total_chars: usize = inputs_owned.iter().map(|s| s.chars().count()).sum();
    let avg_chars = total_chars / num_inputs;

    // 出力モードに応じた1件あたりの予想倍率
    // exclude_posで指定した品詞の出現率がわかればさらに精度があがる
    let multiplier = if lib.wakati { 
        1.3
    } else if lib.print_all { 
        14.0
    } else { 
        10.0
    };

    // 1回のコールバックで約 8MB 程度を目指す場合の件数
    // (目標バイト数) / (平均文字数 * 3(UTF8) * 倍率)
    let target_batch_bytes = 8 * 1024 * 1024;
    let estimated_per_input = (avg_chars as f64 * 3 as f64 * multiplier) as usize;
    let dynamic_chunk_size = (target_batch_bytes / estimated_per_input).max(1).min(8192);

    for chunk in inputs_owned.chunks(dynamic_chunk_size) {
        let chunk_vec = chunk.to_vec();
        
        let results = if lib.multi_thread {
            analyze_multi(chunk_vec, lib, "Raw")
        } else {
            analyze_single(chunk_vec, lib, "Raw")
        };

        // バッチごとにコールバックを実行
        if !results.is_empty() {
            callback(results.as_ptr(), results.len(), user_data);
        }
    }

    // 終了通知 (EOS) 
    let eos = b"EOS\n";
    callback(eos.as_ptr(), eos.len(), user_data);

    0
}

// メモリ解放用
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
mod tests;
