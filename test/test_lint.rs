#[cfg(test)]
mod lint_test {
    use std::process::Command;

    #[test]
    fn test_all() {
        let mut process = Command::new("cargo");
        let output = process
            .arg("run")
            .arg("--bin")
            .arg("lc3lint")
            .arg("test/data/expected/lint/all.asm")
            .output();
        assert!(output.is_ok());
        assert!(output.unwrap().status.success());
    }
}
