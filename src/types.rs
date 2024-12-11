use std::{
    error::Error,
    fmt::Display,
    ops::{AddAssign, Sub},
};

use serde_derive::Deserialize;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct ClientId(pub u16);

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ClientState {
    pub balances: Balances,
    pub is_frozen: bool,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Balances {
    pub available: Centicents,
    pub held: Centicents,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct TransactionId(pub u32);

#[derive(Clone, PartialEq, Debug)]
pub struct TransactionInfo {
    pub amount: Centicents,
    pub is_disputed: bool,
}

#[derive(Debug, Deserialize, Default, Clone, PartialEq)]
pub struct Centicents(pub i64);

impl AddAssign for Centicents {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0 + rhs.0;
    }
}

impl Sub for Centicents {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

// Better way to do it?
impl TryFrom<String> for Centicents {
    type Error = Box<dyn Error>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mut centicents = 0;
        let values: Vec<_> = value.split_terminator(".").collect();

        if values.len() > 2 {
            return Err("Failed to parse centicents".into());
        }

        if let Some(whole) = values.first() {
            centicents += whole.parse::<i64>()? * 10000; // Try from...
        }
        if let Some(fractional) = values.get(1) {
            let mut multiplier = 1000;
            for index in 0..4 {
                if let Some(value) = fractional.chars().nth(index) {
                    let value = value
                        .to_digit(10)
                        .ok_or("Failed to convert fraction to digit")?
                        as i64; // Is this correct
                    centicents += value * multiplier;
                } else {
                    break;
                }
                multiplier /= 10;
            }
        }
        Ok(Centicents(centicents))
    }
}

impl Display for Centicents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0 as f64 / 10000.0)?;
        Ok(())
    }
}

#[derive(Deserialize, Debug)]
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
    pub transaction_id: u32,
    pub amount: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_centicents_conversion() {
        assert_eq!(
            Centicents::try_from("1.2345".to_string()).unwrap(),
            Centicents(12345)
        );
        assert_eq!(
            Centicents::try_from("1.234".to_string()).unwrap(),
            Centicents(12340)
        );
        assert_eq!(
            Centicents::try_from("1.23".to_string()).unwrap(),
            Centicents(12300)
        );
        assert_eq!(
            Centicents::try_from("1.2".to_string()).unwrap(),
            Centicents(12000)
        );
        assert_eq!(
            Centicents::try_from("1".to_string()).unwrap(),
            Centicents(10000)
        );
        assert_eq!(
            Centicents::try_from("1.".to_string()).unwrap(),
            Centicents(10000)
        );
        assert_eq!(
            Centicents::try_from("1.0".to_string()).unwrap(),
            Centicents(10000)
        );
        assert_eq!(
            Centicents::try_from("1.00".to_string()).unwrap(),
            Centicents(10000)
        );
        assert_eq!(
            Centicents::try_from("1.000".to_string()).unwrap(),
            Centicents(10000)
        );
        assert_eq!(
            Centicents::try_from("1.0000".to_string()).unwrap(),
            Centicents(10000)
        );
        assert_eq!(
            Centicents::try_from("1.00000".to_string()).unwrap(),
            Centicents(10000)
        );
        assert_eq!(
            Centicents::try_from("1.000000".to_string()).unwrap(),
            Centicents(10000)
        );
        assert_eq!(
            Centicents::try_from("1.0000000".to_string()).unwrap(),
            Centicents(10000)
        );
        assert_eq!(
            Centicents::try_from("2.0001".to_string()).unwrap(),
            Centicents(20001)
        );
        assert_eq!(
            Centicents::try_from("0.0001".to_string()).unwrap(),
            Centicents(1)
        );
        assert_eq!(
            Centicents::try_from("0.08".to_string()).unwrap(),
            Centicents(800)
        );
        assert_eq!(
            Centicents::try_from("0015".to_string()).unwrap(),
            Centicents(150000)
        );
        assert_eq!(
            Centicents::try_from("521321.0001".to_string()).unwrap(),
            Centicents(5213210001)
        );
    }

    #[test]
    fn err_centicents_conversion() {
        assert!(Centicents::try_from(".234".to_string()).is_err());
        assert!(Centicents::try_from("5.234.555".to_string()).is_err());
        assert!(Centicents::try_from("awbawd".to_string()).is_err());
    }
}
