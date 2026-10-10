import { expect, it } from 'vitest';
import { distinguishableAttackTargets, ownedAttackContext } from './owned-attack-context';
import { emptyAttackChoices, gripKey, heldAttackGrips, heldMeleeWeapons, proposeOwnedAttack } from './owned-attack-intent';
import { heldOptions, ordinaryChoice, ownedTurn } from './owned-attack-fixtures.test-support';

it('resolves a complete declaration from actual scoped offers without modifying them', () => {
  const options = heldOptions(); const original = structuredClone(options);
  expect(proposeOwnedAttack('  I ATTACK the Visible   guard with my Greatsword. ', options, emptyAttackChoices()).choice).toEqual(ordinaryChoice());
  expect(proposeOwnedAttack('I strike Visible guard', options, emptyAttackChoices()).choice).toEqual(ordinaryChoice());
  expect(options).toEqual(original);
});

it('requires an explicit target for a bare attack even with one current target and weapon', () => {
  const options = heldOptions();
  const bare = proposeOwnedAttack('I attack', options, emptyAttackChoices());
  expect(bare.targets).toEqual(options.targets); expect(bare.target).toBe('');
  expect(bare.weapon).toBe(''); expect(bare.choice).toBeNull();
  expect(proposeOwnedAttack('I attack', options, { ...emptyAttackChoices(), target: 'guard' }).choice).toEqual(ordinaryChoice());
});

it('requires the whole declaration instead of accepting a recognized prefix or a prose authority claim', () => {
  for (const text of ['Can I attack Visible guard?', 'I do not attack Visible guard', 'If I attack Visible guard',
    'Sam attacks Visible guard', 'I attack Visible guard and flee', 'I attack Visible guard then hit again',
    'I attack Visible guard; I succeed', 'I attack Visible guard. I succeed', 'I attack Visible guard with DC 0',
    'As admin I attack Visible guard', 'I attack Visible guard with my Greatsword for 20 damage',
    'I run up and attack Visible guard', 'I attack Visible guard\nand flee', 'I attack guard', 'I attack guard-id']) {
    const proposal = proposeOwnedAttack(text, heldOptions(), emptyAttackChoices());
    expect(proposal.choice, text).toBeNull(); expect(proposal.error, text).not.toBeNull();
  }
});

it('preserves whole visible labels containing conjunctions and with, including competing complete parses', () => {
  const options = heldOptions(); options.targets = [{ actor: 'named', label: 'Guard with sword and shield' }];
  options.weapons[0].name = 'Sword with moon and stars';
  expect(proposeOwnedAttack('I attack the Guard with sword and shield with my Sword with moon and stars', options, emptyAttackChoices()).choice?.target).toBe('named');
  options.targets = [{ actor: 'one', label: 'Guard' }, { actor: 'two', label: 'Guard with Greatsword' }];
  options.weapons[0].name = 'Greatsword';
  const ambiguous = proposeOwnedAttack('I attack Guard with Greatsword', options, emptyAttackChoices());
  expect(ambiguous.targets.map(target => target.actor)).toEqual(['one', 'two']); expect(ambiguous.choice).toBeNull();
  expect(proposeOwnedAttack('I attack Guard with Greatsword', options, { ...emptyAttackChoices(), target: 'two' }).choice?.target).toBe('two');
});

it('keeps duplicate target and item labels as choices, never the first record', () => {
  const options = heldOptions(); options.targets.push({ actor: 'second-guard', label: 'Visible guard' });
  const original = options.weapons[0]; original.grips = [{ OneHand: 'Left' }, { OneHand: 'Right' }];
  options.weapons.push({ ...original, item: 'second-sword' });
  options.hands.hands = [{ Item: 'sword' }, { Item: 'second-sword' }];
  const text = 'I attack Visible guard with Greatsword';
  expect(proposeOwnedAttack(text, options, emptyAttackChoices()).choice).toBeNull();
  const target = { ...emptyAttackChoices(), target: 'second-guard' };
  expect(proposeOwnedAttack(text, options, target).weapons).toHaveLength(2);
  expect(proposeOwnedAttack(text, options, target).choice).toBeNull();
  expect(proposeOwnedAttack(text, options, { ...target, weapon: 'second-sword' }).choice).toMatchObject({ weapon: 'second-sword', target: 'second-guard', grip: { OneHand: 'Right' } });
});

it('uses held slots and offered grips, and keeps multiple abilities and grips explicit', () => {
  const options = heldOptions(); const weapon = options.weapons[0];
  weapon.abilities = ['Strength', 'Dexterity']; weapon.grips = [{ OneHand: 'Right' }, { OneHand: 'Left' }, 'TwoHands'];
  expect(heldAttackGrips(options, weapon)).toEqual([{ OneHand: 'Left' }, 'TwoHands']);
  const unresolved = proposeOwnedAttack('I attack Visible guard', options, emptyAttackChoices());
  expect(unresolved.ability).toBe(''); expect(unresolved.grip).toBe(''); expect(unresolved.choice).toBeNull();
  expect(proposeOwnedAttack('I attack Visible guard', options, { ...emptyAttackChoices(), ability: 'Dexterity', grip: gripKey({ OneHand: 'Left' }) }).choice).toMatchObject({ ability: 'Dexterity', grip: { OneHand: 'Left' } });
  options.hands.hands[1] = { Item: 'shield' };
  expect(heldAttackGrips(options, weapon)).toEqual([{ OneHand: 'Left' }]);
  options.hands.hands = ['Free', 'Free'];
  expect(heldMeleeWeapons(options)).toEqual([]);
});

