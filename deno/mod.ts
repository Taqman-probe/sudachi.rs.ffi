/**
 * sudachi.rs.ffi (Foreign Function Interface) for Deno
 * 
 * A high-performance Japanese morphological analyzer wrapper around sudachi.rs
 * * Requires permissions:
 * - `--allow-ffi` (to load sudachi_ffi)
 * - `--allow-env` (to read SUDACHI_FFI path)
 * - `--allow-read` (to load configuration and dictionary files)
 * 
 * @example
 * ```ts
 * import { Sudachi, SplitMode, SentenceSplitMode } from "sudachi-ffi";
 * 
 * const sudachi = new Sudachi({
 *   configPath: "./sudachi_default.json",
 *   mode: SplitMode.C,
 *   wakati: false,
 *   printAll: false,
 *   splitSentences: SentenceSplitMode.Default,
 *   excludePos: [],
 *   multi: false,
 * });
 * 
 * const result = sudachi.analyze(["今日はいい天気です。"]);
 * console.log(JSON.parse(result || "{}"));
 * 
 * sudachi.close();
 * sudachi.dylibInstance.close();
 * ```
 * 
 * @module
 */

import { join } from "jsr:@std/path@^1.0.0";

let prefix = "lib";
let libSuffix = "so";

if (Deno.build.os === "windows") {
  libSuffix = "dll";
  prefix = "";
} else if (Deno.build.os === "darwin") {
  libSuffix = "dylib";
}

const libName = `./${prefix}sudachi_ffi.${libSuffix}`;

const libPath = Deno.env.get("SUDACHI_FFI") || libName;

/**
 * Analysis modes for Sudachi
 */
export enum SplitMode {
  /** Mode A: Short */
  A = 0,
  /** Mode B: Middle */
  B = 1,
  /** Mode C: Named Entity */
  C = 2,
}

/**
 * Analysis modes for Sudachi
 */
export enum SentenceSplitMode {
  /** Default: Sentence split + analysis */
  Default = 0,
  /** Only: Sentence split only */
  Only = 1,
  /** None: Analysis without split */
  None = 2,
}

/** Configuration options for Sudachi initialization */
export type SudachiConfig = {
  /** Path to sudachi.json configuration file */
  configPath: string;
  /** Analysis mode: SplitMode.A, B, or C */
  mode: SplitMode;
  /** Output in wakati format (space-separated tokens) */
  wakati: boolean;
  /** Print all tokens including unknown words */
  printAll: boolean;
  /** Sentence splitting mode: Default, Only or None */
  splitSentences: SentenceSplitMode;
  /** Part-of-speech tags to exclude from output */
  excludePos: string[];
  /** Enable multi-threaded analysis */
  multi: boolean;
}

const SYMBOLS = {
  init: { parameters: ["buffer", "i8", "i8", "i8", "i8", "buffer", "i8"], result: "pointer" },
  analyze: { parameters: ["pointer", "buffer", "buffer"], result: "pointer" },
  analyze_raw: { parameters: ["pointer", "buffer", "u64", "buffer"], result: "pointer" },
  analyze_callback: { parameters: ["pointer", "buffer", "u64", "function", "pointer"], result: "i32" },
  free_string: { parameters: ["pointer"], result: "void" },
  free_sudachi: { parameters: ["pointer"], result: "void" },
} as const;

/**
 * Copies the default Sudachi configuration and asset files to a local directory.
 * * This is a helper function to set up required files (`default_config.json`, `char.def`)
 * in your project environment. It fetches assets from the module's distribution
 * and writes them to the specified path.
 * * @param {string} [targetDir="."] - The destination directory where assets will be copied.
 * @returns {Promise<void>}
 * * @example
 * ```ts
 * import { setupDefaultAssets } from "@your-scope/sudachi";
 * * // Setup assets in the current directory
 * await setupDefaultAssets();
 * ```
 * * @throws {Deno.errors.PermissionDenied} If the process lacks write permissions for the target directory.
 * @throws {Error} If the assets cannot be fetched or written.
 */
export async function setupDefaultAssets(targetDir: string = "."): Promise<void> {
  const libName = Deno.build.os === "windows" ? "sudachi_ffi.dll" : 
                  Deno.build.os === "darwin" ? "libsudachi_ffi.dylib" : 
                  "libsudachi_ffi.so";
  const assets = [
    "sudachi_default.json",
    "char.def",
    libName,
  ];

  for (const filename of assets) {
    const sourceUrl = new URL(`./${filename}`, import.meta.url);
    const destPath = join(targetDir, filename);

    try {
      const response = await fetch(sourceUrl);
      if (!response.ok) throw new Error(`Failed to fetch ${filename}`);
      
      const data = await response.arrayBuffer();
      await Deno.writeFile(destPath, new Uint8Array(data));
      
      console.log(`[Sudachi] Created: ${destPath}`);
    } catch (err) {
      console.error(`[Sudachi] Failed to copy ${filename}:`, err);
      throw err;
    }
  }
}

/**
 * Main Sudachi class for Japanese text morphological analysis
 * 
 * This class provides FFI bindings to the Sudachi morphological analyzer
 * written in Rust. It handles memory management and provides convenient
 * methods for analyzing Japanese text.
 */
export class Sudachi {
  private ptr: Deno.PointerValue;

  private dylib = Deno.dlopen(libPath, SYMBOLS);

