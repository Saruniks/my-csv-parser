// Can you stream values
// through memory as opposed to loading the entire data set upfront?

use std::sync::{Arc, Mutex};

use my_csv_parser::ledger::Ledger;

// What if your code was bundled in a server, and these CSVs came from
// thousands of concurrent TCP streams?
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
