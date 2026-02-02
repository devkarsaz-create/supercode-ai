use crate::types::AgentState;

pub fn transition(current: &AgentState, next: &AgentState) -> bool {
    use AgentState::*;
    match (current, next) {
        (AgentState::Idle, AgentState::Planning) => true,
        (AgentState::Planning, AgentState::Executing) => true,
        (AgentState::Executing, AgentState::Reviewing) => true,
        (AgentState::Reviewing, AgentState::Completed) => true,
        (_, AgentState::Failed) => true,
        _ => false,
    }
}
