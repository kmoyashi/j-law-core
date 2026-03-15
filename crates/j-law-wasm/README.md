# j-law-wasm

日本の法令に基づく各種計算を提供する WebAssembly バインディングです。

Rust コアライブラリを `wasm-bindgen` 経由でラップし、JavaScript / TypeScript から利用できます。

> [!WARNING]
> **アルファ版・AI生成コードに関する注意事項**
>
> - 本ライブラリは現在 **アルファ版（v0.0.1）** です。API は予告なく変更される場合があります。
> - コードの大部分は **AI（LLM）によって生成** されており、人間による網羅的なレビューが十分に行われていません。
> - 計算結果を実際の法的手続きや業務判断に用いる際は、必ず有資格者または専門家に確認してください。

## 対応機能

- `calcConsumptionTax(amount, date, isReducedRate = false)`
- `calcBrokerageFee(price, date, isLowCostVacantHouse = false, isSeller = false)`
- `calcIncomeTax(taxableIncome, date, applyReconstructionTax = true)`
- `calcIncomeDeductions(input)`
- `calcIncomeTaxAssessment(input, applyReconstructionTax = true)`
- `calcStampTax(documentCode, statedAmount, date, flags = [])`
- `calcWithholdingTax(paymentAmount, date, category, isSubmissionPrize = false, separatedConsumptionTaxAmount = 0)`

## インストール

```sh
npm install j-law-wasm
```

公開 npm パッケージは `wasm-pack --target nodejs` で生成した Node.js 向け配布物です。
CI では Node.js `20` / `22` / `24` / `25` を検証しています。

ソースからビルドする場合:

```sh
wasm-pack build --target nodejs crates/j-law-wasm
```

ブラウザや bundler 向け成果物が必要な場合は、公開 npm パッケージではなく次を使ってソースから生成してください。

```sh
wasm-pack build --target bundler crates/j-law-wasm
```

## クイックスタート

```js
import jLawWasm from "j-law-wasm";

const {
  calcBrokerageFee,
  calcConsumptionTax,
  calcIncomeTaxAssessment,
  calcStampTax,
  calcWithholdingTax,
} = jLawWasm;

const date = new Date(Date.UTC(2024, 7, 1));

console.log(calcConsumptionTax(100_000, date, false).taxAmount);
console.log(calcBrokerageFee(5_000_000, date, false, false).totalWithTax);

const assessment = calcIncomeTaxAssessment(
  {
    totalIncomeAmount: 8_000_000n,
    date: new Date(Date.UTC(2024, 0, 1)),
    socialInsurancePremiumPaid: 600_000n,
  },
  true,
);
console.log(assessment.totalTax);

console.log(calcStampTax("article1_real_estate_transfer", 5_000_000, date).taxAmount);
console.log(
  calcWithholdingTax(
    1_500_000,
    new Date(Date.UTC(2026, 0, 1)),
    "professional_fee",
    false,
    0,
  ).taxAmount,
);
```

## API メモ

- すべての `Date` 引数は **JST** として解釈されます。
- `calcConsumptionTax` / `calcBrokerageFee` / `calcIncomeTax` / `calcWithholdingTax` は安全な整数 `number` を受け取ります。
- `calcIncomeDeductions` / `calcIncomeTaxAssessment` は `number` または `BigInt` の入力を受け取り、`u64` 相当の戻り値は `BigInt` になります。
- `calcStampTax()` の `documentCode` と `flags` は文字列で指定します。

## テスト

```sh
wasm-pack build --target nodejs crates/j-law-wasm
node --test crates/j-law-wasm/tests/*.test.mjs
```

## 関連ドキュメント

- [リポジトリ README](../../README.md)
- [利用ガイド](../../docs/usage.md)

## ライセンス

[MIT License](../../LICENSE)
