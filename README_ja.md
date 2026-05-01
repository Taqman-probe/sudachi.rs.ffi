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
* バッチ処理前提 (["文字列1", 文字列2]の文字列配列を渡す設計)
* Callback対応: 巨大なデータを一定サイズ（チャンク）ごとに逐次処理し、メモリ消費を劇的に抑えた解析が可能。
* **3つの解析メソッド**: 汎用的な `analyze` (JSON) 、メモリ効率を最適化した `analyze_raw` (Binary)、 さらに巨大データ向けのストリーミング解析 analyze_callback を提供

---

## 🚀 セットアップ

### 1. Rust 側ビルド (非必須)

```bash
cargo build --release
```

DLL / dylib / so が生成されます。

例:

```
target/release/sudachi_ffi.dll
```

**Releasesにあるダイナミックリンクライブラリをダウンロードして使用することができます。**

---

### 2. Sudachi 設定ファイルと辞書の準備 (必須)

* Sudachi の設定ファイル (`sudachi.json`, `char.def`) と辞書を用意してください。

  当リポジトリにデフォルト設定の `char.def`、 `sudachi_default.json` とプラグイン (後述) を指定した `sudachi.json` を用意しています。

* 辞書は以下からダウンロードしてください：

  [SudachiDict](https://d2ej7fkh96fzlu.cloudfront.net/sudachidict/)
  または
  [WorksApplications/SudachiDict](https://github.com/WorksApplications/SudachiDict)

  - 辞書ファイルいずれか (small / core / full)


---

### 3. 使用例

#### TypeScript & Deno から利用

* TypeScript
```ts
import { Sudachi } from "./mod.ts";

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
  const result = sudachi.analyze(["今日はいい天気です。"]); // JSON形式
  console.log(JSON.parse(result || "{}"));
} finally {
  sudachi.close();
  sudachi.dylibInstance.close();
}
```

#### コピペで試す (Denoは別途インストールが必要)

* Bash (Macの場合は.soを.dylibに変更してください。)
```sh
mkdir -p sudachi-demo/resources && mkdir -p sudachi-demo/deno && cd sudachi-demo
# Download exsample
curl -L https://raw.githubusercontent.com/Taqman-probe/sudachi.rs.ffi/main/deno/mod.ts -o deno/mod.ts
curl -L https://raw.githubusercontent.com/Taqman-probe/sudachi.rs.ffi/main/deno/exsample.ts -o deno/exsample.ts
curl -L https://raw.githubusercontent.com/Taqman-probe/sudachi.rs.ffi/main/resources/sudachi_default.json -o resources/sudachi_default.json
curl -L https://github.com/Taqman-probe/sudachi.rs.ffi/releases/download/v0.6.11-1/libsudachi_ffi.so -o resources/libsudachi_ffi.so
# Download char.def
curl -L "https://raw.githubusercontent.com/Taqman-probe/sudachi.rs.ffi/main/resources/char.def" -o "resources/char.def"
# Download Dictionary
curl -LO https://d2ej7fkh96fzlu.cloudfront.net/sudachidict/sudachi-dictionary-latest-core.zip
unzip sudachi-dictionary-latest-core.zip
mv sudachi-dictionary-*/system_core.dic resources/
rm -rf sudachi-dictionary-latest-core.zip sudachi-dictionary-*/
# Execute
export SUDACHI_FFI=./resources/libsudachi_ffi.so
deno run --allow-ffi --allow-read --allow-env deno/exsample.ts
```

* PowerShell
```powershell
New-Item -ItemType Directory -Force -Path "sudachi-demo\deno"
New-Item -ItemType Directory -Force -Path "sudachi-demo\resources"
Set-Location "sudachi-demo"
# Download exsample
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/Taqman-probe/sudachi.rs.ffi/main/deno/mod.ts" -OutFile "deno/mod.ts"
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/Taqman-probe/sudachi.rs.ffi/main/deno/exsample.ts" -OutFile "deno/exsample.ts"
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/Taqman-probe/sudachi.rs.ffi/main/resources/sudachi_default.json" -OutFile "resources/sudachi_default.json"
Invoke-WebRequest -Uri "https://github.com/Taqman-probe/sudachi.rs.ffi/releases/download/v0.6.11-1/sudachi_ffi.dll"-OutFile "resources/sudachi_ffi.dll"
# Download char.def
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/Taqman-probe/sudachi.rs.ffi/main/resources/char.def" -Outfile "resources/char.def"
# Download Dictionary
Invoke-WebRequest -Uri "https://d2ej7fkh96fzlu.cloudfront.net/sudachidict/sudachi-dictionary-latest-core.zip" -OutFile "sudachi-dictionary-latest-core.zip"
Expand-Archive -Path "sudachi-dictionary-latest-core.zip" -DestinationPath "."
Move-Item -Path "sudachi-dictionary-*/system_core.dic" -Destination "resources"
Remove-Item -Path "sudachi-dictionary-latest-core.zip", "sudachi-dictionary-*" -Recurse
# Execute
$env:SUDACHI_FFI = "./resources/sudachi_ffi.dll"
deno run --allow-ffi --allow-read --allow-env deno/exsample.ts
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

### 解析メソッドの使い分け

本ライブラリは、用途に応じて3つのインターフェースを提供しています。

| メソッド     | 入力形式 | 特徴 |
| -- | ------ | -------- |
| analyze     | JSON    | 扱いやすく汎用的。出力もJSONで扱いやすくなっています。ただし内部で入力テキストのJSONの変換処理でメモリ再確保が発生し、出力もJSONのため情報密度が薄く入力データ量の割に出力データ量が膨らみます。 |
| analyze_raw | Binary  | パースは最速。ホスト言語側の長さ情報を付与したバイナリを直接参照するため、メモリを効率よく使用します。ただし結果を一度に巨大な文字列として受け取るため、極端に大きなデータではメモリを圧迫します。 |
| analyze_callback | Binary | 巨大テキスト用。 結果を Callback で返すため、メモリ消費を一定 (推定出力 8MB 単位) に抑えられ、ホスト言語側で入力テキストが保持できていれば処理可能になります。 |

#### analyze_raw の使用例と結果 (Deno)
```ts
const rawResult = sudachi.analyzeRaw(["今日は来る？", "明日は行く。"]);
console.log(rawResult);
// wakati: true 時
// 今日 は 来る ？
// 明日 は 行く 。
// EOS

// wakati: false, print_all: true 時 (入力文字列に対応する区切りは改行2回)
// 今日    名詞,普通名詞,副詞可能,*,*,*    今日    今日    キョウ  0       [981]
// は      助詞,係助詞,*,*,*,*     は      は      ハ      0       []
// 来る    動詞,非自立可能,*,*,カ行変格,終止形-一般        来る    来る    クル    0       []
// ？      補助記号,句点,*,*,*,*   ?       ?       ?       0       []
//
// 明日    名詞,普通名詞,副詞可能,*,*,*    明日    明日    アス    0       [13183]
// は      助詞,係助詞,*,*,*,*     は      は      ハ      0       []
// 行く    動詞,非自立可能,*,*,五段-カ行,終止形-一般       行く    行く    イク    0       []
// 。      補助記号,句点,*,*,*,*   。      。      。      0       []
//
//EOS
```
---

## 💡 巨大テキスト解析のポイント

形態素解析の結果は入力テキストに対して非常に大きなサイズになります。タブ、カンマ、改行区切りにしてバイト数換算でおおよそ・・・

分かち書きのみ: 入力の約 1.3 倍

通常解析 (品詞込): 入力の約 9 〜 10 倍

詳細情報追加 (読み・辞書形込): 入力の約 13 〜 14 倍

品詞情報を出力する場合、出力データが 9 倍以上に膨れ上がるため、数百MB超えのテキストを解析する際は analyze_callback を使用し、出力側のメモリ消費を一定に保つことが安定動作の鍵となります。

---

## ⚙️ オプション

### `mode`

Sudachi の解析モード

| 値 | モード | 説明 |
| -- | ------ | -------- |
| 0 | A | 短い単位 |
| 1 | B | 中間 |
| 2 | C | 長い単位 |

---

### `splitSentences`

| 値 | モード | 説明       |
| -- | ------ | ------------- |
| 0 | default | 文分割 + 解析 |
| 1 | only    | 文分割のみ    |
| 2 | none    | 分割せず解析   |

---

### `wakati`

* `true`: 表層形のみ
* `false`: `print_all` 参照

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

### `multi`

* `true`: マルチスレッドモード、スレッド数は **環境変数 RAYON_NUM_THREADS** により指定可能です（例: export RAYON_NUM_THREADS=4）。指定しない場合は全論理プロセッサを使用します
* `false`: シングルスレッドモード

---

## ⚠️ 注意

* Sudachi 辞書が必要です
* Windows / Linux / Mac でビルド成果物が異なります

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

これらのバイナリ (.dll / .so / .dylib) は配布されていないため、sudachi.rsのリポジトリからビルドする必要があります。

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

## 🚀 性能検証

[Livedoor コーパス](https://www.rondhuit.com/download.html) (約 25MB / 9カテゴリ) を使用したベンチマーク結果です。
SudachiPy (Python) と比較して、本ライブラリ (Deno FFI via Rust) は最速のマルチスレッドRawスタイルで **約4.5倍高速** に動作します。

| benchmark             | time/iter (avg) |        iter/s |      (min … max)      |      p75 |      p99 |     p995 |
| --------------------- | --------------- | ------------- | --------------------- | -------- | -------- | -------- |
| Multi thread Raw      |           2.3 s |           0.4 | (   2.0 s …    2.5 s) |    2.5 s |    2.5 s |    2.5 s |
| Multi thread Callback |           2.6 s |           0.4 | (   2.5 s …    2.7 s) |    2.7 s |    2.7 s |    2.7 s |
| Multi thread JSON     |           3.6 s |           0.3 | (   3.3 s …    4.1 s) |    3.8 s |    4.1 s |    4.1 s |
| Single thread Raw     |           8.1 s |           0.1 | (   7.9 s …    8.7 s) |    8.2 s |    8.7 s |    8.7 s |
| Single thread Callback |           8.9 s |           0.1 | (   7.7 s …   13.1 s) |    9.0 s |   13.1 s |   13.1 s |
| Single thread JSON    |           9.6 s |           0.1 | (   8.9 s …   11.3 s) |   10.3 s |   11.3 s |   11.3 s |
| SudachiPy (Python) |              9.2 s |           0.1 | (   8.1 s …   10.6 s) |    9.3 s |   10.6 s |   10.6 s |

* **テスト環境:** [CPU | 13th Gen Intel(R) Core(TM) i7-1355U (10 Cores / 12 Threads) | Runtime | Deno 2.7.9 (x86_64-pc-windows-msvc)]
* **条件:** SplitMode: C, Wakati: ON
* **注記1:** Deno側は実用面ではオブジェクトの変換が必須なため、変換コストを含んだ時間となっています。Rust内部の純粋な解析速度はさらに高速です。
* **注記2:** SudachiPyには一度に処理できる入力サイズに制限 (約48KB) があるため、Denoスクリプトでは文字列配列を一括で処理しているのに対し、Pythonスクリプトでは記事単位でループ処理を行っています。

### マルチスレッドとデータ形式について
今回のベンチマーク結果から得られた主要な知見を以下にまとめます。

1. Python (SudachiPy) との比較

 * オーバーヘッドの抑制: 1件ごとにFFIを呼び出すのではなく、まとめてデータを転送することで、言語境界を越える際のコストを最小化しています。
  
  * バッチ処理の優位性: 大量データの一括処理において、この「まとめてRustに投げる」手法が SudachiPy の逐次処理よりもわずか (JSONで入出力すると逆転する程度) ながら効率的であることが確認されました。

2. データ転送形式によるパフォーマンス差 (JSON vs Raw)

    Rust側でのメモリ挙動の差が、明確なパフォーマンス差として現れています。

  * JSON形式: Rust側でJSONをパースする際、改行や特殊文字のエスケープ解除に伴い、内部で**メモリの再確保** が発生します。これがボトルネックとなり、Raw形式より速度が低下します。

  * Raw形式: 長さ情報を付与したバイナリデータを直接渡すことで、Rust側でのメモリ再確保を最小限に抑えています。転送フォーマットを最適化する重要性が示されました。

3. マルチスレッドによる圧倒的な高速化

    rayon を利用したマルチスレッド実行は、シングルスレッドに対して圧倒的なアドバンテージがあります。

  * 並列化の恩恵: 物理コア・論理プロセッサをフル活用することで、シングルスレッド実行時の数倍のスループットを達成しました。

  * 安定性: CPU (i7-1355U) の特性上、シングルスレッドではOSのスケジューリングによる変動を受けやすいですが、マルチスレッドではリソースを使い切ることで高いパフォーマンスを安定して維持できます。

  * **注記:** マルチスレッド処理は、**配列として分割された複数の文字列を渡す**ことで最適化されます。巨大なテキストを1つの文字列として渡すと、単一のスレッドで処理されるため並列化の恩恵を受けられません。マルチスレッドの効果を最大化するには、あらかじめテキストを適切な単位 (記事単位など) で配列に分けて投入してください。

---

## 👀 ひとこと

Sudachi には Java・Python・CLI などの実装がありますが、
任意の言語へライブラリとして組み込み、高速にインプロセスで扱うという用途では、やや制約があると感じました。
本プロジェクトは、Rust 実装である sudachi.rs の性能と柔軟性を活かし、
FFI を通じてあらゆる言語から直接呼び出せる形態素解析コアとして構築しています。
日本語自然言語処理を、より自由に・より高速に扱うための基盤として、
さまざまな環境で活用してもらえたら嬉しいです。

This project is based on [sudachi.rs](https://github.com/WorksApplications/sudachi.rs) and includes modifications.