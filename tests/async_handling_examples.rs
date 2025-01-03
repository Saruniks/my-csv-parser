use std::sync::{Arc, Mutex};

use csv_async::{AsyncReaderBuilder, Trim};
use futures_util::stream::StreamExt;
use my_csv_parser::{ledger::Ledger, types::Record};
use tokio::{fs::File, io::BufReader};
use tokio_util::compat::TokioAsyncReadCompatExt;

#[tokio::test]
#[ignore = "compilation only"]
async fn test_shared_state_and_async_handling() {
    let ledger = Arc::new(Mutex::new(Ledger::default()));

    let mut tasks = Vec::new();
    for _ in 0..100 {
        let ledger = ledger.clone();
        let task = tokio::spawn(async move {
            ledger.lock().unwrap().process(vec![]).unwrap();
        });
        tasks.push(task);
    }

    for task in tasks {
        task.await.unwrap();
    }

    ledger.lock().unwrap().print();
}

#[tokio::test]
#[ignore = "compilation only"]
async fn test_streamed_concurrent_handling() {
    // For running concurrent streams different chronology requirements are needed:
    // single file (file-scope) vs across multiple files (inter-file).
    let files = vec!["transactions.csv", "transactions_complex.csv"];

    let ledger = Arc::new(Mutex::new(Ledger::default()));

    let mut handles = Vec::new();

    for filename in files {
        let ledger_clone = ledger.clone();
        let filename = filename.to_string();

        let handle = tokio::spawn(async move {
            let file = File::open(&filename).await.unwrap();
            let reader = BufReader::new(file);
            let compat_reader = reader.compat();
            let mut csv_reader = AsyncReaderBuilder::new()
                .trim(Trim::All)
                .create_deserializer(compat_reader);

            let mut records = csv_reader.deserialize::<Record>();

            while let Some(result) = records.next().await {
                let record = result.unwrap();

                let mut ledger = ledger_clone.lock().unwrap();
                ledger.process_record(record).unwrap();
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let ledger = ledger.lock().unwrap();
    ledger.print();
}
