pub mod persistence;

pub fn hello_core() -> String {
    "Hello from tax_ctrl_core 🦀".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_core() {
        assert_eq!(hello_core(), "Hello from tax_ctrl_core 🦀");
    }
}
