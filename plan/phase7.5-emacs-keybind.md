# Phase 7.5: Emacs キーバインドの追加

## 目標

Emacs スタイルのキーバインド（Ctrl+キー）を変換エンジンに追加し、
ホームポジションから手を離さずに変換操作できるようにする。

## 背景

現在のキーバインドは矢印キーや Enter/Escape など、ホームポジションから離れたキーに依存している。
日本語 IME のユーザーには Emacs キーバインドを好む層が多く、SKK 自体も Emacs 発祥のため親和性が高い。
Ctrl+キーの組み合わせで変換・確定・キャンセル等の操作を可能にすることで、入力効率を大幅に向上させる。

### 対象キーバインド

| キーバインド | 動作 | 対応する EngineCommand | 既存キー |
|-------------|------|----------------------|---------|
| `Ctrl+J` | 確定 (Commit) | `Commit` | `Enter` |
| `Ctrl+G` | キャンセル (Cancel) | `Cancel` | `Escape` |
| `Ctrl+N` | 次の候補 | `NextCandidate` | `↓` |
| `Ctrl+P` | 前の候補 | `PrevCandidate` | `↑` |
| `Ctrl+H` | 1文字削除 (Backspace) | `Backspace` | `Backspace` |
| `Ctrl+M` | 確定 (Commit) ※ Enter と同等 | `Commit` | `Enter` |
| `Ctrl+A` | 行頭移動 ※ 将来拡張用 | (Phase 7.5 ではスコープ外) | — |
| `Ctrl+E` | 行末移動 ※ 将来拡張用 | (Phase 7.5 ではスコープ外) | — |

**Phase 7.5 のスコープ:** `Ctrl+J`, `Ctrl+G`, `Ctrl+N`, `Ctrl+P`, `Ctrl+H`, `Ctrl+M` の6つ。
カーソル移動系 (`Ctrl+A`, `Ctrl+E`, `Ctrl+F`, `Ctrl+B`) は Composing 状態でのカーソル移動機能と
合わせて将来フェーズで実装する。

## 前提

- Phase 3 の `EngineCommand` 体系が実装済みであること
- `key_mapping.rs` の `map_key` 関数が VirtualKey → EngineCommand の変換を担当していること
- `Modifiers` 構造体が `ctrl` フラグを持っていること

## タスク

### 7.5.1 key_mapping.rs の Ctrl+キー対応

現在 `map_key` は `modifiers.ctrl == true` の場合に一律 `None` を返している。
Ctrl が押されている場合でも、特定のキーについては EngineCommand を返すように変更する。

- [ ] `map_key` の Ctrl 判定ロジックを変更: Ctrl+特定キーの場合は EngineCommand を返す
- [ ] `Ctrl+J` → `EngineCommand::Commit`
- [ ] `Ctrl+G` → `EngineCommand::Cancel`
- [ ] `Ctrl+N` → `EngineCommand::NextCandidate`
- [ ] `Ctrl+P` → `EngineCommand::PrevCandidate`
- [ ] `Ctrl+H` → `EngineCommand::Backspace`
- [ ] `Ctrl+M` → `EngineCommand::Commit`
- [ ] Ctrl+上記以外のキーは引き続き `None` を返す

**動作確認:**
- Red: `cargo test` で新しいテスト（下記 7.5.2）が失敗すること
- Green: `cargo test` で全テストがパスすること
- `cargo clippy` と `cargo fmt -- --check` がクリーンであること

### 7.5.2 テストの追加

TDD に従い、まずテストを書いてから 7.5.1 の実装を行う。

- [ ] `ctrl_j_commits` — `Ctrl+J` が `EngineCommand::Commit` を返す
- [ ] `ctrl_g_cancels` — `Ctrl+G` が `EngineCommand::Cancel` を返す
- [ ] `ctrl_n_next_candidate` — `Ctrl+N` が `EngineCommand::NextCandidate` を返す
- [ ] `ctrl_p_prev_candidate` — `Ctrl+P` が `EngineCommand::PrevCandidate` を返す
- [ ] `ctrl_h_backspace` — `Ctrl+H` が `EngineCommand::Backspace` を返す
- [ ] `ctrl_m_commits` — `Ctrl+M` が `EngineCommand::Commit` を返す
- [ ] `ctrl_other_returns_none` — `Ctrl+A` 等の未定義キーが `None` を返す
- [ ] `ctrl_alt_returns_none` — `Ctrl+Alt+J` 等の複合修飾キーが `None` を返す
- [ ] 既存テスト `ctrl_key_returns_none` の修正（`Ctrl+A` は引き続き `None` だが、テスト名と内容の確認）

