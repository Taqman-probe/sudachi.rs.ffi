# Sudachi FFI (Rust)

A library that enables Japanese morphological analyzer **Sudachi** (**[Sudachi.rs](https://github.com/WorksApplications/sudachi.rs)**) to be used from any language via FFI.
All options available in the original sudachi.rs, such as sentence splitting, morphological analysis, and tokenization (wakati), are implemented.
Additionally, I have extended it to support part-of-speech filtering.

[日本語 README.](README_ja.md)

---

## ✨ Features

* High-precision Japanese morphological analysis by Sudachi
* Sentence split (Sentence Split) toggle: ON / OFF / split only
* Wakati (word tokenization) support
* Detailed information output (print_all) toggle: ON / OFF
* Part-of-speech filtering (exclusion list specification) support
* High-speed processing with Rust implementation
* Available from any language via FFI (TypeScript / Deno samples included)
* Batch processing oriented (design that accepts an array of strings ["string1", "string2"])
* Callback support: Process large data sequentially in fixed-size chunks, dramatically reducing memory consumption
* **3 analysis methods**: General-purpose `analyze` (JSON), memory-efficient `analyze_raw` (Binary), and streaming analysis `analyze_callback` for huge data

---

## 🚀 Setup

### 1. Rust-side Build (Optional)

```bash
cargo build --release
```

This generates DLL / dylib / so files.

Example:
```
target/release/sudachi_ffi.dll
```

**You can download and use dynamic link libraries from Releases.**

---

### 2. Sudachi Configuration File and Dictionary Preparation (Required)

* Prepare Sudachi configuration files (sudachi.json, char.def) and dictionary.

  This repository includes a default configuration `char.def`, `sudachi_defabult.json` and `sudachi.json` which set plugins (see below).

* Download the dictionary from:

  [SudachiDict](https://d2ej7fkh96fzlu.cloudfront.net/sudachidict/)
  or
  [WorksApplications/SudachiDict](https://github.com/WorksApplications/SudachiDict)

  - Any of the dictionary files (small / core / full)

  ---

### 3. Usage Examples

#### Using from TypeScript & Deno

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
  wakati: false,      // Outputs only surface form
  printAll: true,   // Prints all fields
  splitSentences: 0, // default:0 / only:1 / none:2
  excludePos: ["記号", "助詞"], // part-of-speech exclusion settings
  multi: false // multi-threading
});

try {
  const result = sudachi.analyze(["今日はいい天気です。"]); // JSON format
  console.log(JSON.parse(result || "{}"));
} finally {
  sudachi.close();
  sudachi.dylibInstance.close();
}
```

#### Try with Copy-Paste (Deno needs to be installed separately)

* Bash (For Mac, change .so to .dylib)
```sh
mkdir -p sudachi-demo/resources && mkdir -p sudachi-demo/deno && cd sudachi-demo
# Download example
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
# Download example
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

#### Output

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

### Analysis Method Selection

This library provides 3 interfaces depending on your use case.

| Method | Input Format |	Features |
| ------ | ------------ | -------- |
| analyze	| JSON | Easy to use and versatile. The output is also in JSON format, making it easy to work with. However, the internal process of converting the input text to JSON requires reallocating memory, and since the output is also in JSON, the information density is low, resulting in a large output file size relative to the amount of input data. |
| analyze_raw |	Binary | Fastest parsing. Because it directly references the binary with length information provided by the host language, it uses memory efficiently. However, since the results are received as a single massive string, it can consume memory when dealing with extremely large datasets. |
| analyze_callback |Binary | For huge text. Returns results via callback, keeping memory consumption constant (estimated ~8MB output units). This can be processed as long as the input text is retained on the host language side. |

#### analyze_raw Usage Example and Results (Deno)

```ts
const rawResult = sudachi.analyzeRaw(["今日は来る？", "明日は行く。"]);
console.log(rawResult);
// When wakati: true
// 今日 は 来る ？
// 明日 は 行く 。
// EOS

// When wakati: false, print_all: true (double space separates input strings)
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

## 💡 Tips for Analyzing Large Text

Morphological analysis results are significantly larger than input text. In byte count terms with tab, comma, or newline delimiters...

Tokenization only: ~1.3x input size

Normal analysis (with POS): ~9-10x input size

With detailed info (reading, dictionary form): ~13-14x input size

When outputting POS information, output data increases 9x or more, so for text exceeding hundreds of MB, use analyze_callback to suppress output-side memory consumption.


When outputting part-of-speech information, the output data increases 9x or more; therefore, when analyzing text exceeding hundreds of MB, using `analyze_callback` to keep memory consumption on the output side constant is key to ensuring stable operation.

---

## ⚙️ Options

### `mode`

Sudachi analysis mode

| Value | Mode | Description |
| ----- | ---- | ----------- |
| 0 | A | Short |
| 1 | B | Middle |
| 2 | C | Named Entity |

---

### `splitSentences`

| Value | Mode | Description |
| ----- | ---- | ----------- |
| 0 | default | Sentence split + analysis |
| 1 | only | Sentence split only |
| 2 | none | Analysis without split |

---

### `wakati`

* `true`: Outputs only surface form
* `false`: See `print_all`

### `print_all`

* `true`: surface, poses (array), normalized form, dictionary form, reading form (, "is_oov": true for unknown words)
* `false`: surface, poses (array), normalized form

---

### `excludePos`

Exclude specific part-of-speech

Example:

```json
["記号", "助詞"]
```

### `multi`

* `true`: Multi-threaded mode, number of threads can be specified via **environment variable RAYON_NUM_THREADS** (example: export RAYON_NUM_THREADS=4). If not specified, it uses all logical processors.

* `false`: Single-threaded mode

---

## ⚠️ Notes

* Sudachi dictionary is required
* Build artifacts differ on Windows / Linux / Mac

### About Plugins

With default settings, the following plugins are used:

- default_input_text
- simple_oov
- join_numeric
- join_katakana_oov

#### Recommended: Built-in Style

Set sudachi_default.json uses plugins built into this FFI as aliases (class names matching Java version), so external files are not needed.

### Advanced: External Specification Style

Set sudachi.json requires specifying dynamic link library files.

Since these binaries (.dll / .so / .dylib) are not distributed, you need to build them from the sudachi.rs repository.

```sh
git clone https://github.com/WorksApplications/sudachi.rs
cd sudachi.rs
cargo build --release -p default_input_text
cargo build --release -p simple_oov
cargo build --release -p join_numeric
cargo build --release -p join_katakana_oov
```
After building, use the generated dynamic libraries.

---

🚀 Performance Verification

Benchmark results using [Livedoor Corpus](https://www.rondhuit.com/download.html) (approximately 25MB / 9 categories). Compared to SudachiPy (Python), this library (Deno FFI via Rust) runs approximately **4.5x faster** in the fastest multi-threaded raw style.

| benchmark             | time/iter (avg) |        iter/s |      (min … max)      |      p75 |      p99 |     p995 |
| --------------------- | --------------- | ------------- | --------------------- | -------- | -------- | -------- |
| Multi thread Raw      |           2.3 s |           0.4 | (   2.0 s …    2.5 s) |    2.5 s |    2.5 s |    2.5 s |
| Multi thread Callback |           2.6 s |           0.4 | (   2.5 s …    2.7 s) |    2.7 s |    2.7 s |    2.7 s |
| Multi thread JSON     |           3.6 s |           0.3 | (   3.3 s …    4.1 s) |    3.8 s |    4.1 s |    4.1 s |
| Single thread Raw     |           8.1 s |           0.1 | (   7.9 s …    8.7 s) |    8.2 s |    8.7 s |    8.7 s |
| Single thread Callback |           8.9 s |           0.1 | (   7.7 s …   13.1 s) |    9.0 s |   13.1 s |   13.1 s |
| Single thread JSON    |           9.6 s |           0.1 | (   8.9 s …   11.3 s) |   10.3 s |   11.3 s |   11.3 s |
| SudachiPy (Python) |              9.2 s |           0.1 | (   8.1 s …   10.6 s) |    9.3 s |   10.6 s |   10.6 s |

* **Test Environment:** [CPU | 13th Gen Intel(R) Core(TM) i7-1355U (10 Cores / 12 Threads) | Runtime | Deno 2.7.9 (x86_64-pc-windows-msvc)]
* **Conditions:** SplitMode: C, Wakati: ON
* **Note 1:** For practical purposes on Deno, object conversion is necessary, so the time includes conversion overhead. Pure analysis speed within Rust is even faster.
* **Note 2:** SudachiPy has input size limitations (approximately 49KB), so while the Deno script processes string arrays in bulk, Python processes them sequentially.

### Multi-threaded and Data Format
Key findings from this benchmark are summarized below.

1. Comparison with Python (SudachiPy)

  * Overhead suppression: By transferring data in batches rather than calling FFI for each item, I minimize the cost of crossing language boundaries.

  * Batch processing advantage: In bulk data processing, this "batch transfer to Rust" approach slightly outperforms SudachiPy's sequential processing (Although, it is reversed when using JSON input/output).

2. Performance differences by data transfer format (JSON vs Raw)

  Differences in memory behavior on Rust-side manifest as a clear performance differences.

  * JSON format: When parsing JSON on Rust side, **memory reallocation** occurs with escape character removal, becoming a bottleneck. This explains why JSON processing takes longer than Raw.

* Raw format: By directly passing binary data with length information, I minimize memory reallocation on the Rust side. Optimizing transfer format yields substantial performance gains.

3. Dramatic speed improvement with multi-threading

  Multi-threaded execution using rayon provides significant advantages over single-threaded.

  * Parallelization benefits: Full utilization of physical cores and logical processors achieves several times throughput of single-threaded execution.

  * Stability: Due to the nature of the CPU (i7-1355U), single-threaded performance is prone to fluctuations caused by OS scheduling; however, in multi-threaded scenarios, it can consistently maintain high performance by fully utilizing available resources.

  * **Note:** Multi-threaded processing is optimized **when multiple strings divided as arrays are passed**. Passing huge text as a single string doesn't leverage multi-threading benefits. To maximize the benefits of multi-threading, please divide the text into appropriate units (such as individual articles) and load them into an array beforehand.

---

## 👀 A Few Words

While Sudachi has Java, Python, CLI, and other implementations, I felt there were certain constraints in embedding it as a library in arbitrary languages and handling it in-process at high speed.
This project is being developed as a morphological analysis core that can be called directly from any language via FFI leveraging the performance and flexibility of the Rust implementation, sudachi.rs.

I hope this serves as a foundation for making Japanese natural language processing available, faster and more freely, in a variety of environments.

This project is based on [sudachi.rs](https://github.com/WorksApplications/sudachi.rs) and includes modifications.