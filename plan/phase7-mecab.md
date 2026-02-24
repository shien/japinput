# Phase 7: MeCab/形態素解析による高機能変換

## 目標

SKK 方式（ユーザーが変換範囲を指定）から、連文節変換（自動で文節を区切る）へ進化させる。
形態素解析エンジンを導入し、より自然な日本語変換を実現する。

## 背景

SKK 方式は実装が簡単だが、ユーザーが変換範囲を明示的に指定する必要がある。
MeCab 等の形態素解析を使えば、MS-IME や Google 日本語入力のように
文章全体を入力して自動で文節区切り・変換ができるようになる。

## 候補ライブラリ

| ライブラリ | 言語 | 特徴 |
|-----------|------|------|
| **lindera** | Pure Rust | FFI 不要。Rust エコシステムに自然に統合 |
| **vibrato** | Pure Rust | 高速。Viterbi アルゴリズムベース |
| **mecab-rs** | Rust (FFI) | MeCab の Rust バインディング。C ライブラリへの依存あり |

## 前提

Phase 3 で `ConversionBackend` trait を定義済みであること:

```rust
trait ConversionBackend {
    fn lookup(&self, reading: &str) -> Vec<Candidate>;
}
```

## タスク

### 7.1 形態素解析ライブラリの選定・導入

- [ ] lindera / vibrato / mecab-rs を比較検証
  - ビルドの容易さ（Pure Rust が望ましい）
  - 辞書サイズとライセンス
  - 変換精度・速度
- [ ] 選定したライブラリを `Cargo.toml` に追加
- [ ] 基本的な形態素解析の動作確認テスト

### 7.2 連文節変換バックエンドの実装

- [ ] `MorphBackend` 構造体 (`ConversionBackend` trait を実装)
- [ ] 入力ひらがな → 形態素解析 → 文節ごとの候補生成
- [ ] 文節区切り位置の調整（Shift+←/→ で文節伸縮）
- [ ] テスト: 基本的な連文節変換、文節区切りの検証

### 7.3 バックエンド切り替え

- [ ] 設定で SKK 方式 / 連文節変換を切り替え可能にする
- [ ] エンジンの `ConversionBackend` を動的に差し替え
- [ ] テスト: 両バックエンドの切り替え

### 7.4 予測変換（任意）

- [ ] 入力途中の文字列から変換候補を予測して提示
- [ ] 変換履歴・頻度に基づく候補順のソート
- [ ] テスト: 予測候補の生成

### 7.5 ベンチマーク

- [ ] SKK 方式と形態素解析方式の速度比較
- [ ] 大量テキストでの変換精度評価
- [ ] メモリ使用量の計測

## 完了条件

- 連文節変換が動作する（文章を入力→自動文節区切り→変換）
- SKK 方式との切り替えが可能
- 変換精度・速度が実用レベル
- `cargo test` で全テストがパスする

## ファイル構成 (予定)

```
src/
├── engine.rs            # ConversionBackend trait (Phase 3 で定義済み)
├── backend/
│   ├── mod.rs
│   ├── skk.rs           # SKK 辞書バックエンド (Phase 2-3 で実装済み)
│   └── morph.rs         # 形態素解析バックエンド (新規)
├── ...
```

## 依存 crate (追加予定)

- `lindera` (推奨) または `vibrato` または `mecab` (FFI)

## 注意事項

- 形態素解析用の辞書 (IPAdic 等) が数十MB になるため、配布サイズに注意
- lindera は IPADIC, UniDic, ko-dic, cc-cedict に対応
- ライセンス: IPAdic は IPA ライセンス（再配布可）
