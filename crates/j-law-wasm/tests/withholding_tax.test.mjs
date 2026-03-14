/**
 * 所得税法第204条第1項に基づく源泉徴収税額計算のテスト。
 */

import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);
const { calcWithholdingTax } = require("../pkg/j_law_wasm.js");

const fixtures = JSON.parse(
  readFileSync(resolve(__dirname, "../../../tests/fixtures/withholding_tax.json"), "utf-8")
);

function withDate(input) {
  const [year, month, day] = input.date.split("-").map(Number);
  return new Date(Date.UTC(year, month - 1, day));
}

describe("calcWithholdingTax - フィクスチャ駆動", () => {
  for (const c of fixtures.withholding_tax) {
    it(`${c.id}: ${c.description}`, () => {
      const result = calcWithholdingTax(
        c.input.payment_amount,
        withDate(c.input),
        c.input.category,
        c.input.is_submission_prize,
        c.input.separated_consumption_tax_amount
      );

      assert.equal(result.taxablePaymentAmount, c.expected.taxable_payment_amount);
      assert.equal(result.taxAmount, c.expected.tax_amount);
      assert.equal(result.netPaymentAmount, c.expected.net_payment_amount);
      assert.equal(result.submissionPrizeExempted, c.expected.submission_prize_exempted);
    });
  }
});

describe("calcWithholdingTax - JS固有テスト", () => {
  it("対象日が範囲外の場合にエラー", () => {
    assert.throws(() =>
      calcWithholdingTax(
        100_000,
        new Date(Date.UTC(2012, 11, 31)),
        "manuscript_and_lecture",
        false,
        0
      )
    );
  });

  it("breakdown の各フィールドが有効", () => {
    const result = calcWithholdingTax(
      1_500_000,
      new Date(Date.UTC(2026, 0, 1)),
      "professional_fee",
      false,
      0
    );
    const breakdown = result.breakdown();
    assert.equal(breakdown.length, 2);
    assert.ok(breakdown[0].label);
  });
});
