# J-Law-Core 利用ガイド

このドキュメントは、**現在の実装に基づく公開 API の見取り図**です。詳細なコード例や各言語のセットアップは、対応する binding README を参照してください。

## 対応ドメイン

| ドメイン | 代表 API | 概要 |
| --- | --- | --- |
| `consumption_tax` | 消費税額計算 | 標準税率・軽減税率、税込/税抜、適用税率 |
| `real_estate` | 媒介報酬計算 | 3段階ティア、低廉な空き家等特例、消費税連携 |
| `income_tax` | 所得税額計算 | 速算表、復興特別所得税、所得控除、通し計算 |
| `stamp_tax` | 印紙税額計算 | 文書コード別税額、軽減措置、非課税フラグ |
| `withholding_tax` | 源泉徴収税額計算 | 報酬・料金等の二段階税率、応募作品賞金免税、区分消費税控除 |

## 言語別 API マッピング

| 機能 | Rust | Python | JavaScript / WASM | Ruby | Go |
| --- | --- | --- | --- | --- | --- |
| 消費税 | `calculate_consumption_tax` | `calc_consumption_tax` | `calcConsumptionTax` | `calc_consumption_tax` | `CalcConsumptionTax` |
| 不動産媒介報酬 | `calculate_brokerage_fee` | `calc_brokerage_fee` | `calcBrokerageFee` | `calc_brokerage_fee` | `CalcBrokerageFee` |
| 所得税額 | `calculate_income_tax` | `calc_income_tax` | `calcIncomeTax` | `calc_income_tax` | `CalcIncomeTax` |
| 所得控除 | `calculate_income_deductions` | `calc_income_deductions` | `calcIncomeDeductions` | `calc_income_deductions` | `CalcIncomeDeductions` |
| 所得控除から税額まで | `calculate_income_tax_assessment` | `calc_income_tax_assessment` | `calcIncomeTaxAssessment` | `calc_income_tax_assessment` | `CalcIncomeTaxAssessment` |
| 印紙税 | `calculate_stamp_tax` | `calc_stamp_tax` | `calcStampTax` | `calc_stamp_tax` | `CalcStampTax` |
| 源泉徴収 | `calculate_withholding_tax` | `calc_withholding_tax` | `calcWithholdingTax` | `calc_withholding_tax` | `CalcWithholdingTax` |

## 入力の共通ルール

- 金額は原則として **円単位の整数**です。
- 事実認定が必要なフラグは、ライブラリ側ではなく呼び出し元が判断します。
- 端数処理の順序はドメインごとに Rust コアで固定されており、各 binding はその結果を返します。

## 日付型

| 言語 | 日付の渡し方 | 補足 |
| --- | --- | --- |
| Rust | `LegalDate` | registry loader も `LegalDate` を受け取る |
| Python | `datetime.date` | 型不一致は `TypeError` |
| JavaScript / WASM | `Date` | 常に JST として解釈 |
| Ruby | `Date` / `DateTime` | 型不一致は `TypeError` |
| Go | `time.Time` | 年月日部分が使用される |

## ドメイン別の補足

### `consumption_tax`

- 標準税率と軽減税率を施行日に応じて切り替えます。
- Ruby / Python / Go では真偽値フラグ、WASM では `isReducedRate` 引数で軽減税率を指定します。

### `real_estate`

- 宅建業法第46条の媒介報酬上限を計算します。
- 低廉な空き家等特例の適用可否は、`is_low_cost_vacant_house` / `isLowCostVacantHouse` などのフラグで呼び出し側が指定します。

### `income_tax`

- 速算表ベースの税額計算に加え、所得控除単体計算と「所得控除から税額まで」の通し計算を実装しています。
- Python / Go は構造体、Ruby は引数 + keyword、WASM はオブジェクト入力で所得控除情報を渡します。

### `stamp_tax`

- 文書種別は binding ごとに **文字列 / シンボル / 定数** で指定します。
- 非課税フラグも binding ごとに文字列・シンボル・定数のいずれかを使います。

### `withholding_tax`

- 対象は「報酬・料金等」のうち二段階税率類型です。
- カテゴリ指定は Python では enum / 文字列 / 整数、Ruby ではシンボル / 文字列 / 整数、Go では定数、WASM では文字列を使います。

## どのドキュメントを見るべきか

- Rust 全体像: [README.md](../README.md)
- Python: [crates/j-law-python/README.md](../crates/j-law-python/README.md)
- JavaScript / WASM: [crates/j-law-wasm/README.md](../crates/j-law-wasm/README.md)
- Ruby: [crates/j-law-ruby/README.md](../crates/j-law-ruby/README.md)
- Go: [crates/j-law-go/README.md](../crates/j-law-go/README.md)

## 開発時の確認コマンド

```sh
make ci
make docker-test
```
