import { Sudachi, SplitMode, SentenceSplitMode } from "./mod.ts";

let configPath = new URL("../resources/sudachi_default.json", import.meta.url).pathname;
if (Deno.build.os === "windows") {
  configPath = configPath.slice(1);
} 

type Morpheme = {
  surface: string;
  pos: Array<string>;
  normalized_form: string;
  dictionary_form?: string;
  reading_form?: string;
  dictionary_id?: number;
  synonym_group_ids?: Array<number>;
  is_oov?: boolean;
};

type EOS = {
  surface: "EOS" | "";
}

function splitMorpheme(morpheme: string): Morpheme | EOS {
  if (morpheme === "EOS") {
    return {
      surface: "EOS",
    } as EOS;
  }

  const parts = morpheme.split("\t");
  const result: Morpheme = {
    surface: parts[0],
    pos: parts[1].split(","), 
    normalized_form: parts[2],
  }

  if (parts.length > 3) {
    result.dictionary_form = parts[3];
    result.reading_form = parts[4];
    result.dictionary_id = parseInt(parts[5]);
    result.synonym_group_ids = JSON.parse(parts[6]);
  }

  if (parts.length > 7) {
    result.is_oov = parts[7] === "OOV";
  }

  return result;
}

// 詳細情報出力モード
let sudachi: Sudachi;
sudachi = new Sudachi ({
  configPath: configPath,
  mode: SplitMode.C,           // A:0 / B:1 / C:2
  wakati: false,      // 分かち書き
  printAll: true,   // 詳細情報出力
  splitSentences: SentenceSplitMode.Default, // default:0 / only:1 / none:2
  excludePos: ["補助記号", "助詞"], // 品詞除外設定
  multi: false // マルチスレッド
});

const resultsDetail: Array<Array<Morpheme | EOS>> = [];
let isEOS = false;
const userDataId = new BigUint64Array([64n]);
const userData = Deno.UnsafePointer.of(userDataId);

const callbackDetail = (rawString: string, userPtr: Deno.PointerValue) => {
  console.log(`userData in Callback: ${new Deno.UnsafePointerView(userPtr as Deno.PointerObject).getBigInt64()}`);
  const inter: Array<Morpheme | EOS> = [];
  let morpheme: Morpheme | EOS = { surface: "" } as EOS;
  let str = "";
  let eol = false;
  for (const char of rawString) {
    if (char === "\n" && !eol) {
      morpheme = splitMorpheme(str);
      inter.push(morpheme);
      str = "";
      eol = true;
    } else if (char === "\n" && eol) {
      resultsDetail.push([...inter]);
      inter.splice(0);
      eol = false;
    } else {
      eol = false;
      str += char;
    }
  }
  if (morpheme.surface == "EOS") {
    isEOS = true;
  }
}

try {
  const num = sudachi.analyzeCallback([`今日は来る？`, `明日は行く。`], callbackDetail, userData);
  console.log(num);
  console.log(resultsDetail);
  if (!isEOS) {console.log("Stream truncated: EOS not found")}
} finally {
  sudachi.close();
  sudachi.dylibInstance.close();
}

// 分かち書きモード
sudachi = new Sudachi ({
  configPath: configPath,
  mode: SplitMode.C,           // A:0 / B:1 / C:2
  wakati: true,      // 分かち書き
  printAll: false,   // 詳細情報出力
  splitSentences: SentenceSplitMode.Default, // default:0 / only:1 / none:2
  excludePos: ["補助記号", "助詞"], // 品詞除外設定
  multi: false // マルチスレッド
});

const resultsWakati: Array<Array<string>> = [];
const callbackWakati = (rawString: string, userPtr: Deno.PointerValue) => {
  console.log(`userData in Callback: ${new Deno.UnsafePointerView(userPtr as Deno.PointerObject).getBigInt64()}`);
  let str = "";
  for (const char of rawString) {
    if (char === "\n") {
      if (str !== "EOS") {
        resultsWakati.push(str.split(" "));
      }
      str = "";
    } else {
      str += char;
    }
  }
}

try {
  const num = sudachi.analyzeCallback([`今日は来る？`, `明日は行く。`], callbackWakati, userData);
  console.log(num);
  console.log(resultsWakati);
} finally {
  sudachi.close();
  sudachi.dylibInstance.close();
}