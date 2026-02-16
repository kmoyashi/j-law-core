# J-Law-Core

[![CI](https://github.com/your-org/j-law-core/actions/workflows/ci.yml/badge.svg)](https://github.com/your-org/j-law-core/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

日本法令に準拠した計算・検証ライブラリ（Rust / Python）

## 概要

J-Law-Core は、日本の法令・告示・省令が定める各種計算を、法的正確性を保証して実装するライブラリです。

- 金額・数値計算に浮動小数点を一切使用せず、整数演算と分数表現で端数処理の再現性を確保
- 法令パラメータ（税率・上限額・経過措置など）をJSONで外部管理し、法改正に対応
- ドメイン単位で法令を追加できる拡張可能なアーキテクチャ
- Rustネイティブライブラリとして利用可能なほか、Python Binding（PyO3）を提供

**実装済みドメイン**

| ドメイン | 対象法令 | 対応告示 |
|---|---|---|
| 不動産（`real_estate`） | 宅地建物取引業法 第46条 | 2019年10月1日施行 / 2024年7月1日施行 |

---

## インストール

### Rust

`Cargo.toml` に追加してください:

```toml
[dependencies]
j-law-core = { git = "https://github.com/your-org/j-law-core" }
```

### Python

```sh
pip install j-law-core
```

---

## 使い方

### Rust — 不動産ドメイン（宅建業法 媒介報酬）

```rust
use j_law_core::domains::real_estate::{
    RealEstateContext, calculator::calculate_brokerage_fee,
    policy::StandardMliitPolicy,
};
use j_law_registry::loader::load_registry;
use std::collections::HashSet;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = RealEstateContext {
        price: 5_000_000,         // 売買価格 500万円
        target_date: (2024, 8, 1),
        flags: HashSet::new(),
        policy: Box::new(StandardMliitPolicy),
    };
    let params = load_registry("real_estate/brokerage_fee", ctx.target_date)?;
    let result = calculate_brokerage_fee(&ctx, &params)?;

    println!("税込報酬額: {}円", result.total_with_tax.as_yen()); // 231,000円
    println!("税抜報酬額: {}円", result.total_without_tax.as_yen()); // 210,000円
    Ok(())
}
```

### Python — 不動産ドメイン（宅建業法 媒介報酬）

```python
import j_law_core

result = j_law_core.real_estate.calc_brokerage_fee(
    price=5_000_000,
    target_date=(2024, 8, 1),
)
print(result.total_with_tax)     # 231000
print(result.total_without_tax)  # 210000
print(result.tax_amount)         # 21000

# 低廉な空き家特例（2024年7月施行・800万円以下）
result = j_law_core.real_estate.calc_brokerage_fee(
    price=8_000_000,
    target_date=(2024, 8, 1),
    flags=["IsLowCostVacantHouse"],
)
print(result.total_with_tax)  # 363000
```

---

## プロジェクト構成

```
j-law-core/
├── crates/
│   ├── j-law-core/               # 共通基盤（型・エラー・計算プリミティブ）
│   │   └── src/
│   │       ├── types/            # FinalAmount, Rate, RoundingStrategy など
│   │       ├── error.rs          # 共通エラー型（JLawError 階層）
│   │       └── domains/
│   │           └── real_estate/  # 不動産ドメイン（Phase 1 参照実装）
│   ├── j-law-registry/           # 法令パラメータ管理
│   │   └── data/
│   │       └── real_estate/      # 宅建業法告示パラメータ（JSON）
│   └── j-law-python/             # Python Binding（PyO3）
└── tests/
    └── real_estate/
        ├── mlitt_examples.rs     # 国交省公式計算例テスト
        └── edge_cases.rs         # 境界値テスト
```

新しい法令ドメインを追加する場合は `crates/j-law-core/src/domains/<ドメイン名>/` 以下に実装し、対応するRegistryデータを `crates/j-law-registry/data/<ドメイン名>/` に配置します。

---

## 不動産ドメイン 計算仕様

宅地建物取引業法第46条・国土交通省告示に基づく媒介報酬の計算仕様です。

### 3段階ティア

| 売買価格の範囲 | 率 |
|---|---|
| 200万円以下の部分 | 5% |
| 200万円超〜400万円以下の部分 | 4% |
| 400万円超の部分 | 3% |

端数処理: 各ティアで切り捨て → 合計 → 消費税10%（切り捨て）

### 低廉な空き家特例（2024年7月1日〜）

売買価格が800万円以下で `IsLowCostVacantHouse` フラグを指定した場合、税抜報酬額の上限が330,000円になります。

> **注意**: `IsLowCostVacantHouse` フラグの事実認定（対象物件が空き家かどうかの判断）はこのライブラリの責任範囲外です。必ず呼び出し元で確認してください。

### 計算例

| 売買価格 | 税抜合計 | 消費税 | 税込合計 |
|---|---|---|---|
| 1,000,000円 | 50,000円 | 5,000円 | 55,000円 |
| 2,000,000円 | 100,000円 | 10,000円 | 110,000円 |
| 3,000,000円 | 140,000円 | 14,000円 | 154,000円 |
| 4,000,000円 | 180,000円 | 18,000円 | 198,000円 |
| 5,000,000円 | 210,000円 | 21,000円 | 231,000円 |
| 10,000,000円 | 360,000円 | 36,000円 | 396,000円 |
| 30,000,000円 | 960,000円 | 96,000円 | 1,056,000円 |
| 8,000,000円（低廉特例） | 330,000円 | 33,000円 | 363,000円 |

出典: 国土交通省「宅地建物取引業法の解釈・運用の考え方」第46条関係

---

## ビルド・テスト

```sh
# 依存: Rust (https://rustup.rs)
cargo build --all
cargo test --all
```

Python Bindingをローカルでビルドする場合:

```sh
pip install maturin
maturin develop -m crates/j-law-python/Cargo.toml
```

---

## コントリビューション

Issue・Pull Request を歓迎します。コードを変更・追加する場合は以下の点に従ってください。

- `crates/j-law-core/src/` に `f64`/`f32` を使わないこと（CIで検出されます）
- 公開APIにはRustdocで根拠条文を明記すること
- 変更に対応するテストを必ず追加すること
- 新規ドメインを追加する場合は既存ドメイン（`real_estate`）の構成に倣うこと

詳細なコーディングルールは [AGENTS.md](AGENTS.md) を参照してください。

---

## 免責事項

本ライブラリは法的助言を提供するものではありません。計算結果は参考情報であり、実際の手続きにおいては必ず有資格者または弁護士に確認してください。法改正により計算ロジックが変わる場合があります。

---

## ライセンス

[MIT License](LICENSE)
