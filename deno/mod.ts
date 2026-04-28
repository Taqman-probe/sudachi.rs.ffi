let prefix = "lib";
let libSuffix = "so";
if (Deno.build.os === "windows") {
  libSuffix = "dll";
  prefix = "";
} else if (Deno.build.os === "darwin") {
  libSuffix = "dylib";
}
const libPath = Deno.env.get("SUDACHI_FFI") || `./target/release/${prefix}sudachi_ffi.${libSuffix}`;

export class Sudachi {
  private ptr: Deno.PointerValue;

  private dylib = Deno.dlopen(libPath, {
    init: {
      parameters: ["buffer", "i8", "i8", "i8", "i8", "buffer", "i8"],
      result: "pointer",
    },
    analyze: {
      parameters: ["pointer", "buffer", "buffer"],
      result: "pointer",
    },
    analyze_raw: {
      parameters: ["pointer", "buffer", "u64", "buffer"],
      result: "pointer",
    },
    analyze_callback: {
      parameters: ["pointer", "buffer", "u64", "function", "pointer"],
      result: "i32"      
    },
    
    free_string: {
      parameters: ["pointer"],
      result: "void",
    },
    free_sudachi: {
      parameters: ["pointer"],
      result: "void",
    },
  });

  get dylibInstance() {
    return this.dylib;
  }

  // 文字列をC文字列（null終端）に変換するヘルパー
  encode = (s: string) => {
    return new Uint8Array([...new TextEncoder().encode(s), 0]);
  };

  constructor(config: {
    configPath: string,
    mode: number, //0: A, 1: B, 2: C
    wakati: boolean,
    printAll: boolean,
    splitSentences: number, //0: Default, 1: Only, 2: None
    excludePos: Array<string>,
    multi: boolean,
  }) {
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
      throw new Error("Failed to initialize session");
    }
  }

  // Cから戻ってきた文字列を読み取って解放するヘルパー
  readAndFreeString(ptr: Deno.PointerValue, len: number): string | null {
    if (ptr === null || len === 0) return null;
    
    // len はヌル終端文字を含むバイト数なので、データ自体は len - 1
    const view = new Deno.UnsafePointerView(ptr);
    const buf = new Uint8Array(len - 1);
    view.copyInto(buf);
    
    const text = new TextDecoder().decode(buf);

    this.dylib.symbols.free_string(ptr);
    return text;
  }

  analyze(queries: string[]): string | null {
    const jsonInput = JSON.stringify(queries);
    const lenBuffer = new BigUint64Array(1); //確保するべき長さを格納させる
    const resultPtr = this.dylib.symbols.analyze(this.ptr, this.encode(jsonInput), lenBuffer);

    const actualLen = Number(lenBuffer[0]);
    const jsonStr = this.readAndFreeString(resultPtr, actualLen);
    return jsonStr || null;
  }

  analyzeRaw(queries: string[]): string | null {
    const maxPossibleSize = queries.reduce((acc, s) => acc + 4 + (s.length * 4), 0);
    const buffer = new Uint8Array(maxPossibleSize);
    const view = new DataView(buffer.buffer);
    const encoder = new TextEncoder();

    let offset = 0;
    for (const str of queries) {
      // Back-patching1: 長さの値4バイト分は空にしたまま先に文字列をエンコードしながら埋める
      const result = encoder.encodeInto(str, buffer.subarray(offset + 4));
      // Back-patching2: 実際に書き込まれたバイト数をoffset先頭に遡って長さ情報として記録
      view.setUint32(offset, result.written, true);
      offset += 4 + result.written;
    }
    const lenBuffer = new BigUint64Array(1); //確保するべき長さを格納させる
    const resultPtr = this.dylib.symbols.analyze_raw(this.ptr, buffer.subarray(0, offset), BigInt(offset), lenBuffer);
    return this.readAndFreeString(resultPtr, Number(lenBuffer[0])) || null;
  }

  analyzeCallback(
    queries: string[],
    onData: (text: string, userData: Deno.PointerValue) => void,
    userData: Deno.PointerValue = null,
  ): number {
    const maxPossibleSize = queries.reduce((acc, s) => acc + 4 + (s.length * 4), 0);
    const buffer = new Uint8Array(maxPossibleSize);
    const view = new DataView(buffer.buffer);
    const encoder = new TextEncoder();

    let offset = 0;
    for (const str of queries) {
      // Back-patching1: 長さの値4バイト分は空にしたまま先に文字列をエンコードしながら埋める
      const result = encoder.encodeInto(str, buffer.subarray(offset + 4));
      // Back-patching2: 実際に書き込まれたバイト数をoffset先頭に遡って長さ情報として記録
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
          // Rustから渡されたバイナリをデコード
          const array = new Uint8Array(Deno.UnsafePointerView.getArrayBuffer(bufPtr, Number(len)));
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
      userData
    );
    callback.close();
    return result;
  }

  close() {
    if (this.ptr !== null) {
      this.dylib.symbols.free_sudachi(this.ptr);
      this.ptr = null;
    }
  }
}
