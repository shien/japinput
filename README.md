# japinput

Windows 向け日本語入力システム (IME)。Rust で書かれた SKK 方式の入力メソッドエディタ。

## 特徴

- **ローマ字→ひらがな変換**: 五十音・濁音・半濁音・拗音・促音・撥音に対応
- **ひらがな→カタカナ変換**: Unicode オフセットによる高速変換
- **SKK 辞書検索**: EUC-JP / UTF-8 形式の SKK 辞書に対応
- **変換エンジン**: Direct → Composing → Converting の3状態による変換パイプライン
- **TSF 連携**: Windows Text Services Framework によるシステム IME として動作
- **プラットフォーム分離**: 変換ロジックはプラットフォーム非依存、Windows 固有部分は `#[cfg(windows)]` で分離

## 必要環境

- Rust (Edition 2024)
- Windows 10/11 (IME として使用する場合)

## ビルド

```sh
git clone <repo-url>
cd japinput
cargo build
```

## 使い方

### CLI デモ

ローマ字→かな変換を対話的に試せる CLI デモを実行できる。

```sh
# 辞書なし（ひらがな変換のみ）
cargo run

# 辞書あり（かな→漢字変換）
cargo run -- --dict /path/to/SKK-JISYO.L
```

実行すると標準入力からローマ字を受け付け、変換結果を表示する。

```
ローマ字を入力してください（空行で終了）:
> kanji
ローマ字: kanji
  ひらがな: かんじ
  カタカナ: カンジ
  変換候補: 漢字 / 感じ / 幹事
  確定: 漢字
```

### キー操作

| キー | 動作 |
|------|------|
| `a`-`z` | ローマ字入力 |
| `Space` | 変換開始 / 次の候補 |
| `Enter` | 確定 |
| `Escape` | キャンセル |
| `Backspace` | 1文字削除 |
| `↑` | 前の候補 |
| `↓` | 次の候補 |

### 変換フロー

```
Direct (待機)
  ↓ 文字入力
Composing (ローマ字→かな変換中)
  ↓ Space
Converting (候補選択中)
  ↓ Enter
Direct (確定して待機に戻る)
```

### Windows IME としてインストール

```powershell
# リリースビルド
cargo build --release

# インストール（管理者権限で実行）
.\installer\install.ps1

# アンインストール
.\installer\install.ps1 -Uninstall
```

## 開発

### よく使うコマンド

```sh
cargo test                  # 全テストを実行
cargo test -- --nocapture   # stdout を表示しながらテスト
cargo test <test_name>      # 特定のテストを実行
cargo clippy                # lint チェック
cargo fmt                   # コードフォーマット
cargo fmt -- --check        # フォーマット差分チェック（CI 向け）
```

### プロジェクト構成

```
japinput/
├── src/
│   ├── lib.rs             # クレートルート（モジュール宣言 + DLL エクスポート）
│   ├── main.rs            # CLI デモ
│   ├── romaji.rs          # ローマ字→ひらがな変換
│   ├── katakana.rs        # ひらがな→カタカナ変換
│   ├── input_state.rs     # 逐次入力状態管理
│   ├── dictionary.rs      # SKK 辞書読み込み・検索
│   ├── candidate.rs       # 変換候補リスト管理
│   ├── engine.rs          # 変換エンジン（状態機械）
│   ├── key_mapping.rs     # VirtualKey → EngineCommand 変換
│   ├── guids.rs           # CLSID / Profile GUID
│   ├── text_service.rs    # TSF TextService（Windows）
│   ├── class_factory.rs   # COM ClassFactory（Windows）
│   └── registry.rs        # COM/TSF レジストリ登録（Windows）
├── tests/fixtures/        # テスト用辞書ファイル
├── installer/             # Windows インストーラー
├── plan/                  # 開発計画ドキュメント
└── dict/                  # 辞書ファイル配置先
```

### テスト駆動開発

このプロジェクトは TDD (テスト駆動開発) を採用している。新機能の追加やバグ修正は以下のサイクルで行う:

1. **Red** — 失敗するテストを書く
2. **Green** — テストが通る最小限の実装を書く
3. **Refactor** — コードを整理する

## ライセンス

MIT License (Copyright 2026 shien)
