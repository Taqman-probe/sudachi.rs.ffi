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
  excludePos: ["補助記号", "助詞"], // 品詞除外設定
  multi: false // マルチスレッド
});

try {
  const result = sudachi.analyze(["今日はいい天気です。"]);
  console.log(JSON.parse(result || "{}"));
} finally {
  sudachi.close();
  sudachi.dylibInstance.close();
}