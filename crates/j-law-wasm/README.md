# j-law-wasm

日本法令計算の PoC を JavaScript / TypeScript から試すための WebAssembly バインディングです。

Rust コアライブラリを `wasm-bindgen` 経由でラップし、JavaScript / TypeScript から利用できます。

> [!WARNING]
> **PoC / アルファ版に関する注意事項**
>
> - 本ライブラリは現在 **`v0.0.1` のアルファ版**です。API と配布形態は予告なく変更される場合があります。
> - この binding が返す計算結果について、法的正確性、完全性、最新性、個別事案への適合性は保証しません。
> - コードの一部には **AI 生成 / AI 補助**による実装が含まれ、人手による全面レビューは完了していません。
> - 税務申告や契約実務の唯一の根拠として使用せず、一次資料と専門家で検証してください。
> - 詳細は [プロジェクトステータスと免責](../../docs/project-status.md) を参照してください。

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
CI では Node.js `20` / `22` / `24` / `25` を技術的に検証しています。

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

- [プロジェクトステータスと免責](../../docs/project-status.md)
- [リポジトリ README](../../README.md)
- [利用ガイド](../../docs/usage.md)

## ライセンス

[MIT License](../../LICENSE)
