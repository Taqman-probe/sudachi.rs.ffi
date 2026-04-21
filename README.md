# Sudachi FFI (Rust)

日本語形態素解析器 **Sudachi** (**[Sudachi.rs](https://github.com/WorksApplications/sudachi.rs)**) を FFI 経由で任意の言語から利用可能にしたライブラリです。
文章分割・形態素解析・分かち書きなどオリジナルのsudachi.rsで指定できるオプションは一通り実装しています。
さらに品詞フィルタリングを指定できるよう拡張しています。

---

## ✨ 特徴

* Sudachi による高精度な日本語形態素解析
* 文分割 (Sentence Split) の ON / OFF / 分割のみ の切り替え
* 分かち書き (wakati) 対応
* 詳細情報 (print_all) の ON / OFF の切り替え
* 品詞フィルタリング (除外リスト指定) 指定可能
* Rust 実装による高速処理
* FFI 経由で 各言語から利用可能 (TypeScript / Deno サンプルあり)
* バッチ処理前提 (JSONで["文字列1", 文字列2]の文字列配列を渡す設計)

---

## 🚀 セットアップ

### 1. Rust 側ビルド

```bash
cargo build --release
```

DLL / dylib / so が生成されます。

例:

```
target/release/sudachi_ffi.dll
```

---

### 2. Sudachi 辞書の準備

Sudachi の設定ファイル (`sudachi.json`) と辞書を用意してください。

---

### 3. 使用例 (TypeScript & Deno から利用)

```ts
let configPath = new URL("../resources/sudachi_default.json", import.meta.url).pathname;
if (Deno.build.os === "windows") {
  configPath = configPath.slice(1);
} 
const sudachi = new Sudachi ({
  configPath: configPath,
  mode: 2,           // A:0 / B:1 / C:2
  wakati: false,      // 分かち書き
  printAll: true,   // 詳細情報出力
  splitSentences: 0, // default:0 / only:1 / none:2
  excludePos: ["記号", "助詞"], // 品詞除外設定
  multi: false // マルチスレッド
});

try {
  const result = sudachi.analyze(["今日はいい天気です。"]);
  console.log(JSON.parse(result || "{}"));
} finally {
  sudachi.close();
  sudachi.dylibInstance.close();
}
```

```sh
mkdir -p sudachi-demo/resources && mkdir -p sudachi-demo/deno && cd sudachi-demo
# Download exsample
curl -L https://raw.githubusercontent.com/Taqman-probe/sudachi.rs.ffi/main/deno/mod.ts -o deno/mod.ts
curl -L https://raw.githubusercontent.com/Taqman-probe/sudachi.rs.ffi/main/deno/exsample.ts -o deno/exsample.ts
curl -L https://raw.githubusercontent.com/Taqman-probe/sudachi.rs.ffi/main/resources/sudachi_default.json -o resources/sudachi_default.json
curl -L https://github.com/Taqman-probe/sudachi.rs.ffi/releases/download/v0.6.11-1/libsudachi_ffi.so -o resources/libsudachi_ffi.so
# Download Dictionary
curl -LO http://sudachi.s3-website-ap-northeast-1.amazonaws.com/sudachidict/sudachi-dictionary-latest-core.zip
unzip sudachi-dictionary-latest-core.zip
mv sudachi-dictionary-*/system_core.dic resources/
rm -rf sudachi-dictionary-latest-core.zip sudachi-dictionary-*/
# Execute
export SUDACHI_FFI=./resources/libsudachi_ffi.so
deno run --allow-ffi --allow-read --allow-env deno/exsample.ts
```

```powershell
New-Item -ItemType Directory -Force -Path "sudachi-demo\deno"
New-Item -ItemType Directory -Force -Path "sudachi-demo\resources"
Set-Location "sudachi-demo"
# Download exsample
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/Taqman-probe/sudachi.rs.ffi/main/deno/mod.ts" -OutFile "mod.ts"
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/Taqman-probe/sudachi.rs.ffi/main/deno/exsample.ts" -OutFile "exsample.ts"
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/Taqman-probe/sudachi.rs.ffi/main/resources/sudachi_default.json" -OutFile "resources/config_default.json"
Invoke-WebRequest -Uri "https://github.com/Taqman-probe/sudachi.rs.ffi/releases/download/v0.6.11-1/sudachi_ffi.dll"-OutFile "resources/sudachi_ffi.dll"
# Download Dictionary
Invoke-WebRequest -Uri "http://sudachi.s3-website-ap-northeast-1.amazonaws.com/sudachidict/sudachi-dictionary-latest-core.zip" -OutFile "sudachi-dictionary-latest-core.zip"
Expand-Archive -Path "sudachi-dictionary-latest-core.zip" -DestinationPath "."
Move-Item -Path "sudachi-dictionary-*/system_core.dic" -Destination "resources"
Remove-Item -Path "sudachi-dictionary-latest-core.zip", "sudachi-dictionary-*" -Recurse
# Execute
$env:SUDACHI_FFI = "./resources/sudachi_ffi.dll"
deno run --allow-ffi --allow-read --allow-env exsample.ts
```

#### 出力

```json
[
  [
    {
      "surface": "今日",
      "poses": ["名詞", "普通名詞", "副詞可能", "*", "*", "*"],
      "normalized_form": "今日",
      "dictionary_form": "今日",
      "reading_form": "今日"
    },
    {
      "surface": "いい",
      "poses": ["形容詞", "非自立可能", "*", "*", "形容詞", "連体形-一般"],
      "normalized_form": "良い",
      "dictionary_form": "いい",
      "reading_form": "いい"
    },
    {
      "surface": "天気",
      "poses": ["名詞", "普通名詞", "一般", "*", "*", "*"],
      "normalized_form": "天気",
      "dictionary_form": "天気",
      "reading_form": "天気"
    },
    {
      "surface": "です",
      "poses": ["助動詞", "*", "*", "*", "助動詞-デス", "終止形-一般"],
      "normalized_form": "です",
      "dictionary_form": "です",
      "reading_form": "です"
    },
    {
      "surface": "。",
      "poses": ["補助記号", "句点", "*", "*", "*", "*"],
      "normalized_form": "。",
      "dictionary_form": "。",
      "reading_form": "。"
    }
  ]
]
```

---

## ⚙️ オプション

### `mode`

Sudachi の解析モード

| 値 | モード | 説明 |
| - | - | -------- |
| 0 | A | 短い単位 |
| 1 | B | 中間 |
| 2 | C | 長い単位 |

---

### `splitSentences`

| 値       | 説明       |
| ------- | -------- |
| default | 文分割 + 解析 |
| only    | 文分割のみ    |
| none    | 分割せず解析   |

---

### `wakati`

* `true`: 表層形のみ
* `false`: print_all 参照

---

### `print_all`

* `true`: 表層形, 品詞 (配列), 正規化表記, 辞書形, 読み (, 未知語の場合"is_oov": true)
* `false`: 表層形, 品詞 (配列), 正規化表記 

---

### `excludePos`

特定の品詞を除外

例:

```json
["記号", "助詞"]
```

---

## ⚠️ 注意

* Sudachi 辞書、プラグインDLLが必要です
* Windows / Linux / Mac でビルド成果物が異なります

### Sudachi 辞書の準備 (必須)

以下をダウンロードしてください：

[WorksApplications/SudachiDict](https://github.com/WorksApplications/SudachiDict)

- 辞書ファイル（small / core / full）
- `char.def`（character definition）

### プラグインについて

デフォルト設定では以下のプラグインを使用するようです：

- default_input_text
- simple_oov
- join_numeric
- join_katakana_oov

#### 推奨: 内蔵スタイル

config_default.jsonを指定すると本体にエイリアス (Java版と同名のクラス指定) として組み込まれたプラグインを使用するため、外部ファイルは不要です。

#### 上級: 外部指定スタイル

config.jsonを指定すると、別途ダイナミックリンクライブラリのファイルを指定することになります。

これらのバイナリ（.dll / .so / .dylib）は配布されていないため、sudachi.rsのリポジトリからビルドする必要があります。

```sh
git clone https://github.com/WorksApplications/sudachi.rs
cd sudachi.rs
cargo build --release -p default_input_text
cargo build --release -p simple_oov
cargo build --release -p join_numeric
cargo build --release -p join_katakana_oov
```
ビルド後、生成された動的ライブラリを利用してください。

---

## 👀 ひとこと
Sudachi には Java・Python・CLI などの実装がありますが、
任意の言語へライブラリとして組み込み、高速にインプロセスで扱うという用途では、やや制約があると感じました。
本プロジェクトは、Rust 実装である sudachi.rs の性能と柔軟性を活かし、
FFI を通じてあらゆる言語から直接呼び出せる形態素解析コアとして構築しています。
日本語自然言語処理を、より自由に・より高速に扱うための基盤として、
さまざまな環境で活用してもらえたら嬉しいです。

This project is based on [sudachi.rs](https://github.com/WorksApplications/sudachi.rs) and includes modifications.