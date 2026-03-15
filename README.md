# J-Law-Core

日本法令に準拠した計算・検証ライブラリ（Rust / Python / JavaScript / Ruby / Go）

> [!WARNING]
> **アルファ版・AI生成コードに関する注意事項**
>
> - 本ライブラリは現在 **アルファ版（v0.0.1）** です。API・仕様は予告なく変更される場合があります。
> - コードの大部分は **AI（LLM）によって生成** されており、人間による網羅的なレビューおよび品質検証が十分に行われていません。
> - 法令計算の正確性について独自に検証を行うことなく、本番環境での利用は推奨しません。
> - 計算結果を実際の法的手続きや業務判断に用いる際は、必ず有資格者または専門家に確認してください。

## 概要

J-Law-Core は、日本の法令・告示・省令が定める各種計算を、整数演算と分数表現で再現するライブラリです。

- 金額計算で `f64` / `f32` を使わず、端数処理の順序を再現
- 法令パラメータを JSON registry として外部管理し、施行日ごとの差分を履歴管理
- Rust コアに加え、C ABI と Python / WASM / Ruby / Go バインディングを提供
- `tests/fixtures/` の共通 JSON を使って複数言語で同じケースを検証

## 実装済みドメイン

| ドメイン | 法的根拠 | 現在の実装範囲 |
| --- | --- | --- |
| `consumption_tax` | 消費税法 第29条 | 標準税率・軽減税率、税額、税込/税抜、適用税率の返却 |
| `real_estate` | 宅地建物取引業法 第46条 | 媒介報酬の3段階ティア計算、低廉な空き家等特例、消費税連携 |
| `income_tax` | 所得税法 第89条 / 復興財源確保法 第13条 | 速算表による所得税額、復興特別所得税、所得控除、通し計算 |
| `stamp_tax` | 印紙税法 別表第一 | 主要文書コード、軽減措置、非課税フラグ、適用ルールの返却 |
| `withholding_tax` | 所得税法 第204条第1項 | 報酬・料金等の二段階税率類型、応募作品賞金の免税、区分消費税控除 |

README では各ドメインの説明を概要レベルに留めています。利用例と API 名の対応は [docs/usage.md](docs/usage.md) を参照してください。

## パッケージ構成

| パッケージ | 役割 | 補足 |
| --- | --- | --- |
| `crates/j-law-core` | コアライブラリ | 型、エラー、各ドメインの計算ロジック |
| `crates/j-law-registry` | 法令パラメータ loader | JSON registry を読み込み、施行日に応じたパラメータを返す |
| `crates/j-law-wasm` | JavaScript / WASM バインディング | `wasm-bindgen` ベース |
| `crates/j-law-c-ffi` | C ABI | Python / Ruby / Go バインディングの共通入口 |
| `crates/j-law-python` | Python バインディング | `ctypes` で C ABI を利用。Cargo workspace 外 |
| `crates/j-law-ruby` | Ruby バインディング | `ffi` で C ABI を利用。Cargo workspace 外 |
| `crates/j-law-go` | Go バインディング | CGo で C ABI を利用。Cargo workspace 外 |

Cargo workspace メンバーは `j-law-core` / `j-law-registry` / `j-law-wasm` / `j-law-c-ffi` です。

## 公開サポート

| 言語 | サポート範囲 | 公開物 |
| --- | --- | --- |
| Python | CPython `3.10`-`3.14` | PyPI wheel: `linux/x86_64` `linux/aarch64` `macos/x86_64` `macos/arm64` `windows/amd64`。その他は source build |
| JavaScript / WASM | Node.js `20` `22` `24` `25` | npm package は `wasm-pack --target nodejs` で生成 |
| Ruby | Ruby `3.1`-`4.0` | RubyGems source gem のみ。install 時に Rust `1.94.0` が必要 |
| Go | Go `1.21+` | `darwin/amd64` `darwin/arm64` `linux/amd64` `linux/arm64` の同梱 native archive。Windows 非対応 |

## ドキュメント

- [利用ガイド](docs/usage.md)
- [Python バインディング](crates/j-law-python/README.md)
- [WASM / JavaScript バインディング](crates/j-law-wasm/README.md)
- [Ruby バインディング](crates/j-law-ruby/README.md)
- [Go バインディング](crates/j-law-go/README.md)
- [実装ルール](AGENTS.md)

## クイックスタート

### Rust

```toml
[dependencies]
j-law-core = { path = "crates/j-law-core" }
j-law-registry = { path = "crates/j-law-registry" }
```

```rust
use std::collections::HashSet;

use j_law_core::domains::real_estate::{
    calculator::calculate_brokerage_fee,
    context::RealEstateContext,
    policy::StandardMliitPolicy,
};
use j_law_core::LegalDate;
use j_law_registry::load_brokerage_fee_params;

let date = LegalDate::new(2024, 8, 1);
let ctx = RealEstateContext {
    price: 5_000_000,
    target_date: date,
    flags: HashSet::new(),
    policy: Box::new(StandardMliitPolicy),
};

let params = load_brokerage_fee_params(date)?;
let result = calculate_brokerage_fee(&ctx, &params)?;

assert_eq!(result.total_with_tax.as_yen(), 231_000);
```

### その他の言語

- Python: `pip install j-law-python`
- JavaScript / WASM: `npm install j-law-wasm`
- Ruby: `gem install j_law_ruby`
- Go: `go get github.com/kmoyashi/j-law-core/crates/j-law-go`（Windows 非対応）

ローカル checkout から直接試す場合は、各 binding README の開発手順を参照してください。

公開 API 名の一覧は [docs/usage.md](docs/usage.md) にまとめています。

## テスト

### Rust CI 相当

```sh
make ci
```

### 全言語バインディング

```sh
make docker-test
```

### よく使う個別コマンド

```sh
cargo test --workspace
pytest crates/j-law-python/tests/ -v
node --test crates/j-law-wasm/tests/*.test.mjs
cd crates/j-law-ruby && bundle exec rake test
cd crates/j-law-go && make test
```

## コントリビューション

- `crates/j-law-core/src/` では `f64` / `f32` を使わない
- `crates/j-law-core/src/` では `panic!` / `unwrap()` / `expect()` を使わない
- 公開 API には根拠条文を doc コメントで明記する
- 変更に対応するテストを追加し、提出前に `make ci` を通す
- ドメイン追加や registry 変更時は関連 binding / fixture / docs も更新する

詳細な実装ルールは [AGENTS.md](AGENTS.md) を参照してください。

## 免責事項

本ライブラリは法的助言を提供するものではありません。計算結果は参考情報であり、実際の手続きにおいては必ず有資格者または専門家に確認してください。法改正により計算ロジックが変わる場合があります。

## ライセンス

[MIT License](LICENSE)
