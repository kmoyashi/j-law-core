# AGENTS.md — J-Law-Core AIエージェント向け指示書

このファイルはClaude CodeなどのAIコーディングエージェントが読む指示書です。
コードを生成・編集する前に必ずこのファイルの内容を確認してください。

---

## プロジェクト概要（エージェント向け）

- **目的**: 日本の法令・告示・省令が定める各種計算を、法的正確性を保証して実装するライブラリ
- **アーキテクチャ**: ドメイン単位で法令を追加できる設計
- **Rust エディション**: 2021
- **Cargo workspace メンバー**:

| クレート | 役割 | 主な依存 |
|---|---|---|
| `j-law-core` | 共通基盤（型・エラー・計算ロジック） | `thiserror = "1"` |
| `j-law-registry` | 法令パラメータ管理（JSON） | `j-law-core`, `serde/serde_json = "1"` |
| `j-law-python` | Python バインディング | `pyo3 = "0.21"` |
| `j-law-wasm` | WASM/JS バインディング | `wasm-bindgen = "0.2"`, `js-sys = "0.3"` |
| `j-law-ruby/ext/j_law_core` | Ruby バインディング | `magnus = "0.7"` |
| `j-law-cgo` | C FFI（Go 向け staticlib） | `j-law-core`, `j-law-registry` |

- **非 workspace メンバー**（Go）: `crates/j-law-go/`（`go.mod` で管理、CGo 経由で `j-law-cgo` にリンク）

### 実装済みドメイン

| ドメイン | 法的根拠 | ディレクトリ |
|---|---|---|
| 不動産（`real_estate`） | 宅建業法 第46条 / 国交省告示 | `domains/real_estate/` |
| 所得税（`income_tax`） | 所得税法 第89条 / 復興財源確保法 第13条 | `domains/income_tax/` |

---

## セッション開始時の必須手順

**セッションをまたいで記憶は引き継がれません。毎回以下を実行してください。**

```sh
cat Cargo.toml
ls crates/j-law-core/src/domains/
cargo test --all 2>&1 | tail -20
```

現状のドメイン構成とテスト状態を確認してから作業を開始すること。

---

## コーディングルール（違反すると clippy が警告します）

### ルール1: 金額・数値計算に `f64` / `f32` を使用禁止

法令計算は端数処理の順序が結果を左右します。浮動小数点は使用禁止です。
`clippy.toml` で `disallowed-types` に設定されており、違反すると `cargo clippy` が警告します。

```rust
// NG — 法令計算で絶対に書かない
let fee: f64 = price as f64 * 0.05;

// OK — IntermediateAmount + Rate の整数演算を使う
let fee = IntermediateAmount::from_exact(price)
    .apply_rate(&Rate { numer: 5, denom: 100 }, RoundingStrategy::Floor);
```

確認コマンド: `make clippy`（または `make ci` で一括確認）

### ルール2: `panic!` / `unwrap()` / `expect()` 使用禁止（コア層）

`crates/j-law-core/src/` 内では `panic!` / `unwrap()` / `expect()` を使わず、すべて `Result<T, E>` で返すこと。
`clippy.toml` で `disallowed-methods` と `disallowed-macros` に設定されており、違反すると `cargo clippy` が警告します。

Registry層（`crates/j-law-registry/src/`）の起動時バリデーションのみ `panic!` を許容します。

確認コマンド: `make clippy`（または `make ci` で一括確認）

### ルール3: Registry JSONの数値は整数のみ

`crates/j-law-registry/data/` 内のJSONに小数点を含む数値を書いてはいけません。

```json
// NG
{ "rate": 0.05 }

// OK
{ "rate": { "numer": 5, "denom": 100 } }
```

### ルール4: `pub` な型・関数には根拠条文を docコメントで明記

```rust
/// 標準媒介報酬の3段階ティア計算を実行する。
///
/// # 法的根拠
/// 宅地建物取引業法 第46条第1項
/// 国土交通省告示（2024年7月1日施行）
pub fn calculate_brokerage_fee(...) -> Result<CalculationResult, JLawError> {
```

