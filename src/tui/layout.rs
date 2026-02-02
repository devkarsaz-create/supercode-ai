// helper layout utilities for TUI v0.1

pub enum Panel {
    Graph,
    Conversation,
    Logs,
}

impl Panel {
    pub fn titles() -> [&'static str; 3] {
        ["Graph", "Conversation", "Logs"]
    }
}
