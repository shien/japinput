# Phase 3: 変換エンジンの統合

## 目標

ローマ字入力から漢字候補の表示まで、一連の変換パイプラインを統合する。
SKK 方式の変換操作（変換開始・候補選択・確定）を実装する。

## 背景

SKK 方式の入力フロー:
1. ユーザーがローマ字を入力 → リアルタイムでひらがなに変換（Phase 1）
2. 変換キー（Space）を押す → 辞書から候補を検索（Phase 2）
3. 候補を選択 → 確定してテキストに挿入

## タスク

### 3.1 変換セッション管理

- [ ] `src/engine.rs` に `ConversionEngine` 構造体を実装
- [ ] 状態遷移: `Direct` → `Composing` → `Converting` → `Direct`
  - `Direct`: 直接入力（変換なし）
  - `Composing`: ローマ字→かな変換中（未確定文字列あり）
  - `Converting`: 候補選択中
- [ ] テスト: 各状態遷移

### 3.2 候補管理

- [ ] `CandidateList`: 現在の候補リストと選択インデックス
- [ ] `next()` / `prev()`: 候補を前後に移動
- [ ] `select()`: 現在の候補を確定
- [ ] テスト: 候補のナビゲーション、境界値

### 3.3 コマンド体系

- [ ] `EngineCommand` enum:
  - `InsertChar(char)`: 文字入力
  - `Convert`: 変換開始 (Space)
  - `NextCandidate` / `PrevCandidate`: 候補移動
  - `Commit`: 確定 (Enter)
  - `Cancel`: キャンセル (Escape)
  - `Backspace`: 1文字削除
- [ ] `engine.process(command) -> EngineOutput`
- [ ] `EngineOutput`: 表示用の情報（確定文字列、未確定文字列、候補リスト）
- [ ] テスト: 各コマンドの動作

### 3.4 統合テスト

- [ ] 「ローマ字入力→変換→候補選択→確定」の一連のフローをテスト
- [ ] 例: "kanji" → Space → 「漢字」選択 → Enter

### 3.5 CLI デモの更新

- [ ] CLI で変換エンジンを使った対話的デモ
- [ ] キー入力に対応した変換操作を確認

## 完了条件

- ローマ字入力→辞書検索→候補選択→確定の一連の流れが動作する
- 状態遷移が正しく機能する
- `cargo test` で全テストがパスする

## ファイル構成 (予定)

```
src/
├── engine.rs        # 変換エンジン（状態管理・コマンド処理）
├── candidate.rs     # 候補リスト管理
├── ...
```