**動作確認:**
- Red: 新規テストを追加し `cargo test` で失敗することを確認
- Green: 7.5.1 を実装後 `cargo test` で全テストがパスすること

### 7.5.3 map_key のリファクタリング

Ctrl+キーのマッピングが増えた場合に備えて、コードを整理する。

- [ ] Ctrl+キーのマッチ処理を通常キーのマッチ処理と分離する
  - 案A: `map_key` 内で `if modifiers.ctrl { return map_ctrl_key(vk); }` のように分離
  - 案B: `match` 内にガード条件で統合
- [ ] 最もシンプルで読みやすい形を選択する
- [ ] 既存テストが壊れていないことを確認する

**動作確認:**
- `cargo test` で全テストがパスすること
- `cargo clippy` でワーニングがないこと
- `cargo fmt -- --check` で差分がないこと

### 7.5.4 統合テスト（エンジン経由の動作確認）

`key_mapping` のユニットテストに加え、エンジン経由で Emacs キーバインドが正しく動作することを確認する。

- [ ] Composing 状態で `Ctrl+J` → ひらがな確定 → Direct
- [ ] Converting 状態で `Ctrl+J` → 候補確定 → Direct
- [ ] Composing 状態で `Ctrl+G` → 入力破棄 → Direct
- [ ] Converting 状態で `Ctrl+G` → Composing に戻る
- [ ] Converting 状態で `Ctrl+N` / `Ctrl+P` → 候補移動
- [ ] Composing 状態で `Ctrl+H` → 1文字削除

**動作確認:**
- `cargo test` で統合テストがパスすること
- 既存の全テストが壊れていないこと

### 7.5.5 text_service.rs の Ctrl キー状態取得対応（Windows）

TSF 連携部分で Ctrl キーの押下状態を正しく `Modifiers` に反映させる。

- [ ] `text_service.rs` の `OnKeyDown` で `GetKeyState(VK_CONTROL)` を参照
- [ ] `Modifiers { ctrl: true, .. }` として `map_key` に渡す
- [ ] 手動テスト: Windows 上で Ctrl+J による確定動作を確認

**動作確認:**
- `cargo build --release` がエラーなく完了すること（Windows 環境）
- Windows 上で IME を有効にし、`Ctrl+J` で確定、`Ctrl+G` でキャンセルが動作すること（手動確認）

## 実装順序

| 順序 | タスク | 依存 | 規模 |
|-----|--------|------|------|
| 1 | 7.5.2 テストの追加 (Red) | なし | 小 |
| 2 | 7.5.1 Ctrl+キー対応の実装 (Green) | 7.5.2 | 小 |
| 3 | 7.5.3 リファクタリング (Refactor) | 7.5.1 | 小 |
| 4 | 7.5.4 統合テスト | 7.5.1 | 小 |
| 5 | 7.5.5 TSF 連携（Windows のみ） | 7.5.1 | 中 |

## 完了条件

- Emacs キーバインド (`Ctrl+J/G/N/P/H/M`) が EngineCommand に変換されること
- 既存のキーバインド（Enter, Escape, 矢印キー等）が引き続き動作すること
- Ctrl+未定義キーは `None` を返し、OS に処理を委ねること
- `cargo test` で全テストがパスすること
- `cargo clippy` と `cargo fmt -- --check` がクリーンであること

## ファイル構成 (変更対象)

```
src/
├── key_mapping.rs     # Ctrl+キーのマッピング追加（主な変更）
├── text_service.rs    # Ctrl キー状態の取得（Windows のみ）
├── engine.rs          # 変更なし（既存の EngineCommand をそのまま使用）
```

## 注意事項

- **EngineCommand の追加は不要。** 既存の `Commit`, `Cancel`, `NextCandidate`, `PrevCandidate`, `Backspace` をそのまま使う。新しいキーバインドは「入力経路の追加」であり、エンジンの変更は不要。
- **Alt+キーは対象外。** `modifiers.alt == true` の場合は引き続き `None` を返す。Alt キーの組み合わせは OS のメニューショートカットと競合するため。
- **Ctrl+Alt の組み合わせも対象外。** 一部の言語レイアウトでは `Ctrl+Alt` が `AltGr` として使われるため、`ctrl && alt` が同時に `true` の場合は `None` を返す。
- **将来の拡張性:** カーソル移動（`Ctrl+A/E/F/B`）は `EngineCommand` の拡張が必要なため、Composing 状態でのカーソル移動機能と合わせて別フェーズで実装する。