### ルール5: TDD（テストファースト）

実装前にテストを書くこと。`make ci` がグリーンになることが各タスクの完了基準です。
テストを削除・`#[ignore]` で誤魔化してはいけません。

---

## 型システム（必ず従うこと）

### 金額型（全ドメイン共通）

| 型 | 用途 | 備考 |
|---|---|---|
| `FinalAmount(u64)` | 最終的な円単位の金額 | 税込/税抜/税額など最終値 |
| `IntermediateAmount { whole, numer, denom }` | 計算途中の分数表現 | 端数処理前の中間値 |

`FinalAmount` と `IntermediateAmount` は用途を混在させないこと。
どのドメインでもこの2型を使い、ドメイン固有の金額型を新設してはいけません。

### 日付型

| 型 | 用途 | 備考 |
|---|---|---|
| `LegalDate { year: u16, month: u8, day: u8 }` | 法令の施行日・基準日 | `(u16, u8, u8)` 匿名タプルの代わりに使う |

- `LegalDate::new(year, month, day)` で生成する
- `to_date_str()` で `"YYYY-MM-DD"` 形式の文字列に変換できる
- Registry loader（`load_*_params`）はすべて `LegalDate` を引数として受け取る
- 匿名タプル `(u16, u8, u8)` をそのまま使わないこと（フィールドの意味が不明確になる）

### エラー型階層（全ドメイン共通）

```
JLawError
├── Registry(RegistryError)       — 法令パラメータデータの不整合
│   ├── PeriodOverlap { domain, from, until }
│   ├── PeriodGap { domain, end, next_start }
│   ├── FloatProhibited { path }
│   └── ZeroDenominator { path }
├── Input(InputError)             — ユーザー入力の不正
│   ├── NegativeAmount { value }
│   ├── DateOutOfRange { date }
│   └── ConflictingFlags { flag_a, flag_b }
└── Calculation(CalculationError) — 計算時の異常
    ├── Overflow { step }
    └── PolicyNotApplicable { reason }
```

- 新しいエラーを追加するときは既存バリアントを**削除・改名しない**こと（Breaking Change になります）
- ドメイン固有のエラーは上記4種のいずれかの傘下に追加してください

### ドメイン固有フラグ

各ドメインは独自の `<Domain>Flag` enumを持ちます。フラグの事実認定（「この取引は○○に該当するか」の判断）はライブラリの責任範囲外です。事実認定を要するフラグのdocコメントには必ず以下の警告を付けてください:

```rust
/// WARNING: このフラグの事実認定はライブラリの責任範囲外です。
/// 呼び出し元が正しく判断した上で指定してください。
```

---

## ドメインの追加ルール

新しい法令ドメインを追加する場合は以下の構成に従ってください。

### ディレクトリ構成

```
crates/j-law-core/src/domains/<domain_name>/
├── mod.rs          # pub use でサブモジュールを再エクスポート
├── context.rs      # 計算コンテキスト（入力値 + フラグ + ポリシー）
├── params.rs       # 法令パラメータ型（Registry から読み込む）
├── policy.rs       # <Domain>Policy trait + 標準実装
└── calculator.rs   # calculate_xxx() 関数

crates/j-law-registry/data/<domain_name>/
└── <law_name>.json  # 法令パラメータ（整数のみ・period管理）

crates/j-law-core/tests/<domain_name>.rs           ← エントリファイル
crates/j-law-core/tests/<domain_name>/
├── <official_source>_examples.rs  # 公式計算例テスト
└── edge_cases.rs                  # 境界値テスト
```

### 必須実装項目

各ドメインは以下を必ず実装してください:

1. **Contextオブジェクト**: 入力値・target_date・フラグのセット
2. **Params型**: Registry JSON から読み込むパラメータ構造体
3. **Policy trait**: 端数処理戦略・特例適用判定などのカスタマイズポイント
4. **標準Policy実装**: 省庁の標準解釈に従ったデフォルト実装
5. **calculate関数**: `Result<CalculationResult, JLawError>` を返す主計算関数
6. **CalculationResult**: `breakdown` を含む結果型
7. **公式計算例テスト**: 省庁や公的機関が公表している計算例を全件テスト化

