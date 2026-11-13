# J-Law-Core 利用ガイド

> [!WARNING]
> このドキュメントは、**現在の実装に基づく公開 API の見取り図**です。PoC / alpha のため、ここに記載された API や計算結果について法的正確性、完全性、最新性、個別事案への適合性は保証されません。利用前に [プロジェクトステータスと免責](project-status.md) を確認してください。

詳細なコード例や各言語のセットアップは、対応する binding README を参照してください。

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

#### 人的控除の年分別パラメータ

基礎控除額・扶養親族等の所得要件は、基準日（`date`）の年分に応じて registry から切り替わります。

| 年分 | 基礎控除（合計所得金額に応じた額） | 配偶者・扶養親族の所得要件 | 特定親族特別控除 |
| --- | --- | --- | --- |
| 2020〜2024年分 | 48万円（2,400万円超で逓減） | 48万円以下 | なし |
| 2025年分（令和7年分） | 95万 / 88万 / 68万 / 63万 / 58万円（132万・336万・489万・655万・2,350万円で区分） | 58万円以下 | 58万円超123万円以下が対象 |
| 2026〜2027年分（令和8・9年分） | 104万 / 67万 / 62万円（489万・655万・2,350万円で区分） | 62万円以下 | 62万円超123万円以下が対象 |
| 2028年分以後（令和10年分以後） | 99万 / 62万円（132万・2,350万円で区分） | 62万円以下 | 62万円超123万円以下が対象 |

- 合計所得金額2,350万円超の基礎控除（48万 / 32万 / 16万 / 0円）は全年分で共通です。
- 2028年分以後の値は令和8年法律第12号時点のものです。2028年分以後は物価連動で2年ごとに見直す方針が示されているため、改正があれば registry を更新します。
- 出典: 国税庁タックスアンサー No.1199（基礎控除）/ No.1177（特定親族特別控除）/ No.1191（配偶者控除）、「源泉所得税の改正のあらまし」（令和8年4月）。

#### 特定親族特別控除（所得税法 第84条の2）

特定親族（生計を一にする19歳以上23歳未満の親族で、控除対象扶養親族に該当しない人）ごとに合計所得金額を渡すと、控除額表から控除額を引き当てて合算します。該当性の判断は呼び出し側の責任です。

| 言語 | 入力 |
| --- | --- |
| Rust | `PersonalDeductionInput.specific_relatives: Vec<SpecificRelativeDeductionInput>` |
| Python | `IncomeDeductionInput(specific_relatives=(SpecificRelativeDeductionInput(total_income_amount=...),))` |
| JavaScript / WASM | `specificRelatives: [{ totalIncomeAmount: ... }]` |
| Ruby | `specific_relatives: [{ total_income_amount: ... }]` |
| Go | `IncomeDeductionInput.SpecificRelatives []SpecificRelativeDeductionInput` |

- C ABI 経由の binding（Python / Ruby / Go）では最大 8 人まで指定できます。
- 内訳（breakdown）の「特定親族特別控除」行（kind = 8）は、制度がある2025年分以後のみ扶養控除の直後に含まれます。2024年分以前の内訳は従来どおり7行です。
- 合計所得金額が控除対象範囲外の特定親族は 0 円です。所得要件以下の親族は特定扶養親族として扶養控除の人数に計上してください。

#### 未対応・前提

- 基礎控除の特例加算（租税特別措置法 第41条の16の2）は居住者のみ適用されるため、居住者を前提に計算します（非居住者は未対応）。
- 基礎控除・配偶者控除の区分判定には入力の `total_income_amount`（総所得金額等）を合計所得金額として用います。
- 配偶者特別控除、給与所得控除（給与収入からの所得計算）、障害者控除・ひとり親控除・勤労学生控除などは未実装です。
- 扶養親族・配偶者の所得要件のうち、扶養控除の人数判定は呼び出し側で行います（配偶者控除は配偶者の合計所得金額から判定します）。

### `stamp_tax`

- 文書種別は binding ごとに **文字列 / シンボル / 定数** で指定します。
- 非課税フラグも binding ごとに文字列・シンボル・定数のいずれかを使います。

### `withholding_tax`

- 対象は「報酬・料金等」のうち二段階税率類型です。
- カテゴリ指定は Python では enum / 文字列 / 整数、Ruby ではシンボル / 文字列 / 整数、Go では定数、WASM では文字列を使います。

## どのドキュメントを見るべきか

- プロジェクトの位置付けと免責: [docs/project-status.md](project-status.md)
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
