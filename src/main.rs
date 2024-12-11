use anyhow::{Context, Result};
use my_csv_parser::ledger::Ledger;
use my_csv_parser::types::Record;
use std::env;

fn get_file_path() -> Result<String> {
    let args: Vec<_> = env::args().collect();
    args.get(1).map(|s| s.to_string()).context("Missing file path argument")
}

fn read_records(file_path: &str) -> Result<Vec<Record>> {
    let mut reader = csv::ReaderBuilder::new().trim(csv::Trim::All).from_path(file_path)?;

    Ok(reader.deserialize().collect::<Result<_, _>>()?)
}

fn main() -> Result<()> {
    let file_path = get_file_path()?;
    let records = read_records(&file_path)?;

    let mut ledger = Ledger::default();
    ledger.process(records)?;
    ledger.print();

    Ok(())
}
