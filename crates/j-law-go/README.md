# j-law-go

日本法令計算の PoC を Go から試すためのバインディングです。

`j-law-c-ffi` が提供する C ABI を CGo 経由で利用しています。

> [!WARNING]
> **PoC / アルファ版に関する注意事項**
>
> - 本ライブラリは現在 **`v0.0.1` のアルファ版**です。API と配布形態は予告なく変更される場合があります。
> - この binding が返す計算結果について、法的正確性、完全性、最新性、個別事案への適合性は保証しません。
> - コードの一部には **AI 生成 / AI 補助**による実装が含まれ、人手による全面レビューは完了していません。
> - 税務申告や契約実務の唯一の根拠として使用せず、一次資料と専門家で検証してください。
> - 詳細は [プロジェクトステータスと免責](../../docs/project-status.md) を参照してください。

> [!NOTE]
> Go `1.21` 以上をサポートします。
> 対応済みの同梱ネイティブ配布物は `darwin/amd64` / `darwin/arm64` / `linux/amd64` / `linux/arm64` です。
> Windows は現時点では非対応です。
> CGo を使うため C コンパイラは必要ですが、対応プラットフォームでは Rust ツールチェインは不要です。

## 対応機能

- `CalcConsumptionTax`
- `CalcBrokerageFee`
- `CalcIncomeTax`
- `CalcIncomeDeductions`
- `CalcIncomeTaxAssessment`
- `CalcStampTax`
- `CalcWithholdingTax`

## インストール

```sh
go get github.com/kmoyashi/j-law-core/crates/j-law-go
```

Windows では利用できません。対応プラットフォームは `darwin/amd64` / `darwin/arm64` / `linux/amd64` / `linux/arm64` です。

これで `go run` / `go test` に進めます。

## クイックスタート

```go
package main

import (
	"fmt"
	"time"

	jlawcore "github.com/kmoyashi/j-law-core/crates/j-law-go"
)

func main() {
	date := time.Date(2024, time.August, 1, 0, 0, 0, 0, time.UTC)

	consumption, _ := jlawcore.CalcConsumptionTax(100_000, time.Date(2024, time.January, 1, 0, 0, 0, 0, time.UTC), false)
	fmt.Println(consumption.TaxAmount)

	fee, _ := jlawcore.CalcBrokerageFee(5_000_000, date, false, false)
	fmt.Println(fee.TotalWithTax)

	assessment, _ := jlawcore.CalcIncomeTaxAssessment(jlawcore.IncomeDeductionInput{
		TotalIncomeAmount:          8_000_000,
		Date:                       time.Date(2024, time.January, 1, 0, 0, 0, 0, time.UTC),
		SocialInsurancePremiumPaid: 600_000,
	}, true)
	fmt.Println(assessment.Tax.TotalTax)

	statedAmount := uint64(5_000_000)
	stamp, _ := jlawcore.CalcStampTax(
		jlawcore.StampTaxDocumentArticle1RealEstateTransfer,
		&statedAmount,
		date,
		nil,
	)
	fmt.Println(stamp.TaxAmount)

	withholding, _ := jlawcore.CalcWithholdingTax(
		1_500_000,
		time.Date(2026, time.January, 1, 0, 0, 0, 0, time.UTC),
		jlawcore.WithholdingTaxCategoryProfessionalFee,
		false,
		0,
	)
	fmt.Println(withholding.TaxAmount)
}
```

## API メモ

- すべての金額は整数円です。
- 日付は `time.Time` で渡します。
- `CalcStampTax()` は文書コード定数と `[]StampTaxFlag` を使います。
- `CalcWithholdingTax()` は `WithholdingTaxCategoryProfessionalFee` などの定数を使います。

## ビルドとテスト

```sh
cd crates/j-law-go
make test
```

Rust コアや C ABI を更新して Go 向け配布物を再生成したいメンテナは、リポジトリルートで `make sync-go-native-all` を実行してください。リポジトリルートの `make sync-go-native` と `crates/j-law-go` 直下の `make sync-native` は現在のプラットフォームのみ更新します。

## 関連ドキュメント

- [プロジェクトステータスと免責](../../docs/project-status.md)
- [リポジトリ README](../../README.md)
- [利用ガイド](../../docs/usage.md)

## ライセンス

[MIT License](../../LICENSE)
