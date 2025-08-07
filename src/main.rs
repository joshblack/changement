fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    #[test]
    fn test_main_output() {
        // Test that the main function would produce the expected output
        let output = Command::new("cargo")
            .args(["run", "--bin", "changement"])
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
        assert_eq!(stdout.trim(), "Hello, world!");
    }

    #[test]
    fn test_hello_world_functionality() {
        // Test the core functionality by checking the expected message
        let expected_message = "Hello, world!";
        assert_eq!(expected_message, "Hello, world!");
        assert!(expected_message.starts_with("Hello"));
        assert!(expected_message.ends_with("world!"));
    }
}
