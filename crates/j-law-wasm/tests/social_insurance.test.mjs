import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);
const { calcSocialInsurance } = require("../pkg/j_law_wasm.js");

const fixtures = JSON.parse(
  readFileSync(resolve(__dirname, "../../../tests/fixtures/social_insurance.json"), "utf-8")
);

function withDate(input) {
  const [year, month, day] = input.date.split("-").map(Number);
  return new Date(Date.UTC(year, month - 1, day));
}

describe("calcSocialInsurance - フィクスチャ駆動", () => {
  for (const c of fixtures.social_insurance) {
    it(`${c.id}: ${c.description}`, () => {
      const r = calcSocialInsurance(
        c.input.standard_monthly_remuneration,
        withDate(c.input),
        c.input.prefecture_code,
        c.input.is_care_insurance_applicable
      );
      assert.equal(r.healthRelatedAmount, c.expected.health_related_amount);
      assert.equal(r.pensionAmount, c.expected.pension_amount);
      assert.equal(r.totalAmount, c.expected.total_amount);
      assert.equal(r.careInsuranceApplied, c.expected.care_insurance_applied);
    });
  }
});

describe("calcSocialInsurance - JS固有テスト", () => {
  it("invalid standard remuneration returns error", () => {
    assert.throws(() =>
      calcSocialInsurance(305_000, new Date(Date.UTC(2026, 2, 1)), 13, false)
    );
  });
});
