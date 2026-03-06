/**
 * 消費税法第29条に基づく消費税額計算のテスト。
 *
 * 法的根拠: 消費税法 第29条（税率）
 * テストケースは tests/fixtures/consumption_tax.json から読み込む。
 *
 * 実行方法:
 *   wasm-pack build --target nodejs crates/j-law-wasm
 *   node --test crates/j-law-wasm/tests/consumption_tax.test.mjs
 */

import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);
const { calcConsumptionTax } = require("../pkg/j_law_wasm.js");

const fixtures = JSON.parse(
  readFileSync(resolve(__dirname, "../../../tests/fixtures/consumption_tax.json"), "utf-8")
);

// ─── データ駆動テスト ───────────────────────────────────────────────────────

describe("calcConsumptionTax - フィクスチャ駆動", () => {
  for (const c of fixtures.consumption_tax) {
    it(`${c.id}: ${c.description}`, () => {
      const { amount, is_reduced_rate } = c.input;
      const [year, month, day] = c.input.date.split("-").map(Number);
      const r = calcConsumptionTax(amount, year, month, day, is_reduced_rate);
      const exp = c.expected;

      assert.equal(r.taxAmount, exp.tax_amount, "taxAmount");
      assert.equal(r.amountWithTax, exp.amount_with_tax, "amountWithTax");
      assert.equal(r.amountWithoutTax, exp.amount_without_tax, "amountWithoutTax");
      assert.equal(r.appliedRateNumer, exp.applied_rate_numer, "appliedRateNumer");
      assert.equal(r.appliedRateDenom, exp.applied_rate_denom, "appliedRateDenom");
      assert.equal(r.isReducedRate, exp.is_reduced_rate, "isReducedRate");
    });
  }
});

// ─── 言語固有テスト ─────────────────────────────────────────────────────────

describe("calcConsumptionTax - JS固有テスト", () => {
  it("軽減税率フラグを立てても対応期間外ならエラー", () => {
    assert.throws(() => calcConsumptionTax(100_000, 2016, 1, 1, true));
  });

  it("消費税導入前は税額ゼロで正常終了", () => {
    const r = calcConsumptionTax(100_000, 1988, 1, 1, false);
    assert.equal(r.taxAmount, 0);
    assert.equal(r.amountWithTax, 100_000);
  });
});
