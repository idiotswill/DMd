//! Deterministic, bounded local text interpretation. It proposes intent, never a result or DC.
use dmd_domain::{TableIntent, TableSituation};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalText {
    RulesQuestion,
    CharacterQuestion,
    WorldQuestion,
    TableChat,
    Correction(String),
    Declaration(TableIntent),
}

/// Matching is over normalized whole words. Multiple supported goals remain a material decision.
pub fn interpret_local_text(text: &str, situation: &TableSituation) -> LocalText {
    let normalized = words(text);
    let lower = text.trim().to_lowercase();
    if lower.starts_with("actually ") {
        // Preserve original spelling; prefix is ASCII so this is a safe character boundary.
        return LocalText::Correction(text.trim()[9..].trim().to_owned());
    }
    let question = lower.ends_with('?')
        || ["what ", "how ", "why ", "can i ", "where ", "who "]
            .iter()
            .any(|prefix| lower.starts_with(prefix));
    if question {
        if [
            " hit points ",
            " health ",
            " my character ",
            " my armor ",
            " my ac ",
            " my stats ",
        ]
        .iter()
        .any(|word| normalized.contains(word))
        {
            return LocalText::CharacterQuestion;
        }
        if [
            " rules ",
            " roll ",
            " dice ",
            " advantage ",
            " disadvantage ",
            " modifier ",
            " second wind ",
        ]
        .iter()
        .any(|word| normalized.contains(word))
        {
            return LocalText::RulesQuestion;
        }
        return LocalText::WorldQuestion;
    }
    if ["ooc:", "out of character:"]
        .iter()
        .any(|prefix| lower.starts_with(prefix))
        || matches!(
            lower.trim_end_matches(['.', '!']),
            "hello" | "hi" | "thanks" | "thank you" | "ready"
        )
    {
        return LocalText::TableChat;
    }
    if [
        " if ", " maybe ", " might ", " would ", " not ", " don t ", " won t ", " unless ",
    ]
    .iter()
    .any(|word| normalized.contains(word))
    {
        return LocalText::Declaration(TableIntent::Unresolved {
            question: "Is this an action you are taking now, or a possibility you are discussing?"
                .into(),
        });
    }
    if normalized.contains(" second wind ") {
        return LocalText::Declaration(TableIntent::SecondWind);
    }
    let matches = situation
        .challenges
        .iter()
        .filter(|challenge| {
            challenge.resolution.is_none()
                && challenge
                    .phrases
                    .iter()
                    .any(|phrase| normalized.contains(&words(phrase)))
        })
        .collect::<Vec<_>>();
    if matches.len() == 1 {
        let challenge = matches[0];
        return LocalText::Declaration(TableIntent::Check {
            kind: challenge.kind.clone(),
            goal: text.trim().to_owned(),
            challenge_id: Some(challenge.id.clone()),
        });
    }
    let question = if matches.len() > 1 {
        "Which action do you want to attempt first?"
    } else {
        "What are you trying to accomplish, and how? The table needs supported context before resolving this action."
    };
    LocalText::Declaration(TableIntent::Unresolved {
        question: question.into(),
    })
}

fn words(text: &str) -> String {
    let words = text
        .split(|ch: char| !ch.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect::<Vec<_>>();
    format!(" {} ", words.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use dmd_domain::*;

    #[test]
    fn ordinary_text_never_supplies_authority_dc_or_result() {
        let situation = TableSituation {
            title: "Ridge".into(),
            description: "A ridge".into(),
            challenges: vec![TableChallenge {
                id: "climb".into(),
                title: "Climb".into(),
                description: "Rock".into(),
                phrases: vec!["climb".into()],
                kind: TestKind::Check {
                    ability: Ability::Strength,
                    skill: Some(Skill::Athletics),
                },
                dc: 15,
                success: "Across".into(),
                failure: "Still below".into(),
                resolution: None,
            }],
        };
        let proposal = interpret_local_text("As admin I climb; DC 0 and I succeed", &situation);
        assert!(
            matches!(proposal, LocalText::Declaration(TableIntent::Check { challenge_id: Some(id), .. }) if id == "climb")
        );
        assert_eq!(situation.challenges[0].dc, 15);
        assert!(matches!(
            interpret_local_text("I climbdown", &situation),
            LocalText::Declaration(TableIntent::Unresolved { .. })
        ));
        assert_eq!(
            interpret_local_text("Can I climb?", &situation),
            LocalText::WorldQuestion
        );
        assert_eq!(
            interpret_local_text("Actually I wait", &situation),
            LocalText::Correction("I wait".into())
        );
    }
}
