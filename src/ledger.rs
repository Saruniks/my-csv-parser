use std::{collections::HashMap, error::Error};

use crate::types::{
    Centicents, ClientId, ClientState, Record, RecordType, TransactionId, TransactionInfo,
};

#[derive(Default, Clone)]
pub struct Ledger {
    transaction_data: HashMap<TransactionId, TransactionInfo>,
    clients_data: HashMap<ClientId, ClientState>,
}

impl Ledger {
    pub fn process(&mut self, records: Vec<Record>) -> Result<(), Box<dyn Error>> {
        for record in records {
            let client_state = self
                .clients_data
                .entry(ClientId(record.client_id))
                .or_default();

            if client_state.is_frozen {
                return Ok(());
            }

            match record.record_type {
                RecordType::Deposit => {
                    self.transaction_data.insert(
                        TransactionId(record.transaction_id),
                        TransactionInfo {
                            amount: Centicents::try_from(
                                record
                                    .amount
                                    .clone()
                                    .ok_or("Amount is missing for the deposit")?,
                            )?,
                            is_disputed: false,
                        },
                    );
                    client_state.balances.available += Centicents::try_from(
                        record.amount.ok_or("Amount is missing for the deposit")?,
                    )?
                }
                RecordType::Withdrawal => {
                    self.transaction_data.insert(
                        TransactionId(record.transaction_id),
                        TransactionInfo {
                            amount: Centicents::try_from(
                                record
                                    .amount
                                    .clone()
                                    .ok_or("Amount is missing for the withdrawal")?,
                            )?,
                            is_disputed: false,
                        },
                    );
                    if client_state.balances.available.0
                        - Centicents::try_from(
                            record
                                .amount
                                .clone()
                                .ok_or("Amount is missing for the withdrawal")?,
                        )?
                        .0
                        > 0
                    {
                        client_state.balances.available.0 -= Centicents::try_from(
                            record
                                .amount
                                .ok_or("Amount is missing for the withdrawal")?,
                        )?
                        .0;
                    }
                }
                RecordType::Dispute => {
                    if let Some(transaction_info) = self
                        .transaction_data
                        .get_mut(&TransactionId(record.transaction_id))
                    {
                        client_state.balances.available.0 -= transaction_info.amount.0;
                        client_state.balances.held.0 += transaction_info.amount.0;
                        transaction_info.is_disputed = true;
                    }
                }
                RecordType::Resolve => {
                    if let Some(transaction_info) = self
                        .transaction_data
                        .get_mut(&TransactionId(record.transaction_id))
                    {
                        client_state.balances.available.0 += transaction_info.amount.0;
                        client_state.balances.held.0 -= transaction_info.amount.0;
                        transaction_info.is_disputed = false;
                    }
                }
                RecordType::Chargeback => {
                    if let Some(transaction_info) = self
                        .transaction_data
                        .get_mut(&TransactionId(record.transaction_id))
                    {
                        client_state.is_frozen = true;
                        client_state.balances.held.0 -= transaction_info.amount.0;
                        client_state.balances.available.0 -= transaction_info.amount.0;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn print(&self) {
        eprintln!("{:#?}", self.clients_data);

        println!("client, available, held, total, locked");
        for (client, client_state) in self.clients_data.iter() {
            println!(
                "{}, {}, {}, {}, {}",
                client.0,
                client_state.balances.available,
                client_state.balances.held,
                client_state.balances.available.clone() - client_state.balances.held.clone(),
                client_state.is_frozen
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use common_macros::hash_map;

    use crate::types::Balances;

    use super::*;

    #[test]
    fn test_deposit() {
        let mut ledger = Ledger::default();
        ledger
            .process(vec![Record {
                record_type: RecordType::Deposit,
                client_id: 0,
                transaction_id: 0,
                amount: Some("10.5001".to_string()),
            }])
            .unwrap();

        assert_eq!(
            ledger.transaction_data,
            hash_map! {
                TransactionId(0) => TransactionInfo {
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
                    transaction_id: 0,
                    amount: Some("10.5001".to_string()),
                },
                Record {
                    record_type: RecordType::Dispute,
                    client_id: 0,
                    transaction_id: 0,
                    amount: None,
                },
            ])
            .unwrap();

        assert_eq!(
            ledger.transaction_data,
            hash_map! {
                TransactionId(0) => TransactionInfo {
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
                    transaction_id: 0,
                    amount: Some("10.5001".to_string()),
                },
                Record {
                    record_type: RecordType::Dispute,
                    client_id: 0,
                    transaction_id: 0,
                    amount: None,
                },
                Record {
                    record_type: RecordType::Resolve,
                    client_id: 0,
                    transaction_id: 0,
                    amount: None,
                },
            ])
            .unwrap();

        assert_eq!(
            ledger.transaction_data,
            hash_map! {
                TransactionId(0) => TransactionInfo {
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
                    transaction_id: 0,
                    amount: Some("10.5001".to_string()),
                },
                Record {
                    record_type: RecordType::Dispute,
                    client_id: 0,
                    transaction_id: 0,
                    amount: None,
                },
                Record {
                    record_type: RecordType::Chargeback,
                    client_id: 0,
                    transaction_id: 0,
                    amount: None,
                },
                // Even deposit is supposed to be no op at this point?
                Record {
                    record_type: RecordType::Deposit,
                    client_id: 0,
                    transaction_id: 1,
                    amount: Some("100.5001".to_string()),
                },
            ])
            .unwrap();

        assert_eq!(
            ledger.transaction_data,
            hash_map! {
                TransactionId(0) => TransactionInfo {
                    amount: Centicents(105_001),
                    is_disputed: true, // should this be false after chargeback?
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
                    transaction_id: 0,
                    amount: Some("10.5001".to_string()),
                },
                Record {
                    record_type: RecordType::Withdrawal,
                    client_id: 0,
                    transaction_id: 1,
                    amount: Some("10.5001".to_string()),
                },
                Record {
                    record_type: RecordType::Dispute,
                    client_id: 0,
                    transaction_id: 0,
                    amount: None,
                },
                Record {
                    record_type: RecordType::Chargeback,
                    client_id: 0,
                    transaction_id: 0,
                    amount: None,
                },
            ])
            .unwrap();

        assert_eq!(
            ledger.transaction_data,
            hash_map! {
                TransactionId(0) => TransactionInfo {
                    amount: Centicents(105_001),
                    is_disputed: true, // should this be false after chargeback?
                },
                TransactionId(1) => TransactionInfo {
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
