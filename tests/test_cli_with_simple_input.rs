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