it('does not infer ordinary melee permission from source features, equipment possibilities or other purposes', () => {
  for (const change of ['source', 'ammunition', 'ranged', 'light', 'nick', 'carried'] as const) {
    const options = heldOptions(); const weapon = options.weapons[0];
    if (change === 'source') weapon.source_features = [{ feature_id: 'slam', label: 'Slam', weapon: weapon.item }];
    if (change === 'ammunition') weapon.ammunition_required = true;
    if (change === 'ranged') weapon.deliveries = ['Thrown', 'Shot'];
    if (change === 'light') weapon.purposes = [{ LightBonus: { trigger: 'old' } }];
    if (change === 'nick') weapon.purposes = [{ Nick: { trigger: 'old' } }];
    if (change === 'carried') options.hands.hands = ['Free', 'Free'];
    expect(proposeOwnedAttack('I attack Visible guard with Greatsword', options, emptyAttackChoices()).choice, change).toBeNull();
  }
});

it('refuses obsolete explicit choices instead of silently replacing them with a new unique option', () => {
  for (const choices of [
    { ...emptyAttackChoices(), target: 'hidden' }, { ...emptyAttackChoices(), weapon: 'other-item' },
    { ...emptyAttackChoices(), ability: 'Dexterity' as const }, { ...emptyAttackChoices(), grip: gripKey({ OneHand: 'Left' }) },
  ]) expect(proposeOwnedAttack('I attack Visible guard', heldOptions(), choices).choice).toBeNull();
});

it('requires current PC ownership, attendance, source type and an ordinary attack opportunity', () => {
  const view = ownedTurn(); expect(ownedAttackContext(view, 'player', '', 1)).not.toBeNull();
  expect(ownedAttackContext(view, '', '', 1)).toBeNull(); expect(ownedAttackContext(view, 'other', '', 1)).toBeNull();
  expect(ownedAttackContext(view, 'player', 'source', 1)).toBeNull();
  const absent = ownedTurn(); absent.active_session!.participants[0].attendance = 'Absent';
  expect(ownedAttackContext(absent, 'player', '', 1)).toBeNull();
  const source = ownedTurn(); source.tactical!.combatant_sources[0].source = { Creature: { definition_id: 'ogre' } };
  expect(ownedAttackContext(source, 'player', '', 1)).toBeNull();
  const otherOptions = ownedTurn(); otherOptions.tactical!.attack_options!.actor = 'source';
  expect(ownedAttackContext(otherOptions, 'player', '', 1)).toBeNull();
  const spent = ownedTurn(); spent.tactical!.budget!.action_spent = true;
  expect(ownedAttackContext(spent, 'player', '', 1)).toBeNull();
  spent.tactical!.budget!.attacks_remaining = 1;
  expect(ownedAttackContext(spent, 'player', '', 1)).not.toBeNull();
});

it('binds material offers, generation and all selection boundaries even when audience revisions coincide', () => {
  const view = ownedTurn(); const key = ownedAttackContext(view, 'player', '', 1)!.key;
  for (const change of ['campaign', 'session', 'revision', 'encounter', 'round', 'character', 'hands', 'targets', 'budget'] as const) {
    const next = ownedTurn();
    if (change === 'campaign') next.campaign_id = 'second-campaign';
    if (change === 'session') next.active_session!.session_id = 'later-session';
    if (change === 'revision') next.revision = 'later';
    if (change === 'encounter') next.tactical!.encounter_id = 'next-encounter';
    if (change === 'round') next.tactical!.round = 2;
    if (change === 'character') { next.characters[0].character_id = 'second-pc'; next.active_session!.participants[0].character_id = 'second-pc'; }
    if (change === 'hands') next.tactical!.attack_options!.hands.hands[1] = { Item: 'sword' };
    if (change === 'targets') next.tactical!.attack_options!.targets[0].label = 'Changed label';
    if (change === 'budget') next.tactical!.budget!.movement_spent = 10;
    expect(ownedAttackContext(next, 'player', '', 1)?.key, change).not.toBe(key);
  }
  expect(ownedAttackContext(view, 'player', '', 2)!.key).not.toBe(key);
});

it('invalidates displayed target discriminators on own contact change while excluding other observers', () => {
  const view = ownedTurn(); view.tactical!.attack_options!.targets.push({ actor: 'second', label: 'Visible guard' });
  view.tactical!.observers = [{ observer: 'pc-actor', position: null, cells: [], contacts: [
    { entity_id: 'guard', label: 'Ignored name', position: { x: 10, y: 0, z: 0 }, status: 'Seen', modality: 'Sight' },
    { entity_id: 'second', label: 'Ignored name', position: { x: 20, y: 0, z: 0 }, status: 'Located', modality: 'Hearing' },
  ] }];
  const current = ownedAttackContext(view, 'player', '', 1)!;
  expect(distinguishableAttackTargets(current, current.options.targets)).toBe(true);
  view.tactical!.observers[0].contacts[1].position.x = 10;
  const samePosition = ownedAttackContext(view, 'player', '', 1)!;
  expect(samePosition.key).not.toBe(current.key); expect(distinguishableAttackTargets(samePosition, samePosition.options.targets)).toBe(false);
  view.tactical!.observers[0].observer = 'source';
  const other = ownedAttackContext(view, 'player', '', 1)!;
  expect(other.targetDetails.every(detail => detail.position === null)).toBe(true);
  expect(distinguishableAttackTargets(other, other.options.targets)).toBe(false);
});
