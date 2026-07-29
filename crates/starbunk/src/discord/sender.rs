#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SenderCategory {
    User,
    Bot,
    E2eUser,
    E2eBot,
}

impl SenderCategory {
    pub fn is_bot(&self) -> bool {
        matches!(self, SenderCategory::Bot | SenderCategory::E2eBot)
    }

    pub fn is_human(&self) -> bool {
        matches!(self, SenderCategory::User | SenderCategory::E2eUser)
    }
}
