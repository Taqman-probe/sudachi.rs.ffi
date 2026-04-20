import { Sudachi } from "./mod.ts";

const sudachi = new Sudachi ({
  configPath: new URL("../resources/sudachi.json", import.meta.url).pathname.slice(1),
  mode: 2,           // A:0 / B:1 / C:2
  wakati: false,      // 分かち書き
  printAll: true,   // 詳細情報出力
  splitSentences: 0, // default:0 / only:1 / none:2
  excludePos: ["記号", "助詞"]
});

try {
  const result = sudachi.analyze(["今日はいい天気です。"]);
  console.log(JSON.parse(result || "{}"));
} finally {
  sudachi.close();
  sudachi.dylibInstance.close();
}