// BASE_PATHは/区切りで最後にも/を付ける必要がある (例: C:/temp/)
import { Sudachi } from "./mod.ts";

const categories = [
  "dokujo-tsushin", "it-life-hack", "kaden-channel",
  "livedoor-homme", "movie-enter", "peachy",
  "smax", "sports-watch", "topic-news"
];

async function loadLivedoorCorpus(): Promise<string[]> {
  const allTexts: string[] = [];

  for (const category of categories) {
    const dirPath = new URL("./"+category+"/", "file:///"+Deno.env.get("BASE_PATH"));

    try {
      for await (const entry of Deno.readDir(dirPath)) {
        if (entry.isFile && entry.name.endsWith(".txt")) {
          if (entry.name === "LICENSE.txt") continue;

          const text = await Deno.readTextFile(new URL("./"+entry.name, dirPath));
          
          // コーパス特有の処理：
          // 最初の2行はURLと日付なので、3行目以降（本文）を抽出する
          const lines = text.split("\n").slice(2).join("\n").trim();
          
          if (lines) {
            allTexts.push(lines);
          }
        }
      }
      console.log(`Loaded: ${category} (${allTexts.length} items total)`);
    } catch (err) {
      console.log(err);
      console.warn(`Warning: Could not read directory ${category}. skipping...`);
    }
  }
  return allTexts;//.concat(allTexts);
}

const inputs = await loadLivedoorCorpus();

console.log(`全データ読み込み完了: ${inputs.length} 件`);

let configPath = new URL("../resources/sudachi_default.json", import.meta.url).pathname;
if (Deno.build.os === "windows") {
  configPath = configPath.slice(1);
} 

const baseConfig = {
  configPath: configPath,
  mode: 2,
  wakati: true,
  printAll: false,
  splitSentences: 0,
  excludePos: [],
  multi: false,
};

Deno.bench("Multi thread Raw", () => {
  const sudachi = new Sudachi ({...baseConfig, multi: true});
  try {
    const results: Array<Array<string>> = [];
    let str = "";
    const rawString = sudachi.analyzeRaw(inputs) || "";
    for (const char of rawString) {
      if (char === "\n") {
        if (str !== "EOS") {
          results.push(str.split(" "));
        }
        str = "";
      } else {
        str += char;
      }
    }
  } finally {
    sudachi.close();
    sudachi.dylibInstance.close();
  }
});

Deno.bench("Multi thread Callback", () => {
  const sudachi = new Sudachi ({...baseConfig, multi: true});
  const callback = (rawString: string) => {
    const results: Array<Array<string>> = [];
    let str = "";
    for (const char of rawString) {
      if (char === "\n") {
        if (str !== "EOS") {
          results.push(str.split(" "));
        }
        str = "";
      } else {
        str += char;
      }
    }
  }; 
  try {
    sudachi.analyzeCallback(inputs, callback);
  } finally {
    sudachi.close();
    sudachi.dylibInstance.close();
  }
});

Deno.bench("Multi thread JSON", () => {
  const sudachi = new Sudachi ({...baseConfig, multi: true});
  try {
    JSON.parse(sudachi.analyze(inputs)||"{}");
  } finally {
    sudachi.close();
    sudachi.dylibInstance.close();
  }
});

Deno.bench("Single thread Raw", () => {
  const sudachi = new Sudachi ({...baseConfig});
  try {
    const results: Array<Array<string>> = [];
    let str = "";
    const rawString = sudachi.analyzeRaw(inputs) || "";
    for (const char of rawString) {
      if (char === "\n") {
        if (str !== "EOS") {
          results.push(str.split(" "));
        }
        str = "";
      } else {
        str += char;
      }
    }
  } finally {
    sudachi.close();
    sudachi.dylibInstance.close();
  }
});


Deno.bench("Single thread Callback", () => {
  const sudachi = new Sudachi ({...baseConfig});
  const callback = (rawString: string) => {
    const results: Array<Array<string>> = [];
    let str = "";
    for (const char of rawString) {
      if (char === "\n") {
        if (str !== "EOS") {
          results.push(str.split(" "));
        }
        str = "";
      } else {
        str += char;
      }
    }
  }; 
  try {
    sudachi.analyzeCallback(inputs, callback);
  } finally {
    sudachi.close();
    sudachi.dylibInstance.close();
  }
});

Deno.bench("Single thread JSON", () => {
  const sudachi = new Sudachi ({...baseConfig});
  try {
    JSON.parse(sudachi.analyze(inputs) || "{}");
  } finally {
    sudachi.close();
    sudachi.dylibInstance.close();
  }
});
