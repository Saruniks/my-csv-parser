use anyhow::{Context, Result};
use std::{collections::HashMap, str::FromStr};

use crate::types::{Centicents, ClientId, ClientState, Record, RecordType, TxId, TxInfo};

#[derive(Default, Clone)]
pub struct Ledger {
    tx_data: HashMap<TxId, TxInfo>,
    clients_data: HashMap<ClientId, ClientState>,
}

impl Ledger {
    pub fn process(&mut self, records: Vec<Record>) -> Result<()> {
        for record in records {
            let client_state = self.clients_data.entry(ClientId(record.client_id)).or_default();

            if client_state.is_frozen {
                continue;
            }

            match record.record_type {
                RecordType::Deposit => {
                    let amount_str = record.amount.context("Amount is missing for the deposit")?;
                    let amount = Centicents::from_str(&amount_str)?;

                    self.tx_data.insert(
                        TxId(record.tx_id),
                        TxInfo {
                            amount,
                            is_disputed: false,
                        },
                    );
                    client_state.balances.available += amount;
                }
                RecordType::Withdrawal => {
                    let amount_str = record.amount.context("Amount is missing for the withdrawal")?;
                    let amount = Centicents::from_str(&amount_str)?;

                    self.tx_data.insert(
                        TxId(record.tx_id),
                        TxInfo {
                            amount,
                            is_disputed: false,
                        },
                    );

                    // Only withdraw if there's sufficient available balance
                    if client_state.balances.available.0 >= amount.0 {
                        client_state.balances.available -= amount;
                    }
                }
                RecordType::Dispute => {
                    if let Some(tx_info) = self.tx_data.get_mut(&TxId(record.tx_id)) {
                        client_state.balances.available -= tx_info.amount;
                        client_state.balances.held += tx_info.amount;
                        tx_info.is_disputed = true;
                    }
                }
                RecordType::Resolve => {
                    if let Some(tx_info) = self.tx_data.get_mut(&TxId(record.tx_id)) {
                        if tx_info.is_disputed {
                            client_state.balances.available += tx_info.amount;
                            client_state.balances.held -= tx_info.amount;
                            tx_info.is_disputed = false;
                        }
                    }
                }
                RecordType::Chargeback => {
                    if let Some(tx_info) = self.tx_data.get_mut(&TxId(record.tx_id)) {
                        client_state.is_frozen = true;
                        client_state.balances.held -= tx_info.amount;
                    }
                }
            }
        }
        Ok(())
    }