  /**
   * Get the underlying Deno dynamic library instance
   */
  get dylibInstance(): Deno.DynamicLibrary<typeof SYMBOLS> {
    return this.dylib;
  }

  /**
   * Convert a JavaScript string to a null-terminated C string buffer
   */
  private encode = (s: string): Uint8Array<ArrayBuffer> => {
    return new Uint8Array([...new TextEncoder().encode(s), 0]);
  };

  /**
   * Initialize a new Sudachi analyzer instance
   * 
   * @param config Configuration options for the analyzer
   * @throws {Error} If initialization fails
   */
  constructor(config: SudachiConfig) {
    this.ptr = this.dylib.symbols.init(
      this.encode(config.configPath),
      config.mode,
      config.wakati ? 1 : 0,
      config.printAll ? 1 : 0,
      config.splitSentences,
      this.encode(JSON.stringify(config.excludePos)),
      config.multi ? 1 : 0,
    );

    if (this.ptr === null) {
      throw new Error("Failed to initialize Sudachi session");
    }
  }

  /**
   * Read a C string from memory and free its pointer
   * 
   * @internal
   */
  private readAndFreeString(
    ptr: Deno.PointerValue,
    len: number,
  ): string | null {
    if (ptr === null || len === 0) return null;

    // len includes the null terminator, so actual data is len - 1
    const view = new Deno.UnsafePointerView(ptr);
    const buf = new Uint8Array(len - 1);
    view.copyInto(buf);

    const text = new TextDecoder().decode(buf);

    this.dylib.symbols.free_string(ptr);
    return text;
  }

  /**
   * Analyze Japanese text using standard format
   * 
   * @param queries Array of Japanese text to analyze
   * @returns JSON string containing analysis results, or null if analysis fails
   * 
   * @example
   * ```ts
   * const result = sudachi.analyze(["形態素解析"]);
   * console.log(result);
   * ```
   */
  public analyze(queries: string[]): string | null {
    const jsonInput = JSON.stringify(queries);
    const lenBuffer = new BigUint64Array(1);
    const resultPtr = this.dylib.symbols.analyze(
      this.ptr,
      this.encode(jsonInput),
      lenBuffer,
    );

    const actualLen = Number(lenBuffer[0]);
    const jsonStr = this.readAndFreeString(resultPtr, actualLen);
    return jsonStr || null;
  }

  /**
   * Analyze Japanese text using raw binary format
   * 
   * More efficient for large batches of text
   * 
   * @param queries Array of Japanese text to analyze
   * @returns JSON string containing analysis results, or null if analysis fails
   */
  public analyzeRaw(queries: string[]): string | null {
    const maxPossibleSize = queries.reduce(
      (acc, s) => acc + 4 + (s.length * 4),
      0,
    );
    const buffer = new Uint8Array(maxPossibleSize);
    const view = new DataView(buffer.buffer);
    const encoder = new TextEncoder();

    let offset = 0;
    for (const str of queries) {
      const result = encoder.encodeInto(str, buffer.subarray(offset + 4));
      view.setUint32(offset, result.written, true);
      offset += 4 + result.written;
    }

    const lenBuffer = new BigUint64Array(1);
    const resultPtr = this.dylib.symbols.analyze_raw(
      this.ptr,
      buffer.subarray(0, offset),
      BigInt(offset),
      lenBuffer,
    );

    return this.readAndFreeString(resultPtr, Number(lenBuffer[0])) || null;
  }

  /**
   * Analyze Japanese text with a callback for streaming results
   * 
   * Useful for processing large amounts of text without loading entire results into memory
   * 
   * @param queries Array of Japanese text to analyze
   * @param onData Callback function called for each result
   * @param userData Optional user data passed to the callback
   * @returns Number of results processed
   */
  public analyzeCallback(
    queries: string[],
    onData: (text: string, userData: Deno.PointerValue) => void,
    userData: Deno.PointerValue = null,
  ): number {
    const maxPossibleSize = queries.reduce(
      (acc, s) => acc + 4 + (s.length * 4),
      0,
    );
    const buffer = new Uint8Array(maxPossibleSize);
    const view = new DataView(buffer.buffer);
    const encoder = new TextEncoder();

    let offset = 0;
    for (const str of queries) {
      const result = encoder.encodeInto(str, buffer.subarray(offset + 4));
      view.setUint32(offset, result.written, true);
      offset += 4 + result.written;
    }

    const callback = new Deno.UnsafeCallback(
      {
        parameters: ["buffer", "u64", "pointer"],
        result: "void",
      },
      (bufPtr, len, userPtr) => {
        if (bufPtr) {
          const array = new Uint8Array(
            Deno.UnsafePointerView.getArrayBuffer(bufPtr, Number(len)),
          );
          const text = new TextDecoder().decode(array);
          onData(text, userPtr);
        }
      },
    );

    const result = this.dylib.symbols.analyze_callback(
      this.ptr,
      buffer.subarray(0, offset),
      BigInt(offset),
      callback.pointer,
      userData,
    );

    callback.close();
    return result;
  }

  /**
   * Close and free the Sudachi analyzer instance
   * 
   * Must be called to prevent memory leaks
   */
  public close(): void {
    if (this.ptr !== null) {
      this.dylib.symbols.free_sudachi(this.ptr);
      this.ptr = null;
    }
  }
}