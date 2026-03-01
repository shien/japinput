# Phase 7.5: Emacs キーバインドの追加（設定で変更可能）

## 目標

Emacs スタイルのキーバインド（Ctrl+キー）を変換エンジンに追加し、
ホームポジションから手を離さずに変換操作できるようにする。
さらに、キーバインドの割り当てを `config.toml` で変更可能にし、
ユーザーが好みに応じてカスタマイズできるようにする。

## 背景

現在のキーバインドは矢印キーや Enter/Escape など、ホームポジションから離れたキーに依存している。
日本語 IME のユーザーには Emacs キーバインドを好む層が多く、SKK 自体も Emacs 発祥のため親和性が高い。
Ctrl+キーの組み合わせで変換・確定・キャンセル等の操作を可能にすることで、入力効率を大幅に向上させる。

一方、Emacs キーバインドに馴染みのないユーザーや、独自の割り当てを好むユーザーもいるため、
Ctrl+キーの割り当てを設定ファイルでカスタマイズできるようにする。

### デフォルトキーバインド

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

### 設定ファイルによるキーバインドのカスタマイズ

`config.toml` に `[keybind]` セクションを追加し、Ctrl+キーの割り当てを変更可能にする。

```toml
# config.toml

[keybind]
# Ctrl+キーの割り当て。値は: commit, cancel, next, prev, backspace, none
# "none" を指定するとそのキーの Ctrl+割り当てを無効化する
ctrl_j = "commit"
ctrl_g = "cancel"
ctrl_n = "next"
ctrl_p = "prev"
ctrl_h = "backspace"
ctrl_m = "commit"
```

**設計方針:**
- `[keybind]` セクションが省略された場合、上記のデフォルト値を使用する
- 個別のキーが省略された場合もデフォルト値を使用する
- `"none"` を指定するとそのキーの Ctrl+割り当てを無効化できる
- 不正な値が指定された場合はパースエラーとする
- 割り当て可能なコマンド: `"commit"`, `"cancel"`, `"next"`, `"prev"`, `"backspace"`, `"convert"`, `"none"`
- 同じコマンドを複数のキーに割り当て可能（例: `ctrl_j` と `ctrl_m` の両方に `"commit"`）

## 前提

- Phase 3 の `EngineCommand` 体系が実装済みであること
- `key_mapping.rs` の `map_key` 関数が VirtualKey → EngineCommand の変換を担当していること
- `Modifiers` 構造体が `ctrl` フラグを持っていること
- Phase 6 の `Config` 構造体と TOML パーサーが設計済みであること

## タスク

### 7.5.1 CtrlKeyConfig 構造体の定義

Ctrl+キーの割り当てを保持する構造体を定義する。

- [ ] `CtrlKeyConfig` 構造体を `key_mapping.rs` に定義する
- [ ] 各フィールドは `Option<EngineCommand>` 型（`None` = 無効）
- [ ] `Default` トレイトでデフォルトの Emacs キーバインドを返す
- [ ] Phase 6 の `Config` に `keybind: CtrlKeyConfig` フィールドを追加する設計を明記

```rust
/// Ctrl+キーの割り当て設定。
/// 各フィールドが None の場合、そのキーは OS に処理を委ねる。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CtrlKeyConfig {
    pub ctrl_a: Option<EngineCommand>,
    pub ctrl_b: Option<EngineCommand>,
    // ... ctrl_c は OS のコピーと競合するため対象外
    pub ctrl_g: Option<EngineCommand>,
    pub ctrl_h: Option<EngineCommand>,
    pub ctrl_j: Option<EngineCommand>,
    pub ctrl_m: Option<EngineCommand>,
    pub ctrl_n: Option<EngineCommand>,
    pub ctrl_p: Option<EngineCommand>,
}
```

**動作確認:**
- `cargo test` でデフォルト値のテスト（7.5.2）がパスすること
- `cargo clippy` がクリーンであること

### 7.5.2 テストの追加

TDD に従い、まずテストを書いてから実装を行う。

#### CtrlKeyConfig のテスト

