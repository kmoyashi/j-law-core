use j_law_core::{LegalDate, RegistryError};

pub(crate) fn validate_history_periods<T, FFrom, FUntil>(
    domain: &str,
    path: &str,
    history: &[T],
    effective_from: FFrom,
    effective_until: FUntil,
) -> Result<(), RegistryError>
where
    FFrom: Fn(&T) -> &str,
    FUntil: Fn(&T) -> Option<&str>,
{
    if history.is_empty() {
        return Err(RegistryError::ParseError {
            path: path.into(),
            cause: "history must not be empty".into(),
        });
    }

    let mut sorted = history.iter().collect::<Vec<_>>();
    sorted.sort_by(|a, b| effective_from(*a).cmp(effective_from(*b)));

    for entry in &sorted {
        let from = effective_from(*entry);
        let from_date = parse_legal_date(domain, from)?;

        if let Some(until) = effective_until(*entry) {
            let until_date = parse_legal_date(domain, until)?;
            if from_date.to_date_str() > until_date.to_date_str() {
                return Err(RegistryError::PeriodOverlap {
                    domain: domain.into(),
                    from: from.into(),
                    until: until.into(),
                });
            }
        }
    }

    for [current, next] in sorted.array_windows::<2>() {
        let current = *current;
        let next = *next;
        let next_from = effective_from(next);

        let Some(current_until) = effective_until(current) else {
            return Err(RegistryError::PeriodOverlap {
                domain: domain.into(),
                from: next_from.into(),
                until: "open-ended".into(),
            });
        };

        if current_until >= next_from {
            return Err(RegistryError::PeriodOverlap {
                domain: domain.into(),
                from: next_from.into(),
                until: current_until.into(),
            });
        }

        let expected_next_from = parse_legal_date(domain, current_until)?
            .next_day()
            .to_date_str();
        if expected_next_from != next_from {
            return Err(RegistryError::PeriodGap {
                domain: domain.into(),
                end: current_until.into(),
                next_start: next_from.into(),
            });
        }
    }

    Ok(())
}

fn parse_legal_date(domain: &str, value: &str) -> Result<LegalDate, RegistryError> {
    LegalDate::from_date_str(value).ok_or_else(|| RegistryError::InvalidDateFormat {
        domain: domain.into(),
        value: value.into(),
    })
}
