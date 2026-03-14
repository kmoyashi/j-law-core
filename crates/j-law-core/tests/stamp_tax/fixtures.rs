#![allow(clippy::disallowed_methods)]

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use j_law_core::domains::stamp_tax::{
    calculator::calculate_stamp_tax,
    context::{StampTaxContext, StampTaxDocumentCode, StampTaxFlag},
    policy::StandardNtaPolicy,
};
use j_law_core::LegalDate;
use j_law_registry::load_stamp_tax_params;
use serde::Deserialize;

#[derive(Deserialize)]
struct FixtureRoot {
    stamp_tax: Vec<FixtureCase>,
}

#[derive(Deserialize)]
struct FixtureCase {
    id: String,
    input: FixtureInput,
    expected: FixtureExpected,
}

#[derive(Deserialize)]
struct FixtureInput {
    document_code: String,
    stated_amount: Option<u64>,
    date: String,
    flags: Vec<String>,
}

#[derive(Deserialize)]
struct FixtureExpected {
    tax_amount: u64,
    rule_label: String,
    applied_special_rule: Option<String>,
}

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/stamp_tax.json")
        .canonicalize()
        .unwrap()
}

fn parse_date(date: &str) -> LegalDate {
    let mut parts = date.split('-');
    let year = parts.next().unwrap().parse::<u16>().unwrap();
    let month = parts.next().unwrap().parse::<u8>().unwrap();
    let day = parts.next().unwrap().parse::<u8>().unwrap();
    LegalDate::new(year, month, day)
}

fn load_fixtures() -> FixtureRoot {
    let content = fs::read_to_string(fixture_path()).unwrap();
    serde_json::from_str(&content).unwrap()
}

#[test]
fn stamp_tax_fixture_cases_match_registry_rules() {
    let fixtures = load_fixtures();

    for case in fixtures.stamp_tax {
        let target_date = parse_date(&case.input.date);
        let params = load_stamp_tax_params(target_date).unwrap();
        let document_code = case
            .input
            .document_code
            .parse::<StampTaxDocumentCode>()
            .unwrap();
        let flags = case
            .input
            .flags
            .iter()
            .map(|flag| flag.parse::<StampTaxFlag>().unwrap())
            .collect::<HashSet<_>>();

        let context = StampTaxContext {
            document_code,
            stated_amount: case.input.stated_amount,
            target_date,
            flags,
            policy: Box::new(StandardNtaPolicy),
        };

        let result = calculate_stamp_tax(&context, &params).unwrap();

        assert_eq!(
            result.tax_amount.as_yen(),
            case.expected.tax_amount,
            "{}: tax_amount",
            case.id
        );
        assert_eq!(
            result.rule_label, case.expected.rule_label,
            "{}: rule_label",
            case.id
        );
        assert_eq!(
            result.applied_special_rule, case.expected.applied_special_rule,
            "{}: applied_special_rule",
            case.id
        );
    }
}
