pub(crate) fn run_command(cmd: &[String]) {
    use std::process::Command;

    let status = Command::new(&cmd[0])
        .args(&cmd[1..])
        .status()
        .unwrap_or_else(|e| panic!("failed to execute process (maybe tools is missing?): {}", e));

    if !status.success() {
        println!("build failed with code {}:\n\n{}", status.code().unwrap_or(-1), status);
    }
}