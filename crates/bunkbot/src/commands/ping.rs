use serenity::all::CreateCommand;

pub fn ping_command() -> CreateCommand {
    CreateCommand::new("ping").description("Ping bunkbot")
}

pub fn execute_ping() -> String {
    // Stub returning empty or wrong message for TDD phase 1
    "Wrong response".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_execute_ping() {
        assert_eq!(execute_ping(), "Pong from bunkbot!");
    }
}
