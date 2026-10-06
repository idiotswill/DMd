import type { Id } from './table-api';

type TacticalPrompt =
  | { kind: 'shove'; key: Id; actor: Id; stage: 'SaveChoice' | 'OutcomeChoice' | 'PushReview' }
  | { kind: 'hit-order' | 'missile-order'; key: Id }
  | { kind: 'hit-response' | 'missile-response'; key: Id; actor: Id; selected: boolean }
  | { kind: 'initiative-tie'; total: number }
  | { kind: 'attack-decision'; actor: Id; choice: 'Knockout' | 'Graze' }
  | { kind: 'opportunity'; actor: Id; target: Id }
  | { kind: 'liquid-landing' | 'saving-throw-choice' | 'legendary-resistance' | 'legendary-action'; actor: Id };

// UI identity only: decisions and authority still come from the saved projection.
// Tuples avoid collisions between prompt kinds, actors and opaque response keys.
export function tacticalPromptIdentity(prompt: TacticalPrompt): string {
  switch (prompt.kind) {
    case 'shove': return JSON.stringify([prompt.kind, prompt.actor, prompt.key, prompt.stage]);
    case 'hit-order': case 'missile-order': return JSON.stringify([prompt.kind, prompt.key]);
    case 'hit-response': case 'missile-response': return JSON.stringify([prompt.kind, prompt.actor, prompt.key, prompt.selected]);
    case 'initiative-tie': return JSON.stringify([prompt.kind, prompt.total]);
    case 'attack-decision': return JSON.stringify([prompt.kind, prompt.actor, prompt.choice]);
    case 'opportunity': return JSON.stringify([prompt.kind, prompt.actor, prompt.target]);
    default: return JSON.stringify([prompt.kind, prompt.actor]);
  }
}
