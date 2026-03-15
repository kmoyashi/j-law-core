# j-law-go

日本の法令に基づく各種計算を提供する Go バインディングです。

`j-law-c-ffi` が提供する C ABI を CGo 経由で利用しています。

> [!WARNING]
> **アルファ版・AI生成コードに関する注意事項**
>
> - 本ライブラリは現在 **アルファ版（v0.0.1）** です。API は予告なく変更される場合があります。
> - コードの大部分は **AI（LLM）によって生成** されており、人間による網羅的なレビューが十分に行われていません。
> - 計算結果を実際の法的手続きや業務判断に用いる際は、必ず有資格者または専門家に確認してください。

> [!NOTE]
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

Rust コアや C ABI を更新して Go 向け配布物を再生成したいメンテナは、先に `make sync-native` を実行してください。

## 関連ドキュメント

- [リポジトリ README](../../README.md)
- [利用ガイド](../../docs/usage.md)
