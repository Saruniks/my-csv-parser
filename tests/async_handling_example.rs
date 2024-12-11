use std::sync::{Arc, Mutex};

use my_csv_parser::ledger::Ledger;

#[tokio::test]
#[ignore = "compilation only"]
async fn test_shared_state_and_async_handling() {
    // Use mutex inside of Ledger instead for smaller lock scope
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
