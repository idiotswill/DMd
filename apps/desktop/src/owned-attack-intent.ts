import type { Ability } from './table-api';
import type { AttackOptions, WeaponGrip, WeaponUseChoice } from './tactical-api';

type Weapon = AttackOptions['weapons'][number];
export interface AttackIntentChoices {
  target: string;
  weapon: string;
  ability: Ability | '';
  grip: string;
}
export interface AttackIntentDraft { text: string; choices: AttackIntentChoices }
export interface AttackIntentProposal {
  error: string | null;
  targets: AttackOptions['targets'];
  weapons: Weapon[];
  abilities: Ability[];
  grips: WeaponGrip[];
  target: string;
  weapon: string;
  ability: Ability | '';
  grip: string;
  choice: WeaponUseChoice | null;
}

export const emptyAttackChoices = (): AttackIntentChoices => ({ target: '', weapon: '', ability: '', grip: '' });
const normalize = (value: string) => value.trim().replace(/\s+/gu, ' ').toLowerCase();
export const gripKey = (grip: WeaponGrip) => JSON.stringify(grip);
export const gripLabel = (grip: WeaponGrip) => grip === 'TwoHands' ? 'Both hands' : `${grip.OneHand} hand`;

/** Filter existing offers by visible held slots, without granting equipment changes. */
export function heldAttackGrips(options: AttackOptions, weapon: Weapon): WeaponGrip[] {
  const slots = options.hands.hands;
  if (slots.length !== 2) return [];
  const holds = (slot: typeof slots[number]) => typeof slot === 'object' && slot.Item === weapon.item;
  return weapon.grips.filter(grip => grip === 'TwoHands'
    ? slots.some(holds) && slots.every(slot => slot === 'Free' || holds(slot))
    : holds(slots[grip.OneHand === 'Left' ? 0 : 1]));
}

export function heldMeleeWeapons(options: AttackOptions): Weapon[] {
  return options.weapons.filter(weapon => weapon.deliveries.includes('Melee')
    && weapon.purposes.includes('Normal') && !weapon.ammunition_required
    && !weapon.source_features?.length && weapon.abilities.length > 0
    && heldAttackGrips(options, weapon).length > 0);
}

/** A local proposal only. The normal Attack resolver still admits source, cost and target. */
export function proposeOwnedAttack(text: string, options: AttackOptions, choices: AttackIntentChoices): AttackIntentProposal {
  const result: AttackIntentProposal = {
    error: null, targets: [], weapons: [], abilities: [], grips: [],
    target: '', weapon: '', ability: '', grip: '', choice: null,
  };
  const fail = (error: string) => ({ ...result, error });
  if (!text.trim() || text.length > 512 || /[\r\n]/u.test(text)) {
    return fail('Describe one melee attack in a single short sentence.');
  }
  const sentence = normalize(text).replace(/[.!]$/u, '');
  const match = /^(?:i )?(?:attack|strike|hit)(?: (.+))?$/u.exec(sentence);
  if (!match) return fail('Describe an attack you are taking, such as “I attack the guard with my sword”. Questions and other actions need the table or their existing controls.');
  const remainder = match[1] ?? '';
  const held = heldMeleeWeapons(options);
  const pairs: { target: string; weapon: string }[] = [];
  // Enumerate entire visible labels, never split at "with" or strip words from names.
  // Multiple complete parses retain all material alternatives; there is no first match.
  for (const target of options.targets) {
    const label = normalize(target.label);
    if (!label || target.actor === options.actor) continue;
    const targetPhrases = new Set([label, `the ${label}`]);
    for (const weapon of held) {
      const name = normalize(weapon.name);
      if (!name) continue;
      if (!remainder || [...targetPhrases].some(phrase => remainder === phrase
        || remainder === `${phrase} with ${name}` || remainder === `${phrase} with my ${name}`)) {
        pairs.push({ target: target.actor, weapon: weapon.item });
      }
    }
  }
  if (!pairs.length) return fail('Use the complete displayed target and held weapon names for one melee attack. Clarify any extra action separately; no part of this declaration has been sent.');
  result.targets = options.targets.filter(target => pairs.some(pair => pair.target === target.actor));
  if (choices.target && !result.targets.some(target => target.actor === choices.target)) return fail('The selected target is no longer part of this declaration. Review it again.');
  // An omitted target is an unanswered question even when the view offers only one.
  result.target = choices.target || (remainder && result.targets.length === 1 ? result.targets[0].actor : '');
  if (!result.target) return result;
  result.weapons = held.filter(weapon => pairs.some(pair => pair.target === result.target && pair.weapon === weapon.item));
  if (choices.weapon && !result.weapons.some(weapon => weapon.item === choices.weapon)) return fail('The selected held weapon is no longer part of this declaration. Review it again.');
  result.weapon = choices.weapon || (result.weapons.length === 1 ? result.weapons[0].item : '');
  const weapon = result.weapons.find(candidate => candidate.item === result.weapon);
  if (!weapon) return result;
  result.abilities = [...new Set(weapon.abilities)];
  result.grips = heldAttackGrips(options, weapon);
  if (choices.ability && !result.abilities.includes(choices.ability)) return fail('The selected attack ability changed. Review the declaration again.');
  if (choices.grip && !result.grips.some(grip => gripKey(grip) === choices.grip)) return fail('The selected held grip changed. Review the declaration again.');
  result.ability = choices.ability || (result.abilities.length === 1 ? result.abilities[0] : '');
  result.grip = choices.grip || (result.grips.length === 1 ? gripKey(result.grips[0]) : '');
  const grip = result.grips.find(candidate => gripKey(candidate) === result.grip);
  if (result.ability && grip) result.choice = {
    target: result.target, weapon: result.weapon, delivery: 'Melee', ability: result.ability,
    grip, purpose: 'Normal', ammunition: null, equipment_change: null,
  };
  return result;
}