- [ ] `default_ctrl_config_emacs` — デフォルトが Emacs キーバインドであること
- [ ] `ctrl_config_none_disables_key` — `None` を設定したキーが無効になること

#### map_key のテスト（デフォルト設定）

- [ ] `ctrl_j_commits` — `Ctrl+J` が `EngineCommand::Commit` を返す
- [ ] `ctrl_g_cancels` — `Ctrl+G` が `EngineCommand::Cancel` を返す
- [ ] `ctrl_n_next_candidate` — `Ctrl+N` が `EngineCommand::NextCandidate` を返す
- [ ] `ctrl_p_prev_candidate` — `Ctrl+P` が `EngineCommand::PrevCandidate` を返す
- [ ] `ctrl_h_backspace` — `Ctrl+H` が `EngineCommand::Backspace` を返す
- [ ] `ctrl_m_commits` — `Ctrl+M` が `EngineCommand::Commit` を返す
- [ ] `ctrl_other_returns_none` — `Ctrl+A` 等の未割り当てキーが `None` を返す
- [ ] `ctrl_alt_returns_none` — `Ctrl+Alt+J` 等の複合修飾キーが `None` を返す

#### カスタム設定のテスト

- [ ] `custom_ctrl_j_cancel` — `ctrl_j` を `Cancel` に変更した場合に正しく動作すること
- [ ] `custom_ctrl_n_none` — `ctrl_n` を `None` にした場合、`Ctrl+N` が `None` を返すこと
- [ ] `custom_ctrl_a_convert` — 通常は未割り当ての `ctrl_a` に `Convert` を割り当て可能なこと

#### TOML パースのテスト

- [ ] `parse_keybind_section` — `[keybind]` セクションのパースが正しいこと
- [ ] `parse_keybind_none_value` — `"none"` が `None` にパースされること
- [ ] `parse_keybind_missing_uses_default` — 省略されたキーはデフォルト値を使うこと
- [ ] `parse_keybind_invalid_value_errors` — 不正な値がエラーになること
- [ ] `parse_no_keybind_section_uses_default` — `[keybind]` 省略時はデフォルト全体を使うこと

**動作確認:**
- Red: 新規テストを追加し `cargo test` で失敗することを確認
- Green: 実装後 `cargo test` で全テストがパスすること

### 7.5.3 map_key の設定対応

`map_key` に `CtrlKeyConfig` を受け取る引数を追加し、設定に基づいて Ctrl+キーを処理する。

- [ ] `map_key` のシグネチャを変更: `ctrl_config: &CtrlKeyConfig` 引数を追加
- [ ] Ctrl が押されている場合、`CtrlKeyConfig` から対応するコマンドを検索して返す
- [ ] Ctrl+Alt の場合は引き続き `None` を返す（AltGr 対策）
- [ ] Alt のみの場合も引き続き `None` を返す
- [ ] 既存の呼び出し箇所を更新する（`text_service.rs`, `main.rs`）

```rust
/// 仮想キーコードと修飾キー状態を EngineCommand に変換する。
pub fn map_key(
    vk: u16,
    modifiers: &Modifiers,
    ime_on: bool,
    ctrl_config: &CtrlKeyConfig,
) -> Option<EngineCommand> {
    if !ime_on {
        return None;
    }

    // Alt は常に OS に委ねる
    if modifiers.alt {
        return None;
    }

    // Ctrl+キー: 設定に基づくマッピング
    if modifiers.ctrl {
        return map_ctrl_key(vk, ctrl_config);
    }

    // 通常キー（既存ロジック）
    match vk {
        // ...
    }
}

fn map_ctrl_key(vk: u16, config: &CtrlKeyConfig) -> Option<EngineCommand> {
    match vk {
        VK_A => config.ctrl_a.clone(),
        VK_B => config.ctrl_b.clone(),
        VK_G => config.ctrl_g.clone(),
        VK_H => config.ctrl_h.clone(),
        VK_J => config.ctrl_j.clone(),
        VK_M => config.ctrl_m.clone(),
        VK_N => config.ctrl_n.clone(),
        VK_P => config.ctrl_p.clone(),
        _ => None,
    }
}
```

