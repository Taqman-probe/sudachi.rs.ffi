# @taqman/sudachi-ffi

A high-performance Japanese morphological analyzer for **Deno**, powered by [Sudachi.rs](https://github.com/WorksApplications/sudachi.rs) via FFI.

This package provides a seamless, fast, and flexible bridge to use Sudachi's advanced Japanese text processing capabilities directly within your TypeScript/JavaScript projects.

## ✨ Key Features

* **High Performance:** Leverages Rust's speed via Deno FFI. Up to **4.5x faster** than SudachiPy in multi-threaded benchmarks.
* **Flexible Analysis:** Supports all standard Sudachi modes (A, B, and C).
* **Advanced Controls:** Toggle sentence splitting, wakati (segmentation), and part-of-speech filtering.
* **Memory Optimized:** Includes 3 analysis methods:
  * `analyze:` Standard JSON output.
  * `analyzeRaw:` High-speed binary transfer.
  * `analyzeCallback:` Streaming support for massive datasets to keep memory usage constant.
* **Multi-threading:** Native support for parallel processing using Rayon.

---

## 🚀 Quick Start

### 1. Permissions

Since this module uses FFI, you need to run Deno with the following flags:

* `--allow-ffi:` To load the native dynamic library.
* `--allow-env:` To check for the `SUDACHI_FFI` path.
* `--allow-read:` To read dictionary and configuration files.

### 2. Setup Assets

You need Sudachi's configuration files and a system dictionary.

You can use the helper function to set up the default assets:

**TypeScript**
```bash
deno eval "import { setupDefaultAssets } from 'jsr:@taqman/sudachi-ffi@^0.6.0'; await setupDefaultAssets();"
```

[!IMPORTANT]

**Dictionary Requirement:** You must manually download a system dictionary (e.g., `system_core.dic`) from [SudachiDict](https://github.com/WorksApplications/SudachiDict).

When using the provided `sudachi_default.json`, **place the dictionary file directly in your project's current directory**.

### 3. Usage Example

**TypeScript**
```ts
import { Sudachi, SplitMode, SentenceSplitMode } from "jsr:@taqman/sudachi-ffi";

const sudachi = new Sudachi({
  configPath: "./sudachi_default.json",
  mode: SplitMode.C,
  wakati: false,
  printAll: true,
  splitSentences: SentenceSplitMode.Default,
  excludePos: ["補助記号", "助詞"],
  multi: false,
});

try {
  const result = sudachi.analyze(["今日はいい天気です。"]);
  console.log(JSON.parse(result || "[]"));
} finally {
  sudachi.close();
  // Don't forget to close the library instance if you are done
  sudachi.dylibInstance.close();
}
```

---

## ⚙️ Configuration

| Option       | Type        | Description |
| ------------ | ----------- | ----------- |
| `mode`       | `SplitMode` | `A` (Short), `B` (Middle), or `C` (Named Entity) |
| `wakati`     | `boolean`   | If true, returns space-separated surface forms only. |
| `printAll`   | `boolean`   | Includes reading form and dictionary form. |
| `excludePos` | `string[]`  | List of Part-of-Speech tags to filter out. |
| `multi`      | `boolean`   | Enables multi-threaded analysis for arrays of strings. |

---

## 📊 Performance

ComparisonBenchmarked using the Livedoor Corpus (~25MB text).

| Environment        | Mode              | Time/Iter (Avg) |
| ------------------ | ----------------- | --------------- |
| **Deno + Sudachi FFI** | **Multi-thread Raw**  |           **2.3 s** |
| Deno + Sudachi FFI | Single-thread Raw |           8.1 s |
| Python (SudachiPy) | Sequential        |           9.2 s |

*Hardware: i7-1355U (10 Cores). Multi-threading is most effective when passing an array of multiple strings.*

---

## 🛠️ Library Management

### Default Behavior (Zero Config)
This package **includes pre-compiled binaries** for Windows, macOS, and Linux. It works out-of-the-box without any manual binary placement.

### Using your own Binary (Recommended for Production)
If you prefer to manage the binary yourself—for transparency, security auditing, or using a custom-optimized build—you can point to your specific binary file using the `SUDACHI_FFI` environment variable.

**Example: Pointing to a binary in your project folder**
```sh
# Set the path to your verified binary
export SUDACHI_FFI="./bin/libsudachi_ffi.so"
```


## 📄 License

This project is licensed under the **Apache-2.0 License**.

It is based on and includes modifications to [sudachi.rs](https://github.com/WorksApplications/sudachi.rs).
