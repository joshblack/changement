fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_basic_functionality() {
        // Basic test to ensure the project can run tests
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_string_operations() {
        let hello = "Hello, world!";
        assert!(hello.contains("Hello"));
        assert!(hello.contains("world"));
    }
}
