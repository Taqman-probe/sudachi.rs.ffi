import { assertEquals } from "jsr:@std/assert@1.0.19";
import { Sudachi } from "./mod.ts";

let configPath = new URL("../resources/sudachi_default.json", import.meta.url).pathname;
if (Deno.build.os === "windows") {
  configPath = configPath.slice(1);
} 

const baseConfig = {
  configPath: configPath,
  mode: 2,
  wakati: false,
  printAll: false,
  splitSentences: 0,
  excludePos: [],
  multi: false,
};

Deno.test("Sudachi FFI JSON", async (t) => {

  await t.step("1. split_sentences: Only (文分割のみ)", () => {
    const sudachi = new Sudachi({ ...baseConfig, splitSentences: 1 });
    try {
      const res = sudachi.analyze(["今日はいい天気です！明日は雨かな？", "明後日は知らん。"]);
      const json = JSON.parse(res!);
      assertEquals(json, [["今日はいい天気です！", "明日は雨かな？"], ["明後日は知らん。"]]);
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
      assertEquals(word.reading_form, "ガイコクジンサンセイケン");
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

  await t.step("6. multi (マルチスレッド)", () => {
    const sudachi = new Sudachi({ 
      ...baseConfig, 
      wakati: true,
      multi: true
    });
    try {
      const res = sudachi.analyze(["今日はいい天気です！", "明日は雨かな？", "明後日は知らん。"]);
      const json = JSON.parse(res!);
      assertEquals(json.length, 3);
      assertEquals(json[0][0], "今日");
      assertEquals(json[1][0], "明日");
      assertEquals(json[2][0], "明後日");
    } finally {
      sudachi.close();
      sudachi.dylibInstance.close();
    }
  });
});

Deno.test("Sudachi FFI Raw", async (t) => {
  await t.step("1. split_sentences: Only (文分割のみ)", () => {
    const sudachi = new Sudachi({ ...baseConfig, splitSentences: 1 });
    try {
      const res = sudachi.analyzeRaw(["今日はいい天気です！明日は雨かな？", "明後日は知らん。"]);
      assertEquals(res, "今日はいい天気です！\n明日は雨かな？\n\n明後日は知らん。\n\nEOS\n");
    } finally {
      sudachi.close();
      sudachi.dylibInstance.close();
    }
  });

  await t.step("2. wakati: true (わかち書きモード)", () => {
    const sudachi = new Sudachi({ ...baseConfig, wakati: true });
    try {
      const res = sudachi.analyzeRaw(["プロジェクトに関するお問い合わせ。"]);
      // Mode A (0) なので細かく分割されるはず
      assertEquals(res, "プロジェクト に 関する お 問い合わせ 。\nEOS\n");
    } finally {
      sudachi.close();
      sudachi.dylibInstance.close();
    }
  });

  await t.step("3. wakati: false, print_all: false (標準構造)", () => {
    const sudachi = new Sudachi({ ...baseConfig });
    try {
      const res = sudachi.analyzeRaw(["外国人参政権"]);
      assertEquals(res, "外国人参政権\t名詞,普通名詞,一般,*,*,*\t外国人参政権\n\nEOS\n");
    } finally {
      sudachi.close();
      sudachi.dylibInstance.close();
    }
  });

  await t.step("4. print_all: true (詳細構造)", () => {
    const sudachi = new Sudachi({ ...baseConfig, printAll: true });
    try {
      const res = sudachi.analyzeRaw(["外国人参政権"]);
      assertEquals(res, "外国人参政権\t名詞,普通名詞,一般,*,*,*\t外国人参政権\t外国人参政権\tガイコクジンサンセイケン\t0\t[]\n\nEOS\n");

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
      const res = sudachi.analyzeRaw(["プロジェクトに関するお問い合わせ。"]);
      assertEquals(res, "プロジェクト 関する 問い合わせ\nEOS\n");
    } finally {
      sudachi.close();
      sudachi.dylibInstance.close();
    }
  });

  await t.step("6. multi (マルチスレッド)", () => {
    const sudachi = new Sudachi({ 
      ...baseConfig, 
      wakati: true,
      multi: true
    });
    try {
      const res = sudachi.analyzeRaw(["今日はいい天気です！", "明日は雨かな？", "明後日は知らん。"]);
      assertEquals(res, "今日 は いい 天気 です ！\n明日 は 雨 か な ？\n明後日 は 知ら ん 。\nEOS\n");
    } finally {
      sudachi.close();
      sudachi.dylibInstance.close();
    }
  });
});


Deno.test("Sudachi FFI Callback", async (t) => {
await t.step("1. 基本的なコールバック実行", () => {
  const sudachi = new Sudachi({ ...baseConfig, wakati: true });
  try {
    let fullResult = "";
    // mod.ts の型定義 (text: string, userData: Deno.PointerValue) に合わせる
    sudachi.analyzeCallback(
      ["今日はいい天気です！", "明日は雨かな？", "明後日は知らん。"],
      (text, _userPtr) => {
        fullResult += text;
      }
    );
    assertEquals(fullResult, "今日 は いい 天気 です ！\n明日 は 雨 か な ？\n明後日 は 知ら ん 。\nEOS\n");
  } finally {
    sudachi.close();
    sudachi.dylibInstance.close();
  }
});

  await t.step("2. 大量データのチャンク分割テスト", () => {
    const sudachi = new Sudachi({ ...baseConfig, wakati: true });
    
    try {
      let lineCount = 0;
      // 200行のデータを投入して動的チャンク分割を走らせる
      let fullResult = "";
      const input = Array(200).fill("これはテストデータです。");
      sudachi.analyzeCallback(input,
        (text, _userPtr) => {
        fullResult += text;
        lineCount += text.split("\n").filter(l => l.trim() !== "" && l !== "EOS").length;
      });

      assertEquals(lineCount, 200);
    } finally {
      sudachi.close();
      sudachi.dylibInstance.close();
    }
  });
});
