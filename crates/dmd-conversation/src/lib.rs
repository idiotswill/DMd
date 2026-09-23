use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UtteranceKind {
    InWorldAction,
    InCharacterDialogue,
    RulesQuery,
    WorldQuery,
    CharacterQuery,
    DiceResult,
    Correction,
    TableChat,
    AdminCommand,
    Ambiguous,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterpretedUtterance {
    pub raw_text: String,
    pub kind: UtteranceKind,
    pub confidence: f32,
    pub requires_confirmation: bool,
}

impl InterpretedUtterance {
    pub fn should_interrupt_for_confirmation(&self) -> bool {
        self.requires_confirmation || matches!(self.kind, UtteranceKind::Ambiguous)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ambiguity_requires_confirmation() {
        let u = InterpretedUtterance {
            raw_text: "maybe I open it".into(),
            kind: UtteranceKind::Ambiguous,
            confidence: 0.5,
            requires_confirmation: false,
        };
        assert!(u.should_interrupt_for_confirmation());
    }
}