**動作確認:**
- `cargo test` で全テストがパスすること
- `cargo clippy` と `cargo fmt -- --check` がクリーンであること

### 7.5.4 TOML パース対応

Phase 6 の `Config` 構造体に `keybind: CtrlKeyConfig` フィールドを追加し、
`[keybind]` セクションのパースを実装する。

- [ ] `Config` に `pub keybind: CtrlKeyConfig` フィールドを追加
- [ ] `[keybind]` セクションのパースロジックを実装
- [ ] コマンド名文字列 ↔ `EngineCommand` の変換ヘルパーを用意

```rust
// コマンド名 → EngineCommand の変換
fn parse_command(value: &str) -> Result<Option<EngineCommand>, ConfigError> {
    match value {
        "commit" => Ok(Some(EngineCommand::Commit)),
        "cancel" => Ok(Some(EngineCommand::Cancel)),
        "next" => Ok(Some(EngineCommand::NextCandidate)),
        "prev" => Ok(Some(EngineCommand::PrevCandidate)),
        "backspace" => Ok(Some(EngineCommand::Backspace)),
        "convert" => Ok(Some(EngineCommand::Convert)),
        "none" => Ok(None),
        _ => Err(ConfigError::Parse(
            format!("不正なコマンド名: {value}")
        )),
    }
}
```

**動作確認:**
- `cargo test` で TOML パーステストがパスすること
- 不正な値でエラーが返ることを確認

### 7.5.5 統合テスト（エンジン経由の動作確認）

`key_mapping` のユニットテストに加え、エンジン経由で Emacs キーバインドが正しく動作することを確認する。

#### デフォルト設定での動作

- [ ] Composing 状態で `Ctrl+J` → ひらがな確定 → Direct
- [ ] Converting 状態で `Ctrl+J` → 候補確定 → Direct
- [ ] Composing 状態で `Ctrl+G` → 入力破棄 → Direct
- [ ] Converting 状態で `Ctrl+G` → Composing に戻る
- [ ] Converting 状態で `Ctrl+N` / `Ctrl+P` → 候補移動
- [ ] Composing 状態で `Ctrl+H` → 1文字削除

#### カスタム設定での動作

- [ ] `ctrl_j = "cancel"` に変更した設定で `Ctrl+J` → Cancel が動作すること
- [ ] `ctrl_n = "none"` に変更した設定で `Ctrl+N` → `None`（未処理）になること

**動作確認:**
- `cargo test` で統合テストがパスすること
- 既存の全テストが壊れていないこと

### 7.5.6 text_service.rs の Ctrl キー状態取得対応（Windows）

TSF 連携部分で Ctrl キーの押下状態を正しく `Modifiers` に反映させ、
設定から `CtrlKeyConfig` を読み込んで `map_key` に渡す。

- [ ] `text_service.rs` の `OnKeyDown` で `GetKeyState(VK_CONTROL)` を参照
- [ ] `Modifiers { ctrl: true, .. }` として `map_key` に渡す
- [ ] `Config` から `CtrlKeyConfig` を取得し `map_key` に渡す
- [ ] 手動テスト: Windows 上で Ctrl+J による確定動作を確認
- [ ] 手動テスト: `config.toml` の `[keybind]` を変更して割り当てが変わることを確認

**動作確認:**
- `cargo build --release` がエラーなく完了すること（Windows 環境）
- Windows 上で IME を有効にし、`Ctrl+J` で確定、`Ctrl+G` でキャンセルが動作すること（手動確認）
- `config.toml` の `[keybind]` セクションを編集し、IME 再起動後に変更が反映されること（手動確認）

## 実装順序

| 順序 | タスク | 依存 | 規模 |
|-----|--------|------|------|
| 1 | 7.5.2 テストの追加 (Red) | なし | 小 |
| 2 | 7.5.1 CtrlKeyConfig 構造体の定義 (Green) | 7.5.2 | 小 |
| 3 | 7.5.3 map_key の設定対応 (Green) | 7.5.1, 7.5.2 | 中 |
| 4 | 7.5.4 TOML パース対応 (Green) | 7.5.1 | 中 |
| 5 | 7.5.5 統合テスト | 7.5.3 | 小 |
| 6 | 7.5.6 TSF 連携（Windows のみ） | 7.5.3, 7.5.4 | 中 |

