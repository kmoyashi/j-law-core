# j-law-wasm

日本の法令に基づく各種計算を提供する WebAssembly バインディング。

Rust コアライブラリ（j-law-core）を [wasm-bindgen](https://rustwasm.github.io/wasm-bindgen/) 経由でラップしています。
浮動小数点演算を一切使用せず、整数演算で端数処理の再現性を保証します。

> [!WARNING]
> **アルファ版・AI生成コードに関する注意事項**
>
> - 本ライブラリは現在 **アルファ版（v0.0.1）** です。API は予告なく変更される場合があります。
> - コードの大部分は **AI（LLM）によって生成** されており、人間による網羅的なレビューが十分に行われていません。
> - 計算結果を実際の法的手続きや業務判断に用いる際は、必ず有資格者または専門家に確認してください。

> [!NOTE]
> **数値精度について**
>
> JavaScript の `Number` 型は 53bit 整数精度のため、`u64` を直接扱えません。
> `calcBrokerageFee` / `calcIncomeTax` の金額引数は `number`（最大約42.9億円）です。
> 印紙税の `contractAmount` は50億円超のブラケットに対応するため `number`（`f64`）を使用します。

## インストール

```sh
npm install j-law-wasm
```

ソースからビルドする場合:

```sh
# wasm-pack が必要
wasm-pack build --target nodejs crates/j-law-wasm
```

## 使い方

### Node.js (CommonJS)

```js
const { calcBrokerageFee, calcIncomeTax, calcStampTax } = require("j-law-wasm");
```

### ES Modules / バンドラー

```js
import { calcBrokerageFee, calcIncomeTax, calcStampTax } from "j-law-wasm";
```

---

### 不動産ドメイン — 媒介報酬（宅建業法 第46条）

```js
// 売買価格 500万円、2024年8月1日基準
const result = calcBrokerageFee(5_000_000, 2024, 8, 1, false);

console.log(result.totalWithTax);           // 231000（税込）
console.log(result.totalWithoutTax);        // 210000（税抜）
console.log(result.taxAmount);              // 21000（消費税）
console.log(result.lowCostSpecialApplied);  // false

// 各ティアの内訳
const steps = result.breakdown();
// [
//   { label: "tier1", baseAmount: 2000000, rateNumer: 5, rateDenom: 100, result: 100000 },
//   { label: "tier2", baseAmount: 2000000, rateNumer: 4, rateDenom: 100, result: 80000 },
//   { label: "tier3", baseAmount: 1000000, rateNumer: 3, rateDenom: 100, result: 30000 },
// ]

// 低廉な空き家特例（2024年7月1日施行・800万円以下）
// WARNING: 対象物件が特例に該当するかの事実認定は呼び出し元の責任
const special = calcBrokerageFee(8_000_000, 2024, 8, 1, true);
console.log(special.totalWithTax);           // 363000
console.log(special.lowCostSpecialApplied);  // true
```

### 所得税ドメイン — 所得税額（所得税法 第89条）

```js
// 課税所得 500万円（1,000円未満切り捨て済みの値を渡すこと）
const result = calcIncomeTax(5_000_000, 2024, 1, 1, true);

console.log(result.totalTax);                  // 584500（申告納税額・100円未満切り捨て）
console.log(result.baseTax);                   // 572500（基準所得税額）
console.log(result.reconstructionTax);         // 12022（復興特別所得税）
console.log(result.reconstructionTaxApplied);  // true

// 復興特別所得税を適用しない場合
const result2 = calcIncomeTax(5_000_000, 2024, 1, 1, false);
console.log(result2.totalTax);                 // 572500
```

### 印紙税ドメイン — 印紙税額（印紙税法 別表第一）

```js
// 契約金額 500万円（不動産譲渡契約書）
const result = calcStampTax(5_000_000, 2024, 8, 1, false);

console.log(result.taxAmount);           // 2000（印紙税額）
console.log(result.bracketLabel);        // 適用ブラケット名
console.log(result.reducedRateApplied); // false

// 軽減税率適用（租税特別措置法 第91条）
// WARNING: 対象文書が軽減措置の適用要件を満たすかの事実認定は呼び出し元の責任
const special = calcStampTax(5_000_000, 2024, 8, 1, true);
console.log(special.reducedRateApplied); // true
```

## API リファレンス

### `calcBrokerageFee(price, year, month, day, isLowCostVacantHouse)`

宅建業法第46条に基づく媒介報酬を計算する。

| 引数 | 型 | 説明 |
|---|---|---|
| `price` | `number` | 売買価格（円・最大約42.9億円） |
| `year` | `number` | 基準日（年） |
| `month` | `number` | 基準日（月） |
| `day` | `number` | 基準日（日） |
| `isLowCostVacantHouse` | `boolean` | 低廉な空き家特例フラグ |

**戻り値: `BrokerageFeeResult`**

| プロパティ/メソッド | 型 | 説明 |
|---|---|---|
| `totalWithoutTax` | `number` | 税抜合計額（円） |
| `totalWithTax` | `number` | 税込合計額（円） |
| `taxAmount` | `number` | 消費税額（円） |
| `lowCostSpecialApplied` | `boolean` | 低廉な空き家特例が適用されたか |
| `breakdown()` | `Array<{label, baseAmount, rateNumer, rateDenom, result}>` | 各ティアの計算内訳 |

**例外** — 売買価格が不正、または対象日に有効な法令パラメータが存在しない場合に `Error` をスロー。

---

### `calcIncomeTax(taxableIncome, year, month, day, applyReconstructionTax)`

所得税法第89条に基づく所得税額を計算する。

| 引数 | 型 | 説明 |
|---|---|---|
| `taxableIncome` | `number` | 課税所得金額（円・1,000円未満切り捨て済み） |
| `year` | `number` | 対象年度（年） |
| `month` | `number` | 基準日（月） |
| `day` | `number` | 基準日（日） |
| `applyReconstructionTax` | `boolean` | 復興特別所得税を適用するか |

**戻り値: `IncomeTaxResult`**

| プロパティ/メソッド | 型 | 説明 |
|---|---|---|
| `baseTax` | `number` | 基準所得税額（円） |
| `reconstructionTax` | `number` | 復興特別所得税額（円） |
| `totalTax` | `number` | 申告納税額（円・100円未満切り捨て） |
| `reconstructionTaxApplied` | `boolean` | 復興特別所得税が適用されたか |
| `breakdown()` | `Array<{label, taxableIncome, rateNumer, rateDenom, deduction, result}>` | 速算表の計算内訳 |

**例外** — 課税所得金額が不正、または対象日に有効な法令パラメータが存在しない場合に `Error` をスロー。

---

### `calcStampTax(contractAmount, year, month, day, isReducedRateApplicable)`

印紙税法 別表第一（第1号文書）に基づく印紙税額を計算する。

| 引数 | 型 | 説明 |
|---|---|---|
| `contractAmount` | `number` | 契約金額（円・`f64` で受け取り、50億円超も対応） |
| `year` | `number` | 契約書作成日（年） |
| `month` | `number` | 契約書作成日（月） |
| `day` | `number` | 契約書作成日（日） |
| `isReducedRateApplicable` | `boolean` | 軽減税率適用フラグ |

**戻り値: `StampTaxResult`**

| プロパティ | 型 | 説明 |
|---|---|---|
| `taxAmount` | `number` | 印紙税額（円） |
| `bracketLabel` | `string` | 適用されたブラケットの表示名 |
| `reducedRateApplied` | `boolean` | 軽減税率が適用されたか |

**例外** — 契約金額が不正、または対象日に有効な法令パラメータが存在しない場合に `Error` をスロー。

## テスト

```sh
wasm-pack build --target nodejs crates/j-law-wasm
node --test crates/j-law-wasm/tests/*.test.mjs
```

## ライセンス

[MIT License](../../LICENSE)
