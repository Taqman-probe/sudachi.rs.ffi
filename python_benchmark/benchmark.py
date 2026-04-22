import os
from pathlib import Path
import time
from sudachipy import tokenizer
from sudachipy import dictionary

# カテゴリ一覧
CATEGORIES = [
    "dokujo-tsushin", "it-life-hack", "kaden-channel",
    "livedoor-homme", "movie-enter", "peachy",
    "smax", "sports-watch", "topic-news"
]

def load_livedoor_corpus() -> str:
    all_texts = []
    # 環境変数 BASE_PATH を取得（未設定の場合はカレントディレクトリを基準に）
    base_path_str = os.getenv("BASE_PATH", ".")
    base_path = Path(base_path_str)

    for category in CATEGORIES:
        dir_path = base_path / category
        
        if not dir_path.exists():
            print(f"Warning: Could not find directory {category}. skipping...")
            continue

        try:
            # ディレクトリ内のファイルを走査
            file_count = 0
            for file_path in dir_path.glob("*.txt"):
                # LICENSE.txt は除外
                if file_path.name == "LICENSE.txt":
                    continue
                
                # ファイルを読み込み
                with open(file_path, "r", encoding="utf-8") as f:
                    text = f.read()
                
                # 最初の2行（URLと日付）をスキップし、3行目以降を抽出
                lines = text.splitlines()
                body = "\n".join(lines[2:]).strip()
                
                if body:
                    all_texts.append(body)
                    file_count += 1
            
            print(f"Loaded: {category} (Total items in list: {len(all_texts)})")
            
        except Exception as e:
            print(f"Error reading {category}: {e}")

    # 配列を改行で結合して一つの文字列にする
    return all_texts

# 実行例
corpus_text = load_livedoor_corpus()
tokenizer_obj = dictionary.Dictionary().create()
mode = tokenizer.Tokenizer.SplitMode.C

start = time.perf_counter()
results = [tokenizer_obj.tokenize(text, mode) for text in corpus_text]
end = time.perf_counter()

print(f"SudachiPy Total Time: {end - start:.4f} s")