## 完了条件

- デフォルトで Emacs キーバインド (`Ctrl+J/G/N/P/H/M`) が EngineCommand に変換されること
- `config.toml` の `[keybind]` セクションでキーの割り当てを変更できること
- `"none"` を指定することで特定の Ctrl+キーの割り当てを無効化できること
- `[keybind]` セクション省略時はデフォルト（Emacs キーバインド）が適用されること
- 既存のキーバインド（Enter, Escape, 矢印キー等）が引き続き動作すること
- Ctrl+未設定キーは `None` を返し、OS に処理を委ねること
- `cargo test` で全テストがパスすること
- `cargo clippy` と `cargo fmt -- --check` がクリーンであること

## ファイル構成 (変更対象)

```
src/
├── key_mapping.rs     # CtrlKeyConfig 追加、map_key に設定引数追加（主な変更）
├── config.rs          # [keybind] セクションのパース追加（Phase 6 と共同）
├── text_service.rs    # Ctrl キー状態の取得 + CtrlKeyConfig の受け渡し（Windows のみ）
├── engine.rs          # 変更なし（既存の EngineCommand をそのまま使用）
```

## 設定ファイル例

### デフォルト（Emacs キーバインド）

```toml
[keybind]
ctrl_j = "commit"
ctrl_g = "cancel"
ctrl_n = "next"
ctrl_p = "prev"
ctrl_h = "backspace"
ctrl_m = "commit"
```

### Vim 風にカスタマイズした例

```toml
[keybind]
# Ctrl+J/K で候補移動、Ctrl+L で確定
ctrl_j = "next"
ctrl_k = "prev"          # ※ ctrl_k は将来拡張で対応
ctrl_l = "commit"        # ※ ctrl_l は将来拡張で対応
ctrl_g = "cancel"
ctrl_h = "backspace"
ctrl_m = "commit"
ctrl_n = "none"          # Ctrl+N を無効化（OS に委ねる）
ctrl_p = "none"          # Ctrl+P を無効化（OS に委ねる）
```

### Emacs キーバインドを完全に無効化した例

```toml
[keybind]
ctrl_j = "none"
ctrl_g = "none"
ctrl_n = "none"
ctrl_p = "none"
ctrl_h = "none"
ctrl_m = "none"
```

## 注意事項

- **EngineCommand の追加は不要。** 既存の `Commit`, `Cancel`, `NextCandidate`, `PrevCandidate`, `Backspace`, `Convert` をそのまま使う。新しいキーバインドは「入力経路の追加」であり、エンジンの変更は不要。
- **Alt+キーは対象外。** `modifiers.alt == true` の場合は引き続き `None` を返す。Alt キーの組み合わせは OS のメニューショートカットと競合するため。
- **Ctrl+Alt の組み合わせも対象外。** 一部の言語レイアウトでは `Ctrl+Alt` が `AltGr` として使われるため、`ctrl && alt` が同時に `true` の場合は `None` を返す。
- **Ctrl+C は対象外。** OS のコピー操作と競合するため、`ctrl_c` フィールドは設けない。
- **`InsertChar` は割り当て不可。** Ctrl+キーに文字入力を割り当てることは意味がないため、設定値に `"insert_char"` は含めない。
- **将来の拡張性:** カーソル移動（`Ctrl+A/E/F/B`）は `EngineCommand` の拡張が必要なため、Composing 状態でのカーソル移動機能と合わせて別フェーズで実装する。その際、`CtrlKeyConfig` に `ctrl_a`, `ctrl_b`, `ctrl_e`, `ctrl_f` フィールドは既に用意されており、拡張が容易。
- **Phase 6 との連携:** `config.rs` の `Config` 構造体に `keybind: CtrlKeyConfig` を追加する。Phase 6 未実装の場合は `CtrlKeyConfig::default()` をハードコードして使用し、Phase 6 実装時に `Config` から取得するよう切り替える。