### 新ドメイン追加時の言語バインディング拡張

ドメイン追加時は、Rust コアの実装に加えて全言語バインディングに関数を追加してください:

| 言語 | ファイル | 方式 |
|---|---|---|
| Python | `crates/j-law-python/src/lib.rs` | PyO3 `#[pyfunction]` + サブモジュール登録（`sys.modules` 登録必須） |
| WASM/JS | `crates/j-law-wasm/src/lib.rs` | `#[wasm_bindgen]` 関数 |
| Ruby | `crates/j-law-ruby/ext/j_law_core/src/lib.rs` | Magnus `define_method` |
| C/Go | `crates/j-law-cgo/src/lib.rs` + `j_law_cgo.h` + `crates/j-law-go/j_law_core.go` | `extern "C"` FFI |

テストフィクスチャは `tests/fixtures/<domain_name>.json` に共通 JSON を作成し、全言語のテストで読み込むこと。

### Registry JSONの必須フィールド

```json
{
  "domain": "<domain_name>",
  "history": [
    {
      "effective_from": "YYYY-MM-DD",
      "effective_until": "YYYY-MM-DD",
      "status": "active",
      "citation": {
        "law_name": "法令名",
        "article": 46,
        "paragraph": 1
      },
      "params": {
        // 全数値は整数または { "numer": N, "denom": N } 形式
      }
    }
  ]
}
```

---

## テスト設計

### テスト階層

| 階層 | 配置場所 | 対象 | 備考 |
|---|---|---|---|
| Rust ユニットテスト | 各 `src/*.rs` 内 `#[cfg(test)]` | 関数・型単位 | `cargo test` で実行 |
| Rust 統合テスト | `crates/j-law-core/tests/` | ドメイン全体 | Registry 読み込み含む |
| 言語バインディングテスト | 各 `crates/j-law-*/tests/` | FFI 経由の動作 | 共通 JSON フィクスチャ使用 |

### 共通テストフィクスチャ

`tests/fixtures/` 配下に JSON 形式でテストケースを定義し、全言語のテストで共有する:

```
tests/fixtures/
├── real_estate.json   # 不動産ドメインのテストケース
└── income_tax.json    # 所得税ドメインのテストケース
```

**フィクスチャ JSON の構造**:

```json
{
  "<calculation_name>": [
    {
      "id": "テストケースID",
      "description": "テストの説明",
      "input": { ... },
      "expected": { ... }
    }
  ]
}
```

- **JSON に入れるもの**: 入力→期待出力のデータ駆動テスト
- **JSON に入れないもの**: エラーテスト（例外型が言語固有）、repr/inspect テスト、breakdown 構造テスト

### 各言語のテストパターン

| 言語 | フレームワーク | 手法 |
|---|---|---|
| Python | pytest | `@pytest.mark.parametrize` でフィクスチャをループ |
| JS/Node | `node:test` | `for (const c of fixtures)` でループ |
| Ruby | minitest | `define_method("test_#{c['id']}")` で動的テスト生成 |
| Go | `testing` | テーブル駆動テスト + `t.Run(tc.ID, ...)` |

---

## Docker テスト環境

全言語のテストを Docker コンテナ上で実行できます。

### ファイル構成

- `Dockerfile` — マルチステージビルド（Rust 1.85 ベース）
- `docker-compose.yml` — 各言語テストサービス + `test-all` 統合サービス
- `.dockerignore` — ビルドコンテキスト除外設定

### テストサービス

| サービス | 内容 |
|---|---|
| `test-rust` | `cargo test -p j-law-core -p j-law-registry` |
| `test-python` | `maturin build` → `pytest` |
| `test-wasm` | `wasm-pack build` → `node --test` |
| `test-ruby` | `bundle exec rake compile` → `rake test` |
| `test-go` | `cargo build -p j-law-cgo` → `go test` |
| `test-all` | 上記5つの完了を待って成功判定 |