    /// Print the current state of all clients in CSV format.
    pub fn print(&self) {
        println!("client, available, held, total, locked");
        for (client, client_state) in self.clients_data.iter() {
            let total_balance = client_state.balances.available - client_state.balances.held;

            println!(
                "{}, {}, {}, {}, {}",
                client.0,
                client_state.balances.available,
                client_state.balances.held,
                total_balance,
                client_state.is_frozen
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use common_macros::hash_map;

    use super::*;
    use crate::types::Balances;

    #[test]
    fn test_deposit() {
        let mut ledger = Ledger::default();
        ledger
            .process(vec![Record {
                record_type: RecordType::Deposit,
                client_id: 0,
                tx_id: 0,
                amount: Some("10.5001".to_string()),
            }])
            .unwrap();

        assert_eq!(
            ledger.tx_data,
            hash_map! {
                TxId(0) => TxInfo {
                    amount: Centicents(105_001),
                    is_disputed: false,
                }
            }
        );

        assert_eq!(
            ledger.clients_data,
            hash_map! {
                ClientId(0) => ClientState {
                    balances: Balances {
                        available: Centicents(105_001),
                        held: Centicents(0)
                    },
                    is_frozen: false,
                }
            }
        )
    }

    #[test]
    fn test_dispute() {
        let mut ledger = Ledger::default();
        ledger
            .process(vec![
                Record {
                    record_type: RecordType::Deposit,
                    client_id: 0,
                    tx_id: 0,
                    amount: Some("10.5001".to_string()),
                },
                Record {
                    record_type: RecordType::Dispute,
                    client_id: 0,
                    tx_id: 0,
                    amount: None,
                },
            ])
            .unwrap();

        assert_eq!(
            ledger.tx_data,
            hash_map! {
                TxId(0) => TxInfo {
                    amount: Centicents(105_001),
                    is_disputed: true,
                }
            }
        );

        assert_eq!(
            ledger.clients_data,
            hash_map! {
                ClientId(0) => ClientState {
                    balances: Balances {
                        available: Centicents(0),
                        held: Centicents(105_001)
                    },
                    is_frozen: false,
                }
            }
        )
    }

    #[test]
    fn test_resolve() {
        let mut ledger = Ledger::default();
        ledger
            .process(vec![
                Record {
                    record_type: RecordType::Deposit,
                    client_id: 0,
                    tx_id: 0,
                    amount: Some("10.5001".to_string()),
                },
                Record {
                    record_type: RecordType::Dispute,
                    client_id: 0,
                    tx_id: 0,
                    amount: None,
                },
                Record {
                    record_type: RecordType::Resolve,
                    client_id: 0,
                    tx_id: 0,
                    amount: None,
                },
            ])
            .unwrap();

        assert_eq!(
            ledger.tx_data,
            hash_map! {
                TxId(0) => TxInfo {
                    amount: Centicents(105_001),
                    is_disputed: false,
                }
            }
        );

        assert_eq!(
            ledger.clients_data,
            hash_map! {
                ClientId(0) => ClientState {
                    balances: Balances {
                        available: Centicents(105_001),
                        held: Centicents(0)
                    },
                    is_frozen: false,
                }
            }
        )
    }

    #[test]
    fn test_chargeback() {
        let mut ledger = Ledger::default();
        ledger
            .process(vec![
                Record {
                    record_type: RecordType::Deposit,
                    client_id: 0,
                    tx_id: 0,
                    amount: Some("10.5001".to_string()),
                },
                Record {
                    record_type: RecordType::Dispute,
                    client_id: 0,
                    tx_id: 0,
                    amount: None,
                },
                Record {
                    record_type: RecordType::Chargeback,
                    client_id: 0,
                    tx_id: 0,
                    amount: None,
                },
                // Even deposit is supposed to be no-op at this point
                Record {
                    record_type: RecordType::Deposit,
                    client_id: 0,
                    tx_id: 1,
                    amount: Some("100.5001".to_string()),
                },
            ])
            .unwrap();

        assert_eq!(
            ledger.tx_data,
            hash_map! {
                TxId(0) => TxInfo {
                    amount: Centicents(105_001),
                    is_disputed: true,
                }
            }
        );

        assert_eq!(
            ledger.clients_data,
            hash_map! {
                ClientId(0) => ClientState {
                    balances: Balances {
                        available: Centicents(0),
                        held: Centicents(0)
                    },
                    is_frozen: true,
                }
            }
        )
    }

    #[test]
    fn test_negative_balance_chargeback() {
        let mut ledger = Ledger::default();
        ledger
            .process(vec![
                Record {
                    record_type: RecordType::Deposit,
                    client_id: 0,
                    tx_id: 0,
                    amount: Some("10.5001".to_string()),
                },
                Record {
                    record_type: RecordType::Withdrawal,
                    client_id: 0,
                    tx_id: 1,
                    amount: Some("10.5001".to_string()),
                },
                Record {
                    record_type: RecordType::Dispute,
                    client_id: 0,
                    tx_id: 0,
                    amount: None,
                },
                Record {
                    record_type: RecordType::Chargeback,
                    client_id: 0,
                    tx_id: 0,
                    amount: None,
                },
            ])
            .unwrap();

        assert_eq!(
            ledger.tx_data,
            hash_map! {
                TxId(0) => TxInfo {
                    amount: Centicents(105_001),
                    is_disputed: true,
                },
                TxId(1) => TxInfo {
                    amount: Centicents(105_001),
                    is_disputed: false,
                }
            }
        );

        assert_eq!(
            ledger.clients_data,
            hash_map! {
                ClientId(0) => ClientState {
                    balances: Balances {
                        available: Centicents(-105_001),
                        held: Centicents(0)
                    },
                    is_frozen: true,
                }
            }
        )
    }
}
