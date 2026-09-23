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

/// One typed interpretation candidate produced by a conversation provider.
///
/// `T` is an application-defined proposal type. Provider-specific JSON must be parsed into `T`
/// before the proposal can be handed to application/core validation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntentCandidate<T> {
    pub proposal: T,
    pub confidence: f32,
    pub requires_confirmation: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterpretedUtterance<T> {
    pub raw_text: String,
    pub kind: UtteranceKind,
    pub candidates: Vec<IntentCandidate<T>>,
}

impl<T> InterpretedUtterance<T> {
    #[must_use]
    pub fn should_interrupt_for_confirmation(&self) -> bool {
        matches!(self.kind, UtteranceKind::Ambiguous)
            || self
                .candidates
                .iter()
                .any(|candidate| candidate.requires_confirmation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    enum TestProposal {
        OpenDoor,
    }

    #[test]
    fn ambiguity_requires_confirmation_even_without_candidate() {
        let utterance = InterpretedUtterance::<TestProposal> {
            raw_text: "maybe I open it".into(),
            kind: UtteranceKind::Ambiguous,
            candidates: vec![],
        };
        assert!(utterance.should_interrupt_for_confirmation());
    }

    #[test]
    fn candidate_can_require_confirmation() {
        let utterance = InterpretedUtterance {
            raw_text: "I open that door".into(),
            kind: UtteranceKind::InWorldAction,
            candidates: vec![IntentCandidate {
                proposal: TestProposal::OpenDoor,
                confidence: 0.72,
                requires_confirmation: true,
            }],
        };

        assert!(utterance.should_interrupt_for_confirmation());
    }
}
