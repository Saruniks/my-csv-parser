use std::{env, error::Error};

use my_csv_parser::ledger::Ledger;
use my_csv_parser::types::Record;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args().collect();
    let file_path = if let Some(file_path) = args.get(1) {
        file_path
    } else {
        return Err("Missing argument".into());
    };

    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(file_path)?;

    let records: Vec<Record> = reader.deserialize().collect::<Result<_, _>>()?;

    let mut ledger = Ledger::default();
    ledger.process(records).unwrap();
    ledger.print();

    Ok(())
}
