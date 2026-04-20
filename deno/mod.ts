let libSuffix = "so";
if (Deno.build.os === "windows") {
  libSuffix = "dll";
} else if (Deno.build.os === "darwin") {
  libSuffix = "dylib";
}
const libPath = Deno.env.get("SUDACHI_FFI") || `./target/release/sudachi_ffi.${libSuffix}`;

export class Sudachi {
  private ptr: Deno.PointerValue;

  private dylib = Deno.dlopen(libPath, {
    init: {
      parameters: ["buffer", "i8", "i8", "i8", "i8", "buffer"],
      result: "pointer",
    },
    analyze: {
      parameters: ["pointer", "buffer", "buffer"],
      result: "pointer",
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
  }) {
    this.ptr = this.dylib.symbols.init(
      this.encode(config.configPath),
      config.mode,
      config.wakati ? 1 : 0,
      config.printAll ? 1 : 0,
      config.splitSentences,
      this.encode(JSON.stringify(config.excludePos)),
    );

    if (this.ptr === null) {
      throw new Error("Failed to initialize session");
    }
  }

  // Cから戻ってきたJSON文字列を読み取って解放するヘルパー
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
    const lenBuffer = new BigUint64Array(1); //確保するべき長さを格納させる
    const jsonInput = JSON.stringify(queries);
    const resultPtr = this.dylib.symbols.analyze(this.ptr, this.encode(jsonInput), lenBuffer);

    const actualLen = Number(lenBuffer[0]);
    const jsonStr = this.readAndFreeString(resultPtr, actualLen);
    return jsonStr ? jsonStr : null;
  }

  close() {
    if (this.ptr !== null) {
      this.dylib.symbols.free_sudachi(this.ptr);
      this.ptr = null;
    }
  }
}