### 実行コマンド

```sh
# 全言語テスト一括実行
docker compose up test-all --build

# 個別言語テスト
docker compose up test-<lang> --build
```

---

## 計算仕様 — 不動産ドメイン（参照実装）

### 3段階ティア計算

| 売買価格の範囲 | 率 |
|---|---|
| 200万円以下の部分 | 5/100 |
| 200万円超〜400万円以下の部分 | 4/100 |
| 400万円超の部分 | 3/100 |

**端数処理の順序（厳守）**:
1. 各ティアの金額を個別に `RoundingStrategy::Floor` で切り捨て
2. 切り捨て済みの整数を合計
3. 合計に消費税 10/100 を乗じ `Floor` で切り捨て

各ティアを合計してから一括で端数処理するのは**誤り**です。

### 低廉な空き家特例（2024年7月1日〜）

- 適用条件: `price <= 8_000_000` かつ `flags.contains(RealEstateFlag::IsLowCostVacantHouse)`
- 特例は**最低保証額（floor）**であり、上限キャップ（ceiling）ではありません。通常計算が `330_000` を下回る場合に `330_000` まで**引き上げ**ます。
  - `subtotal = subtotal.max(special.fee_ceiling_exclusive_tax)` — これが正しい実装です
  - `if subtotal > ceiling { subtotal = ceiling }` — **誤りです**（通常計算 > 特例上限の場合に値を下げてしまう）

---

## 計算仕様 — 所得税ドメイン

### 速算表方式（所得税法 第89条）

算出税額 = 課税所得金額 × 税率 − 控除額

| 課税所得金額 | 税率 | 控除額 |
|---|---|---|
| 〜1,950,000円 | 5/100 | 0円 |
| 1,950,001〜3,300,000円 | 10/100 | 97,500円 |
| 3,300,001〜6,950,000円 | 20/100 | 427,500円 |
| 6,950,001〜9,000,000円 | 23/100 | 636,000円 |
| 9,000,001〜18,000,000円 | 33/100 | 1,536,000円 |
| 18,000,001〜40,000,000円 | 40/100 | 2,796,000円 |
| 40,000,001円〜 | 45/100 | 4,796,000円 |

### 復興特別所得税（復興財源確保法 第13条）

- 適用期間: 2013年〜2037年
- 税率: 基準所得税額 × 21/1000
- 申告納税額 = (基準所得税額 + 復興特別所得税額) を100円未満切り捨て

---

## 実装上の注意

### 循環依存の回避

`j-law-registry` は `j-law-core` に依存しています。そのため `j-law-core` が `j-law-registry` に依存すると循環依存になりコンパイルできません。

**解決策**: ドメインのパラメータ型（例: `BrokerageFeeParams`）は `j-law-core` 側で定義し、`j-law-registry` はそれをインポートして使います。`j-law-registry` は `j-law-core` の `[dev-dependencies]` にのみ記載し、`[dependencies]` には載せません。

```toml
# crates/j-law-core/Cargo.toml
[dependencies]
thiserror = "1"

[dev-dependencies]
j-law-registry = { path = "../j-law-registry" }  # テスト時のみ
```

```toml
# crates/j-law-registry/Cargo.toml
[dependencies]
j-law-core = { path = "../j-law-core" }  # パラメータ型を参照するため
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

同様に、`j-law-core/src/error.rs` に `serde_json::Error` を含む `ParseError` バリアントを追加してはいけません（`serde_json` は `j-law-core` の依存に含まれていないため）。JSONパースエラーは `j-law-registry` 内で処理し、`RegistryError` の別バリアントとして返してください。

### Integration testのファイル構成

Rustの統合テスト（`tests/`）はルート直下のファイルしか自動検出されません。テストをサブディレクトリで管理する場合は `#[path]` attribute を使ったエントリファイルが必要です。

```
crates/j-law-core/tests/
├── real_estate.rs         ← エントリファイル（必須）
├── real_estate/
│   ├── mlitt_examples.rs
│   └── edge_cases.rs
├── income_tax.rs          ← エントリファイル（必須）
└── income_tax/
    ├── calculation_examples.rs
    └── edge_cases.rs
```

