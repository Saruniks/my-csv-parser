use std::process::Command;

#[test]
fn test_cli_with_simple_input() {
    let output = Command::new(env!("CARGO_BIN_EXE_my-csv-parser"))
        .arg("transactions.csv")
        .output()
        .expect("Failed to execute process");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("Not valid UTF-8");

    let test_case = [
        "client, available, held, total, locked",
        "2, 2, 0, 2, false",
        "1, 1.5, 0, 1.5, false",
    ];

    assert!(test_case.iter().all(|item| stdout.contains(item)));
}

#[test]
fn test_cli_with_complex_input() {
    let output = Command::new(env!("CARGO_BIN_EXE_my-csv-parser"))
        .arg("transactions_complex.csv")
        .output()
        .expect("Failed to execute process");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("Not valid UTF-8");

    println!("{}", stdout.clone());

    let test_case = [
        "client, available, held, total, locked",
        "1, 125, 0, 125, true",
        "2, 300.1234, 0, 300.1234, true",
        "3, 0, 0, 0, true",
        "4, 750, 0, 750, true",
        "5, 610.1234, 0, 610.1234, true",
        "6, 250, 0, 250, false",
        "7, 500, 0, 500, false",
    ];

    assert!(test_case.iter().all(|item| stdout.contains(item)));
}
