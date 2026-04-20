import { assertEquals } from "jsr:@std/assert@1.0.19";
import { Sudachi } from "./mod.ts";

const baseConfig = {
  configPath: new URL("../resources/sudachi.json", import.meta.url).pathname.slice(1),
  mode: 2,
  wakati: false,
  printAll: false,
  splitSentences: 0,
  excludePos: [],
};

Deno.test("Sudachi FFI", async (t) => {

  await t.step("1. split_sentences: Only (文分割のみ)", () => {
    const sudachi = new Sudachi({ ...baseConfig, splitSentences: 1 });
    try {
      const res = sudachi.analyze(["今日はいい天気です！明日は雨かな？明後日は知らん。"]);
      const json = JSON.parse(res!);
      assertEquals(json[0], ["今日はいい天気です！", "明日は雨かな？", "明後日は知らん。"]);
    } finally {
      sudachi.close();
      sudachi.dylibInstance.close();
    }
  });

  await t.step("2. wakati: true (わかち書きモード)", () => {
    const sudachi = new Sudachi({ ...baseConfig, wakati: true });
    try {
      const res = sudachi.analyze(["プロジェクトに関するお問い合わせ。"]);
      const json = JSON.parse(res!);
      // Mode A (0) なので細かく分割されるはず
      assertEquals(json[0], ["プロジェクト","に","関する","お","問い合わせ","。"]);
    } finally {
      sudachi.close();
      sudachi.dylibInstance.close();
    }
  });

  await t.step("3. wakati: false, print_all: false (標準構造)", () => {
    const sudachi = new Sudachi({ ...baseConfig });
    try {
      const res = sudachi.analyze(["外国人参政権"]);
      const json = JSON.parse(res!);
      const word = json[0][0];
      assertEquals(word.surface, "外国人参政権");
      assertEquals(word.poses[0], "名詞");
      assertEquals(word.normalized_form, "外国人参政権");
      assertEquals(word.dictionary_form, undefined);
      assertEquals(word.reading_form, undefined);
    } finally {
      sudachi.close();
      sudachi.dylibInstance.close();
    }
  });

  await t.step("4. print_all: true (詳細構造)", () => {
    const sudachi = new Sudachi({ ...baseConfig, printAll: true });
    try {
      const res = sudachi.analyze(["外国人参政権"]);
      const json = JSON.parse(res!);
      const word = json[0][0];
      assertEquals(word.surface, "外国人参政権");
      assertEquals(word.poses[0], "名詞");
      assertEquals(word.normalized_form, "外国人参政権");
      assertEquals(word.dictionary_form, "外国人参政権");
      assertEquals(word.reading_form, "外国人参政権");
    } finally {
      sudachi.close();
      sudachi.dylibInstance.close();
    }
  });

  await t.step("5. exclude_pos (品詞除外設定)", () => {
    const sudachi = new Sudachi({ 
      ...baseConfig, 
      wakati: true, 
      excludePos: ["助詞", "接頭辞", "補助記号"] 
    });
    try {
      const res = sudachi.analyze(["プロジェクトに関するお問い合わせ。"]);
      const json = JSON.parse(res!);
      assertEquals(json[0].includes("に"), false);
      assertEquals(json[0], ["プロジェクト","関する","問い合わせ"]);
    } finally {
      sudachi.close();
      sudachi.dylibInstance.close();
    }
  });
});