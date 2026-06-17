use serenity::all::CreateCommand;

pub fn ping_command() -> CreateCommand {
    CreateCommand::new("ping").description("Ping bunkbot")
}

pub fn execute_ping() -> String {
    "Pong from bunkbot!".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_ping() {
        assert_eq!(execute_ping(), "Pong from bunkbot!");
    }
}
