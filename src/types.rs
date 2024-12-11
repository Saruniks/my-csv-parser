use std::{
    fmt::Display,
    ops::{AddAssign, Sub, SubAssign},
    str::FromStr,
};

use anyhow::{anyhow, bail, Context, Error};
use serde_derive::Deserialize;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct ClientId(pub u16);

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Balances {
    pub available: Centicents,
    pub held: Centicents,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ClientState {
    pub balances: Balances,
    pub is_frozen: bool,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct TxId(pub u32);

#[derive(Clone, PartialEq, Debug)]
pub struct TxInfo {
    pub amount: Centicents,
    pub is_disputed: bool,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy, Default)]
pub struct Centicents(pub i64);

impl AddAssign for Centicents {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for Centicents {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for Centicents {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl FromStr for Centicents {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.trim();
        if value.is_empty() {
            bail!("Input is empty, cannot parse Centicents");
        }

        let parts: Vec<&str> = value.split('.').collect();
        if parts.len() > 2 {
            bail!("Invalid format, multiple decimal points found in '{}'", value);
        }

        // Parse the whole part
        let whole_str = parts[0];
        if whole_str.is_empty() {
            bail!("No whole part provided. For example, '.234' is invalid input");
        }
        let whole = whole_str
            .parse::<i64>()
            .with_context(|| format!("Failed to parse whole part '{}'", whole_str))?;

        let mut centicents = whole * 10000;

        // Parse the fractional part if it exists
        if let Some(frac_str) = parts.get(1) {
            let frac_str = frac_str.trim();
            let frac_chars: Vec<char> = frac_str.chars().take(4).collect();

            let mut multiplier = 1000; // For the first fractional digit
            for ch in frac_chars {
                let digit = ch
                    .to_digit(10)
                    .ok_or_else(|| anyhow!("Non-digit character '{}' in fractional part", ch))?;

                centicents += (digit as i64) * multiplier;
                multiplier /= 10;
            }
        }

        Ok(Centicents(centicents))
    }
}

impl Display for Centicents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = self.0 as f64 / 10000.0;
        write!(f, "{}", value)
    }
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RecordType {
    // Using alias would be more resilient?
    // #[serde(alias = "deposit", alias = "DEPOSIT", alias = "Deposit")]
    Deposit,
    // #[serde(alias = "withdrawal", alias = "WITHDRAWAL", alias = "Withdrawal")]
    Withdrawal,
    // #[serde(alias<...>)]
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Deserialize, Debug)]
pub struct Record {
    #[serde(rename = "type")]
    pub record_type: RecordType,
    #[serde(rename = "client")]
    pub client_id: u16,
    #[serde(rename = "tx")]
    pub tx_id: u32,
    pub amount: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_centicents_conversion() {
        assert_eq!(Centicents::from_str("1.2345").unwrap(), Centicents(12345));
        assert_eq!(Centicents::from_str("1.234").unwrap(), Centicents(12340));
        assert_eq!(Centicents::from_str("1.23").unwrap(), Centicents(12300));
        assert_eq!(Centicents::from_str("1.2").unwrap(), Centicents(12000));
        assert_eq!(Centicents::from_str("1").unwrap(), Centicents(10000));
        assert_eq!(Centicents::from_str("1.").unwrap(), Centicents(10000));
        assert_eq!(Centicents::from_str("1.0").unwrap(), Centicents(10000));
        assert_eq!(Centicents::from_str("1.00").unwrap(), Centicents(10000));
        assert_eq!(Centicents::from_str("1.000").unwrap(), Centicents(10000));
        assert_eq!(Centicents::from_str("1.0000").unwrap(), Centicents(10000));
        assert_eq!(Centicents::from_str("1.00000").unwrap(), Centicents(10000));
        assert_eq!(Centicents::from_str("1.000000").unwrap(), Centicents(10000));
        assert_eq!(Centicents::from_str("1.0000000").unwrap(), Centicents(10000));
        assert_eq!(Centicents::from_str("2.0001").unwrap(), Centicents(20001));
        assert_eq!(Centicents::from_str("0.0001").unwrap(), Centicents(1));
        assert_eq!(Centicents::from_str("0.08").unwrap(), Centicents(800));
        assert_eq!(Centicents::from_str("0015").unwrap(), Centicents(150000));
        assert_eq!(Centicents::from_str("521321.0001").unwrap(), Centicents(5213210001));
    }

    #[test]
    fn err_centicents_conversion() {
        assert!(Centicents::from_str(".234").is_err());
        assert!(Centicents::from_str("5.234.555").is_err());
        assert!(Centicents::from_str("awbawd").is_err());
    }
}