### PyO3 サブモジュール登録

PyO3 の `add_submodule` だけでは `from j_law_core.real_estate import ...` が動作しません。`sys.modules` への明示的な登録が必要です:

```rust
py.import_bound("sys")?
    .getattr("modules")?
    .set_item("j_law_core.real_estate", &m)?;
```

### Go CGo リンクフラグ

`crates/j-law-go/j_law_core.go` でプラットフォーム別のリンクフラグを定義しています:

```go
// #cgo darwin LDFLAGS: ... -framework Security -framework CoreFoundation
// #cgo linux  LDFLAGS: ... -ldl -lpthread -lm
```

Docker コンテナ（Linux）では `linux` フラグが使用されます。

---

## タスク指示フォーマット

人間からのタスク指示は以下のフォーマットで渡されます。このフォーマット外の指示を受けた場合も、以下の項目が揃っているか確認してから着手してください。

```
## タスク: [タスクID] [タスク名]

### やること
- [具体的な実装内容 1]
- [具体的な実装内容 2]

### 作成/編集するファイル
- crates/j-law-core/src/domains/xxx/calculator.rs（新規作成）

### 完了条件
- `cargo test domains::xxx::` が全グリーン
- floatを一切使用していないこと

### 制約
- f64/f32の使用禁止
- panicを使用禁止（Result型で返す）
```

---

## タスク完了時のセルフレビュー

各タスクの実装が終わったら、コードを提出する前に **必ず `make ci` を実行**して確認してください。

```sh
# CIチェック一式（フォーマット・リント・テスト）を一括実行
make ci
```

`make ci` が通ったら、追加で以下も確認してください。

```sh
# Registry JSONの数値チェック（小数点禁止）
grep -rn '[0-9]\.[0-9]' crates/j-law-registry/data/ && echo "NG: 小数点あり" || echo "OK"

# 全言語テスト（Docker、binding変更時のみ）
make docker-test
```

**`make ci` がグリーンになってから完了を報告してください。**

---

## エラー対応フロー

| 状況 | 取るべき行動 | 取ってはいけない行動 |
|---|---|---|
| コンパイルエラーが出た | エラー全文を確認し、根本原因を特定してから修正する | 同じ修正を繰り返す |
| テストが失敗した | 期待値と実際の値のどちらが正しいかを公式資料で照合する | テストを削除・`#[ignore]` にする |
| 所有権エラーが収拾つかない | 一旦 `Arc<T>` や `.clone()` で回避し、動作を確認してから最適化する | エラーメッセージを読まずに修正を試みる |
| 計算結果が1円ずれる | `breakdown` の各ステップの中間値を出力し、どのステップで差が出るか特定する | 期待値を書き換えて合わせる |
| 設計と実装が乖離している | このAGENTS.mdの該当セクションを引用して、仕様に合わせて書き直す | 「動いているから問題ない」とみなして放置する |

---

## 実装フェーズとタスク依存関係

各ドメインの実装は以下の順序に従います。

```
Step 0（環境構築・人間が実施）
  └─ Step 1（共通型定義: L1-Types）         ← 全ドメイン共通基盤（実装済み）
  └─ Step 2（共通エラー型: L1-Error）       ← Step 1 と並行可（実装済み）
       └─ Step 3（Registryデータ作成: L2-Data）    ← ドメインごとに実施
            └─ Step 4（Registry Loader: L2-Loader）← ドメインごとに実施
                 └─ Step 5（計算ロジック: L1-Calculator）← ドメインごとに実施
                      └─ Step 6（公式計算例テスト: Integration）← ドメインごとに実施
                           └─ Step 7（言語バインディング: L3-Bindings）← 全言語一括
                                └─ Step 8（テストフィクスチャ: L3-Fixtures）← JSON + 全言語テスト
```

Step 1・2（共通基盤）は実装済みです。
新しいドメインを追加する場合はStep 3〜8を繰り返します。
