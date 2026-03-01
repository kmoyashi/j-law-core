# J-Law-Core 各言語での利用方法

J-Law-Core は宅建業法第46条に基づく媒介報酬計算を、Rust で実装されたコアエンジンを通じて複数の言語から利用できるライブラリです。

## 目次

- [Rust](#rust)
- [Python](#python)
- [JavaScript / TypeScript (WebAssembly)](#javascript--typescript-webassembly)
- [Ruby](#ruby)
- [Go](#go)
- [C / C++](#c--c)
- [共通仕様](#共通仕様)

---

## Rust

コアライブラリを直接利用します。

### インストール

```toml
# Cargo.toml
[dependencies]
j-law-core     = { path = "crates/j-law-core" }
j-law-registry = { path = "crates/j-law-registry" }
```

### 使用例

```rust
use std::collections::HashSet;

use j_law_core::domains::real_estate::{
    calculator::calculate_brokerage_fee,
    context::RealEstateContext,
    policy::StandardMliitPolicy,
    RealEstateFlag,
};
use j_law_registry::load_brokerage_fee_params;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 基本的な計算（売買価格 500万円、2024年8月1日）
    let ctx = RealEstateContext {
        price: 5_000_000,
        target_date: (2024, 8, 1),
        flags: HashSet::new(),
        policy: Box::new(StandardMliitPolicy),
    };
    let params = load_brokerage_fee_params((2024, 8, 1))?;
    let result = calculate_brokerage_fee(&ctx, &params)?;

    println!("税抜: {}円", result.total_without_tax.as_yen());  // 210,000
    println!("税込: {}円", result.total_with_tax.as_yen());      // 231,000
    println!("消費税: {}円", result.tax_amount.as_yen());         // 21,000

    // 内訳の確認
    for step in &result.breakdown {
        println!(
            "{}: {}円 × {}/{} = {}円",
            step.label,
            step.base_amount,
            step.rate_numer,
            step.rate_denom,
            step.result.as_yen(),
        );
    }

    // 低廉な空き家特例の適用（800万円以下 + フラグあり）
    let mut flags = HashSet::new();
    flags.insert(RealEstateFlag::IsLowCostVacantHouse);

    let ctx_special = RealEstateContext {
        price: 8_000_000,
        target_date: (2024, 8, 1),
        flags,
        policy: Box::new(StandardMliitPolicy),
    };
    let result_special = calculate_brokerage_fee(&ctx_special, &params)?;

    println!("特例適用: {}", result_special.low_cost_special_applied);  // true
    println!("税込: {}円", result_special.total_with_tax.as_yen());     // 363,000

    Ok(())
}
```

### エラーハンドリング

```rust
use j_law_core::error::{JLawError, InputError, CalculationError, RegistryError};

match load_brokerage_fee_params((2019, 9, 30)) {
    Ok(params) => { /* ... */ },
    Err(JLawError::Input(InputError::DateOutOfRange { date })) => {
        eprintln!("対象日が範囲外です: {}", date);
    },
    Err(e) => eprintln!("エラー: {}", e),
}
```

### ビルド・テスト

```bash
cargo build --all
cargo test --all
```

---

## Python

PyO3 を使った Python バインディングです。

### インストール

```bash
# ソースからビルド（要 Rust toolchain）
pip install maturin
maturin develop -m crates/j-law-python/Cargo.toml
```

### 使用例

```python
import j_law_python

# 基本的な計算（売買価格 500万円、2024年8月1日）
result = j_law_python.real_estate.calc_brokerage_fee(
    price=5_000_000,
    year=2024,
    month=8,
    day=1,
)

print(result.total_without_tax)  # 210000
print(result.total_with_tax)     # 231000
print(result.tax_amount)         # 21000

# 内訳の確認
for step in result.breakdown:
    print(f"{step.label}: {step.base_amount}円 × {step.rate_numer}/{step.rate_denom} = {step.result}円")
# tier1: 2000000円 × 5/100 = 100000円
# tier2: 2000000円 × 4/100 = 80000円
# tier3: 1000000円 × 3/100 = 30000円

# 低廉な空き家特例の適用
result = j_law_python.real_estate.calc_brokerage_fee(
    price=8_000_000,
    year=2024,
    month=8,
    day=1,
    is_low_cost_vacant_house=True,
)

print(result.low_cost_special_applied)  # True
print(result.total_with_tax)            # 363000
```

### エラーハンドリング

```python
try:
    result = j_law_python.real_estate.calc_brokerage_fee(
        price=5_000_000,
        year=2019,
        month=9,
        day=30,  # 施行前のためエラー
    )
except ValueError as e:
    print(f"エラー: {e}")
```

### API リファレンス

```python
j_law_python.real_estate.calc_brokerage_fee(
    price: int,                          # 売買価格（円）
    year: int,                           # 基準日（年）
    month: int,                          # 基準日（月）
    day: int,                            # 基準日（日）
    is_low_cost_vacant_house: bool = False,  # 低廉な空き家特例フラグ
) -> BrokerageFeeResult
```

**BrokerageFeeResult**

| 属性                       | 型                    | 説明                       |
| -------------------------- | --------------------- | -------------------------- |
| `total_without_tax`        | `int`                 | 税抜合計額（円）           |
| `total_with_tax`           | `int`                 | 税込合計額（円）           |
| `tax_amount`               | `int`                 | 消費税額（円）             |
| `low_cost_special_applied` | `bool`                | 低廉な空き家特例の適用有無 |
| `breakdown`                | `list[BreakdownStep]` | 各ティアの計算内訳         |

**BreakdownStep**

| 属性          | 型    | 説明                 |
| ------------- | ----- | -------------------- |
| `label`       | `str` | ティア名称           |
| `base_amount` | `int` | ティア対象金額（円） |
| `rate_numer`  | `int` | 料率の分子           |
| `rate_denom`  | `int` | 料率の分母           |
| `result`      | `int` | ティア計算結果（円） |

---

## JavaScript / TypeScript (WebAssembly)

wasm-bindgen を使った WebAssembly バインディングです。ブラウザ・Node.js の両方で動作します。

### インストール

```bash
# npm パッケージとして（公開後）
npm install j-law-wasm

# ソースからビルド（要 wasm-pack）
cd crates/j-law-wasm
wasm-pack build --target bundler    # バンドラ向け（webpack, vite 等）
wasm-pack build --target nodejs     # Node.js 向け
wasm-pack build --target web        # ブラウザ直接読み込み向け
```

### 使用例（JavaScript）

```javascript
import { calcBrokerageFee } from "j-law-wasm";

// 基本的な計算（売買価格 500万円、2024年8月1日）
const result = calcBrokerageFee(5_000_000, 2024, 8, 1, false);

console.log(result.totalWithoutTax); // 210000
console.log(result.totalWithTax); // 231000
console.log(result.taxAmount); // 21000

// 内訳の確認
for (const step of result.breakdown()) {
  console.log(
    `${step.label}: ${step.baseAmount}円 × ${step.rateNumer}/${step.rateDenom} = ${step.result}円`,
  );
}

// 低廉な空き家特例の適用
const result2 = calcBrokerageFee(8_000_000, 2024, 8, 1, true);

console.log(result2.lowCostSpecialApplied); // true
console.log(result2.totalWithTax); // 363000
```

### 使用例（TypeScript）

```typescript
import { calcBrokerageFee, BrokerageFeeResult } from "j-law-wasm";

const result: BrokerageFeeResult = calcBrokerageFee(
  5_000_000,
  2024,
  8,
  1,
  false,
);

console.log(result.totalWithTax); // 231000

interface BreakdownStep {
  label: string;
  baseAmount: number;
  rateNumer: number;
  rateDenom: number;
  result: number;
}

const breakdown: BreakdownStep[] = result.breakdown();
breakdown.forEach((step) => {
  console.log(`${step.label}: ${step.result}円`);
});
```

### エラーハンドリング

```javascript
try {
  const result = calcBrokerageFee(5_000_000, 2019, 9, 30, false);
} catch (e) {
  console.error(`計算エラー: ${e}`); // 文字列として throw される
}
```

### API リファレンス

```typescript
function calcBrokerageFee(
  price: number, // 売買価格（円）※ u32 上限: 約42.9億円
  year: number, // 基準日（年）
  month: number, // 基準日（月）
  day: number, // 基準日（日）
  isLowCostVacantHouse: boolean, // 低廉な空き家特例フラグ
): BrokerageFeeResult;
```

**BrokerageFeeResult**

| プロパティ              | 型                                                         | 説明                       |
| ----------------------- | ---------------------------------------------------------- | -------------------------- |
| `totalWithoutTax`       | `number`                                                   | 税抜合計額（円）           |
| `totalWithTax`          | `number`                                                   | 税込合計額（円）           |
| `taxAmount`             | `number`                                                   | 消費税額（円）             |
| `lowCostSpecialApplied` | `boolean`                                                  | 低廉な空き家特例の適用有無 |
| `breakdown()`           | `Array<{label, baseAmount, rateNumer, rateDenom, result}>` | 各ティアの計算内訳         |

> **Note:** WASM バインディングの金額は `u32`（最大約42.9億円）です。JavaScript の Number 精度制約との互換性を保つための設計です。

---

## Ruby

Magnus を使った Ruby バインディングです。

### インストール

```bash
# ソースからビルド（要 Rust toolchain）
cd crates/j-law-ruby
bundle install
rake compile
```

### 要件

- Ruby >= 3.0
- Rust toolchain
- rb_sys ~> 0.9

### 使用例

```ruby
require "j_law_ruby"

# 基本的な計算（売買価格 500万円、2024年8月1日）
result = JLawRuby::RealEstate.calc_brokerage_fee(5_000_000, 2024, 8, 1, false)

puts result.total_without_tax  # 210000
puts result.total_with_tax     # 231000
puts result.tax_amount         # 21000

# 内訳の確認
result.breakdown.each do |step|
  puts "#{step[:label]}: #{step[:base_amount]}円 × #{step[:rate_numer]}/#{step[:rate_denom]} = #{step[:result]}円"
end
# tier1: 2000000円 × 5/100 = 100000円
# tier2: 2000000円 × 4/100 = 80000円
# tier3: 1000000円 × 3/100 = 30000円

# 低廉な空き家特例の適用
result = JLawRuby::RealEstate.calc_brokerage_fee(8_000_000, 2024, 8, 1, true)

puts result.low_cost_special_applied?  # true
puts result.total_with_tax             # 363000

# 文字列表現
puts result.inspect
# #<JLawRuby::RealEstate::BrokerageFeeResult total_without_tax=330000 total_with_tax=363000 ...>
```

### エラーハンドリング

```ruby
begin
  result = JLawRuby::RealEstate.calc_brokerage_fee(5_000_000, 2019, 9, 30, false)
rescue RuntimeError => e
  puts "エラー: #{e.message}"
end
```

### API リファレンス

```ruby
JLawRuby::RealEstate.calc_brokerage_fee(
  price,                       # Integer - 売買価格（円）
  year,                        # Integer - 基準日（年）
  month,                       # Integer - 基準日（月）
  day,                         # Integer - 基準日（日）
  is_low_cost_vacant_house     # true/false - 低廉な空き家特例フラグ
) -> JLawRuby::RealEstate::BrokerageFeeResult
```

**BrokerageFeeResult**

| メソッド                    | 戻り値        | 説明                       |
| --------------------------- | ------------- | -------------------------- |
| `total_without_tax`         | `Integer`     | 税抜合計額（円）           |
| `total_with_tax`            | `Integer`     | 税込合計額（円）           |
| `tax_amount`                | `Integer`     | 消費税額（円）             |
| `low_cost_special_applied?` | `true/false`  | 低廉な空き家特例の適用有無 |
| `breakdown`                 | `Array<Hash>` | 各ティアの計算内訳         |
| `inspect` / `to_s`          | `String`      | 文字列表現                 |

**breakdown の Hash キー**

| キー           | 型        | 説明                 |
| -------------- | --------- | -------------------- |
| `:label`       | `String`  | ティア名称           |
| `:base_amount` | `Integer` | ティア対象金額（円） |
| `:rate_numer`  | `Integer` | 料率の分子           |
| `:rate_denom`  | `Integer` | 料率の分母           |
| `:result`      | `Integer` | ティア計算結果（円） |

---

## Go

CGo を経由して Rust の静的ライブラリ（`libj_law_cgo.a`）にリンクします。

### セットアップ

```bash
cd crates/j-law-go

# 1. Rust 静的ライブラリをビルド
make build-rust

# 2. テスト実行
make test
```

### 要件

- Go 1.21+
- Rust toolchain（静的ライブラリのビルドに必要）
- C コンパイラ（CGo に必要）

### 使用例

```go
package main

import (
	"fmt"
	"log"

	jlawcore "github.com/kmoyashi/j-law-go"
)

func main() {
	// 基本的な計算（売買価格 500万円、2024年8月1日）
	result, err := jlawcore.CalcBrokerageFee(5_000_000, 2024, 8, 1, false)
	if err != nil {
		log.Fatal(err)
	}

	fmt.Println(result.TotalWithoutTax)  // 210000
	fmt.Println(result.TotalWithTax)     // 231000
	fmt.Println(result.TaxAmount)        // 21000

	// 内訳の確認
	for _, step := range result.Breakdown {
		fmt.Printf("%s: %d円 × %d/%d = %d円\n",
			step.Label, step.BaseAmount, step.RateNumer, step.RateDenom, step.Result)
	}

	// 低廉な空き家特例の適用
	result2, err := jlawcore.CalcBrokerageFee(8_000_000, 2024, 8, 1, true)
	if err != nil {
		log.Fatal(err)
	}

	fmt.Println(result2.LowCostSpecialApplied)  // true
	fmt.Println(result2.TotalWithTax)            // 363000
}
```

### エラーハンドリング

```go
result, err := jlawcore.CalcBrokerageFee(5_000_000, 2019, 9, 30, false)
if err != nil {
    // err.Error() にエラーメッセージが含まれる
    fmt.Printf("エラー: %v\n", err)
}
```

### API リファレンス

```go
func CalcBrokerageFee(
    price uint64,
    year, month, day int,
    isLowCostVacantHouse bool,
) (*BrokerageFeeResult, error)
```

**BrokerageFeeResult**

| フィールド              | 型                | 説明                       |
| ----------------------- | ----------------- | -------------------------- |
| `TotalWithoutTax`       | `uint64`          | 税抜合計額（円）           |
| `TotalWithTax`          | `uint64`          | 税込合計額（円）           |
| `TaxAmount`             | `uint64`          | 消費税額（円）             |
| `LowCostSpecialApplied` | `bool`            | 低廉な空き家特例の適用有無 |
| `Breakdown`             | `[]BreakdownStep` | 各ティアの計算内訳         |

**BreakdownStep**

| フィールド   | 型       | 説明                 |
| ------------ | -------- | -------------------- |
| `Label`      | `string` | ティア名称           |
| `BaseAmount` | `uint64` | ティア対象金額（円） |
| `RateNumer`  | `uint64` | 料率の分子           |
| `RateDenom`  | `uint64` | 料率の分母           |
| `Result`     | `uint64` | ティア計算結果（円） |

### Makefile ターゲット

| ターゲット                | 説明                                |
| ------------------------- | ----------------------------------- |
| `make build-rust`         | Rust 静的ライブラリをデバッグビルド |
| `make build-rust-release` | Rust 静的ライブラリをリリースビルド |
| `make test`               | デバッグビルドで Go テストを実行    |
| `make test-release`       | リリースビルドで Go テストを実行    |
| `make clean`              | テストキャッシュをクリア            |

---

## C / C++

`j-law-cgo` クレートが C FFI を提供します。Go 以外の C 互換言語からも利用可能です。

### ヘッダファイル

```c
#include "j_law_cgo.h"
```

### 使用例（C）

```c
#include <stdio.h>
#include "j_law_cgo.h"

int main(void) {
    JLawBrokerageFeeResult result;
    char error_buf[J_LAW_ERROR_BUF_LEN];

    int ret = j_law_calc_brokerage_fee(
        5000000,       /* price */
        2024,          /* year */
        8,             /* month */
        1,             /* day */
        0,             /* is_low_cost_vacant_house: 0=false */
        &result,
        error_buf,
        J_LAW_ERROR_BUF_LEN
    );

    if (ret != 0) {
        fprintf(stderr, "エラー: %s\n", error_buf);
        return 1;
    }

    printf("税抜: %llu円\n", result.total_without_tax);   /* 210000 */
    printf("税込: %llu円\n", result.total_with_tax);       /* 231000 */
    printf("消費税: %llu円\n", result.tax_amount);          /* 21000 */

    /* 内訳 */
    for (int i = 0; i < result.breakdown_len; i++) {
        printf("%s: %llu円\n",
            result.breakdown[i].label,
            result.breakdown[i].result);
    }

    return 0;
}
```

### ビルド

```bash
# 静的ライブラリのビルド
cargo build -p j-law-cgo

# C プログラムとリンク（macOS）
cc -o example example.c \
    -I crates/j-law-cgo \
    -L target/debug \
    -lj_law_cgo \
    -framework Security -framework CoreFoundation

# C プログラムとリンク（Linux）
cc -o example example.c \
    -I crates/j-law-cgo \
    -L target/debug \
    -lj_law_cgo \
    -ldl -lpthread -lm
```

### API リファレンス

```c
int j_law_calc_brokerage_fee(
    uint64_t price,                      // 売買価格（円）
    uint16_t year,                       // 基準日（年）
    uint8_t  month,                      // 基準日（月）
    uint8_t  day,                        // 基準日（日）
    int      is_low_cost_vacant_house,   // 0=false, 非0=true
    JLawBrokerageFeeResult *out_result,  // [OUT] 結果書き込み先
    char     *error_buf,                 // [OUT] エラーメッセージ
    int      error_buf_len               // error_buf のバイト長
);
// 戻り値: 0=成功, 非0=失敗
```

### 定数

| 定数                  | 値  | 説明                                       |
| --------------------- | --- | ------------------------------------------ |
| `J_LAW_MAX_TIERS`     | 8   | ティア内訳の最大件数                       |
| `J_LAW_LABEL_LEN`     | 64  | ティアラベルの最大バイト長（NUL 終端含む） |
| `J_LAW_ERROR_BUF_LEN` | 256 | エラーバッファの推奨バイト長               |

---

## 共通仕様

### 対応法令

| 施行日     | 内容                   | status     |
| ---------- | ---------------------- | ---------- |
| 2019-10-01 | 消費税10%対応          | superseded |
| 2024-07-01 | 低廉な空き家特例の追加 | active     |

### 3段階ティア計算

| ティア | 対象範囲              | 料率  |
| ------ | --------------------- | ----- |
| tier1  | 200万円以下           | 5/100 |
| tier2  | 200万円超 400万円以下 | 4/100 |
| tier3  | 400万円超             | 3/100 |

- 各ティアの端数処理: 切り捨て（Floor）
- 消費税率: 10/100（切り捨て）
- 低廉な空き家特例（2024-07-01〜）: 800万円以下 + フラグあり → 税抜上限 33万円に引き上げ

### 計算例

| 売買価格     | 税抜合計 | 消費税 | 税込合計 | 備考                   |
| ------------ | -------- | ------ | -------- | ---------------------- |
| 1,000,000円  | 50,000   | 5,000  | 55,000   |                        |
| 2,000,000円  | 100,000  | 10,000 | 110,000  |                        |
| 3,000,000円  | 140,000  | 14,000 | 154,000  |                        |
| 5,000,000円  | 210,000  | 21,000 | 231,000  |                        |
| 10,000,000円 | 360,000  | 36,000 | 396,000  |                        |
| 8,000,000円  | 330,000  | 33,000 | 363,000  | 低廉な空き家特例適用時 |

### 注意事項

> **WARNING:** 対象物件が「低廉な空き家」に該当するかの事実認定は呼び出し元の責任です。本ライブラリは `is_low_cost_vacant_house` フラグの値に基づいて計算するのみであり、フラグの妥当性は検証しません。

### 各バインディング比較

|                    | Rust                   | Python                | JS/TS           | Ruby           | Go                | C                 |
| ------------------ | ---------------------- | --------------------- | --------------- | -------------- | ----------------- | ----------------- |
| バインディング方式 | ネイティブ             | PyO3                  | wasm-bindgen    | Magnus         | CGo               | FFI               |
| 金額型             | `u64`                  | `int`                 | `u32`           | `Integer`      | `uint64`          | `uint64_t`        |
| エラー型           | `JLawError`            | `ValueError`          | throw (string)  | `RuntimeError` | `error`           | 戻り値 + バッファ |
| 内訳の形式         | `Vec<CalculationStep>` | `list[BreakdownStep]` | `Array<Object>` | `Array<Hash>`  | `[]BreakdownStep` | 固定長配列        |
