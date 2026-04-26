use super::*;
mod tests {
    use super::*;
    use std::ptr;

    // ヘルパー関数
    fn run_analyze(lib_ptr: *mut SudachiLib, input_json: &str) -> String {
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

    fn run_analyze_raw(lib_ptr: *mut SudachiLib, text_vec: Vec<&str>) -> String {
        let mut input_data = Vec::new();
        for text in text_vec {
            let bytes = text.as_bytes();
            let len = bytes.len() as u32;
            input_data.extend_from_slice(&len.to_le_bytes()); // 長さをリトルエンディアンで追加
            input_data.extend_from_slice(bytes);             // テキスト本体
        }
        let mut out_len: usize = 0;

        let res_ptr = analyze_raw(lib_ptr, input_data.as_ptr(), input_data.len(), &mut out_len);
        if res_ptr.is_null() {
            return String::from("null");
        }
        
        let result = unsafe { CStr::from_ptr(res_ptr).to_string_lossy().into_owned() };
        free_string(res_ptr);
        result
    }

    #[test]
    fn test_all_patterns() {
        let config_path = CString::new("./resources/sudachi.json").unwrap();
        // split_sentences: Only (文分割のみ)
        let lib_only = init(config_path.as_ptr(), 2, 0, 0, 1, ptr::null(), 0);
        let res1 = run_analyze(lib_only, r#"["今日はいい天気。明日は雨？明後日は地震雷火事親父！","え？明々後日・・・"]"#);
        assert!(serde_json::from_str::<Vec<Vec<String>>>(&res1).unwrap() == [["今日はいい天気。明日は雨？", "明後日は地震雷火事親父！"],["え？", "明々後日・・・"]]);
        free_sudachi(lib_only);

        // wakati: true (わかち書き)
        let lib_wakati = init(config_path.as_ptr(), 2, 1, 0, 0, ptr::null(), 0);
        let res2 = run_analyze(lib_wakati, r#"["「この味がいいね」と君が言ったから七月六日はサラダ記念日（俵万智『サラダ記念日』より"]"#);
        assert!(serde_json::from_str::<Vec<Vec<String>>>(&res2).unwrap() == [["「", "この", "味", "が", "いい", "ね", "」", "と", "君", "が", "言っ", "た", "から", "七", "月", "六", "日", "は", "サラダ", "記念日", "（", "俵", "万智", "『", "サラダ", "記念日", "』", "より"]]);
        free_sudachi(lib_wakati);

        // wakati: false, print_all: false (標準構造)
        let lib_simple = init(config_path.as_ptr(), 2, 0, 0, 0, ptr::null(), 0);
        let res3 = run_analyze(lib_simple, r#"["記念日"]"#);
        assert!(res3.contains("記念日"));
        assert!(res3.contains("名詞"));
        assert!(res3.contains("surface")); 
        assert!(res3.contains("poses")); 
        assert!(res3.contains("normalized_form")); 
        assert!(!res3.contains("dictionary_form")); 
        assert!(!res3.contains("reading_form")); 
        free_sudachi(lib_simple);

        // wakati: false, print_all: true (詳細構造)
        let lib_detail = init(config_path.as_ptr(), 2, 0, 1, 0, ptr::null(), 0);
        let res4 = run_analyze(lib_detail, r#"["記念日"]"#);
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
        let lib_exclude = init(config_path.as_ptr(), 2, 1, 0, 0, exclude_json.as_ptr(), 0);
        let res5 = run_analyze(lib_exclude, r#"["君が言った"]"#);
        assert!(serde_json::from_str::<Vec<Vec<String>>>(&res5).unwrap() == [["君", "言っ"]]);
        free_sudachi(lib_exclude);
    }

    #[test]
    fn test_multi_thread() {
        let confing_path = CString::new("./resources/sudachi.json").unwrap();
        // is_multi を 1 に設定
        let lib_multi = init(confing_path.as_ptr(), 2, 1, 0, 0, ptr::null(), 1);
        
        // 複数の入力を準備
        let input_json = r#"["リンゴを食べます", "明日は晴れです", "東京に行きます"]"#;
        let result = run_analyze(lib_multi, input_json);
        
        // JSONとしてパースできるか、期待した要素数（この場合3つ）があるか確認
        let parsed: Vec<Vec<String>> = serde_json::from_str(&result).expect("Failed to parse multi-thread JSON");
        assert_eq!(parsed.len(), 3);
        assert!(parsed[0][0] == "リンゴ");
        assert!(parsed[1][0] == "明日");
        assert!(parsed[2][0] == "東京");
        free_sudachi(lib_multi);
    }

    #[test]
    fn test_raw_all_patterns() {
        let config_path = CString::new("./resources/sudachi.json").unwrap();
        // split_sentences: Only (文分割のみ)
        let lib_only = init(config_path.as_ptr(), 2, 0, 0, 1, ptr::null(), 0);
        let res1 = run_analyze_raw(lib_only, vec!["今日はいい天気。明日は雨？明後日は地震雷火事親父！", "え？明々後日・・・"]);
        assert!(&res1 == "今日はいい天気。明日は雨？\n明後日は地震雷火事親父！\n\nえ？\n明々後日・・・\n\nEOS\n");
        free_sudachi(lib_only);

        // wakati: true (わかち書き)
        let lib_wakati = init(config_path.as_ptr(), 2, 1, 0, 0, ptr::null(), 0);
        let res2 = run_analyze_raw(lib_wakati, vec!["「この味がいいね」と君が言ったから七月六日はサラダ記念日（俵万智『サラダ記念日』より"]);
        assert!(&res2 == "「 この 味 が いい ね 」 と 君 が 言っ た から 七 月 六 日 は サラダ 記念日 （ 俵 万智 『 サラダ 記念日 』 より\nEOS\n");
        free_sudachi(lib_wakati);

        // wakati: false, print_all: false (標準構造)
        let lib_simple = init(config_path.as_ptr(), 2, 0, 0, 0, ptr::null(), 0);
        let res3 = run_analyze_raw(lib_simple, vec!["記念日"]);
        assert!(res3 == "記念日\t名詞,普通名詞,一般,*,*,*\t記念日\n\nEOS\n");
        free_sudachi(lib_simple);

        // wakati: false, print_all: true (詳細構造)
        let lib_detail = init(config_path.as_ptr(), 2, 0, 1, 0, ptr::null(), 0);
        let res4 = run_analyze_raw(lib_detail, vec!["記念日"]);
        assert!(res4 == "記念日\t名詞,普通名詞,一般,*,*,*\t記念日\t記念日\tキネンビ\t0\t[]\n\nEOS\n");
        free_sudachi(lib_detail);

        // exclude_pos (品詞除外、"助詞" を除外する)
        let exclude_json = CString::new(r#"["助詞", "助動詞"]"#).unwrap();
        let lib_exclude = init(config_path.as_ptr(), 2, 1, 0, 0, exclude_json.as_ptr(), 0);
        let res5 = run_analyze_raw(lib_exclude, vec!["君が言った"]);
        assert!(&res5 == "君 言っ\nEOS\n");
        free_sudachi(lib_exclude);
    }

    #[test]
    fn test_raw_multi_thread() {
        let confing_path = CString::new("./resources/sudachi.json").unwrap();
        // is_multi を 1 に設定
        let lib_multi = init(confing_path.as_ptr(), 2, 1, 0, 0, ptr::null(), 1);
        
        // 複数の入力を準備
        let input_json = vec!["リンゴを食べます", "明日は晴れです", "東京に行きます"];
        let result = run_analyze_raw(lib_multi, input_json);
        assert!(result == "リンゴ を 食べ ます\n明日 は 晴れ です\n東京 に 行き ます\nEOS\n");
        free_sudachi(lib_multi);
    }
}