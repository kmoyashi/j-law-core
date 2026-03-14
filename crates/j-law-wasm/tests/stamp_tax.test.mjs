/**
 * 印紙税法 別表第一に基づく印紙税額計算のテスト。
 */

import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);
const { calcStampTax } = require("../pkg/j_law_wasm.js");

const fixtures = JSON.parse(
  readFileSync(resolve(__dirname, "../../../tests/fixtures/stamp_tax.json"), "utf-8")
);

describe("calcStampTax - フィクスチャ駆動", () => {
  for (const c of fixtures.stamp_tax) {
    it(`${c.id}: ${c.description}`, () => {
      const { document_code, stated_amount, flags } = c.input;
      const [year, month, day] = c.input.date.split("-").map(Number);
      const date = new Date(Date.UTC(year, month - 1, day));
      const r = calcStampTax(document_code, stated_amount, date, flags);
      const exp = c.expected;

      assert.equal(r.taxAmount, BigInt(exp.tax_amount), "taxAmount");
      assert.equal(r.ruleLabel, exp.rule_label, "ruleLabel");
      assert.equal(r.appliedSpecialRule ?? null, exp.applied_special_rule, "appliedSpecialRule");
    });
  }
});

describe("calcStampTax - JS固有テスト", () => {
  it("対象日が範囲外の場合にエラー", () => {
    assert.throws(() =>
      calcStampTax("article1_real_estate_transfer", 5_000_000, new Date(Date.UTC(2014, 2, 31)))
    );
  });

  it("不正な documentCode を拒否する", () => {
    assert.throws(() =>
      calcStampTax("invalid_code", 5_000_000, new Date(Date.UTC(2024, 7, 1)))
    );
  });

  it("flags に文字列以外を渡すとエラー", () => {
    assert.throws(() =>
      calcStampTax("article17_sales_receipt", 70_000, new Date(Date.UTC(2024, 7, 1)), [1])
    );
  });
});
