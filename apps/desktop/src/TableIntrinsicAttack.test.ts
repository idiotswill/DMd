import { render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { tick } from 'svelte';
import { beforeEach, expect, it, vi } from 'vitest';
import TableApp from './TableApp.svelte';
import { intrinsicAttackContext } from './intrinsic-attack-context';
import { contract, emptyView, options } from './components/table-fixtures.test-support';
import { REQUEST_KEY, SELECTION_KEY, tableApi, type IntrinsicAttackOptions, type IntrinsicAttackRequest, type TableView } from './table-api';

vi.mock('./table-api', async original => ({ ...await original<typeof import('./table-api')>(), tableApi: {
  defaults: vi.fn(), list: vi.fn(), create: vi.fn(), view: vi.fn(), options: vi.fn(), situation: vi.fn(),
  action: vi.fn(), text: vi.fn(), rollDetails: vi.fn(), creatureOptions: vi.fn(), sourceControlOptions: vi.fn(), intrinsicAttackOptions: vi.fn(),
} }));

function turn(owned = true): TableView {
  return { ...emptyView(), grapple: { version: 3, choices: [], ground_drag: [] },
    players: [{ id: 'player', campaign_id: 'campaign', display_name: 'Sam' }, { id: 'other', campaign_id: 'campaign', display_name: 'Taylor' }],
    characters: [{ character_id: 'pc', entity_id: 'traveler', player_id: 'player', name: 'River', profile: null, sheet: null, details: null, second_wind_remaining: null }],
    active_session: { session_id: 'session', display_name: 'Evening', started_at_world: 0, participants: [{ player_id: 'player', character_id: 'pc', attendance: 'Present' }] },
    source_control: { version: 2, actors: [
      { actor: 'chimera', name: 'Chimera', definition_id: 'chimera', controller: owned ? { Player: 'player' } : 'Host', hp: 114, max_hp: 114 },
      { actor: 'second-source', name: 'Other creature', definition_id: 'warhorse', controller: { Player: 'player' }, hp: 19, max_hp: 19 },
    ] },
    tactical: { encounter_id: 'encounter', phase: 'active', execution: 'EncounterReleaseV1', round: 1, active_actor: 'chimera',
      battlefield: null, participants: [], observers: [], initiative: [], ties: [], continuation: null,
      may_fail_save: null, legendary_resistance: null, legendary_action: null,
      combatant_sources: [{ actor: 'chimera', source: { Creature: { definition_id: 'chimera' } }, initiative_modifier: 0, normal_mode: 'Normal', surprised_mode: 'Disadvantage' }],
      budget: { movement_spent: 0, attacks_remaining: 0, action_spent: false, bonus_action_spent: false, reaction_available: true },
    },
  };
}
function answer(request: IntrinsicAttackRequest, label = 'Bite'): IntrinsicAttackOptions {
  return { version: 1, revision: request.revision, actor: request.actor,
    features: [{ feature_id: 'bite', label }, { feature_id: 'ram', label: 'Ram' }],
    targets: [{ actor: 'located-target', label: 'Located traveler' }],
  };
}
function select(playerId: string | null = 'player', sourceActorId = 'chimera') {
  localStorage.setItem(SELECTION_KEY, JSON.stringify({ campaignId: 'campaign', playerId, sourceActorId }));
}
async function ready() {
  await waitFor(() => expect(screen.getByRole('button', { name: 'Refresh saved table' }).matches(':disabled')).toBe(false));
}
async function choose(user: ReturnType<typeof userEvent.setup>) {
  await user.click(screen.getByRole('button', { name: 'Choose creature attack' }));
  await screen.findByLabelText('Creature attack');
  await user.selectOptions(screen.getByLabelText('Creature attack'), 'ram');
  await user.selectOptions(screen.getByLabelText('Creature attack target'), 'located-target');
}
beforeEach(() => {
  localStorage.clear(); vi.resetAllMocks();
  vi.mocked(tableApi.defaults).mockResolvedValue(structuredClone(contract));
  vi.mocked(tableApi.list).mockResolvedValue([{ id: 'campaign', name: 'First' }, { id: 'other-campaign', name: 'Second' }]);
  vi.mocked(tableApi.options).mockResolvedValue(options);
  vi.mocked(tableApi.situation).mockResolvedValue({ title: '', description: '', challenges: [] });
  vi.mocked(tableApi.creatureOptions).mockResolvedValue([]);
  vi.mocked(tableApi.rollDetails).mockResolvedValue({ version: 1, options: { savage_attacker: null }, display_reason: 'Creature attack' });
  vi.mocked(tableApi.intrinsicAttackOptions).mockImplementation(async request => answer(request));
  vi.mocked(tableApi.action).mockImplementation(async request => ({ command_id: request.command_id, revision: 'accepted', outcome: { message: 'Encounter action recorded.' } }));
});

it.each([false, true])('uses the separate lazy read and existing persisted CreatureAttack route; source owner=%s', async owned => {
  const user = userEvent.setup(); const view = turn(owned); const original = structuredClone(view);
  vi.mocked(tableApi.view).mockResolvedValue(view); select(owned ? 'player' : null); render(TableApp); await ready();
  expect(tableApi.intrinsicAttackOptions).not.toHaveBeenCalled();
  await choose(user);
  const channel = owned ? { SourceCreature: { player_id: 'player', actor: 'chimera' } } : 'Host';
  expect(tableApi.intrinsicAttackOptions).toHaveBeenCalledExactlyOnceWith({ version: 1, campaign_id: 'campaign', channel, revision: 'visible-revision', actor: 'chimera' });
  expect(tableApi.action).not.toHaveBeenCalled(); expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
  await user.click(screen.getByRole('button', { name: 'Use creature attack' })); await ready();
  expect(tableApi.action).toHaveBeenCalledExactlyOnceWith({ version: 3, command_id: expect.any(String), campaign_id: 'campaign', session_id: 'session', channel,
    revision: 'visible-revision', action: { Tactical: { action: { CreatureAttack: { target: 'located-target', feature_id: 'ram', weapon: null } } } } });
  expect(view).toEqual(original); expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
});

it('retains the entire original source request across uncertain delivery, cleared panel and reload', async () => {
  const user = userEvent.setup(); const view = turn(); vi.mocked(tableApi.view).mockResolvedValue(view);
  vi.mocked(tableApi.action).mockRejectedValueOnce({ message: 'Attack delivery uncertain.', retryable: true })
    .mockImplementationOnce(async request => ({ command_id: request.command_id, revision: 'accepted', outcome: { message: 'Encounter action recorded.' } }));
  select(); const mounted = render(TableApp); await ready(); await choose(user);
  await user.click(screen.getByRole('button', { name: 'Use creature attack' })); await screen.findByText('Attack delivery uncertain.');
  const saved = JSON.parse(localStorage.getItem(REQUEST_KEY)!);
  expect(screen.queryByLabelText('Creature attack')).toBeNull();
  expect(saved.request).toEqual({ version: 3, command_id: expect.any(String), campaign_id: 'campaign', session_id: 'session', channel: { SourceCreature: { player_id: 'player', actor: 'chimera' } }, revision: 'visible-revision', action: { Tactical: { action: { CreatureAttack: { target: 'located-target', feature_id: 'ram', weapon: null } } } } });
  expect(tableApi.action).toHaveBeenCalledExactlyOnceWith(saved.request); mounted.unmount();
  const changed = turn(); changed.revision = 'later'; changed.tactical!.active_actor = 'second-source'; changed.active_session!.session_id = 'later-session';
  vi.mocked(tableApi.view).mockResolvedValue(changed); select(null, 'second-source'); render(TableApp); await ready();
  expect(tableApi.intrinsicAttackOptions).toHaveBeenCalledTimes(1);
  await user.click(screen.getByRole('button', { name: 'Retry original request' })); await ready();
  expect(tableApi.action).toHaveBeenNthCalledWith(2, saved.request);
  expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
});

it.each(['source', 'pc', 'audience', 'campaign', 'revision', 'encounter', 'turn', 'pending'].flatMap(change => [
  { change, lateFailure: false }, { change, lateFailure: true },
]))('ignores obsolete read after $change; late failure=$lateFailure', async ({ change, lateFailure }) => {
  const user = userEvent.setup(); const view = turn(); vi.mocked(tableApi.view).mockResolvedValue(view);
  let resolve!: (value: IntrinsicAttackOptions) => void; let reject!: (error: Error) => void;
  const late = new Promise<IntrinsicAttackOptions>((yes, no) => { resolve = yes; reject = no; });
  vi.mocked(tableApi.intrinsicAttackOptions).mockReturnValueOnce(late).mockImplementation(async request => answer(request, 'Current choice'));
  select(); render(TableApp); await ready(); await user.click(screen.getByRole('button', { name: 'Choose creature attack' }));
  const observed = vi.mocked(tableApi.intrinsicAttackOptions).mock.calls[0][0];
  if (change === 'source' || change === 'pc') await user.selectOptions(screen.getByLabelText('Controlled actor'), change === 'source' ? 'second-source' : '');
  else if (change === 'audience') await user.selectOptions(screen.getByLabelText('Local viewing and input channel'), '');
  else {
    const next = turn();
    if (change === 'campaign') next.campaign_id = 'other-campaign';
    if (change === 'revision') next.revision = 'next-revision';
    if (change === 'encounter') next.tactical!.encounter_id = 'next-encounter';
    if (change === 'turn') next.tactical!.round = 2;
    if (change === 'pending') {
      next.roll_channel = 'Tactical'; next.roll = { id: 'pending-attack', roller: 'chimera', dice: [{ count: 1, sides: 20 }], modifier: 7, mode: 'Normal', visibility: 'Public', reason: 'Attack' };
    }
    vi.mocked(tableApi.view).mockResolvedValue(next);
    if (change === 'campaign') await user.selectOptions(screen.getByLabelText('Campaign'), 'other-campaign');
    else await user.click(screen.getByRole('button', { name: 'Refresh saved table' }));
  }
  await ready(); expect(screen.queryByLabelText('Creature attack')).toBeNull();
  const newControl = screen.queryByRole('button', { name: 'Choose creature attack' });
  if (newControl) { await user.click(newControl); await screen.findByText('Current choice'); }
  if (lateFailure) reject(new Error('Obsolete private read failure'));
  else resolve(answer(observed, 'Obsolete feature'));
  await tick(); await Promise.resolve(); await tick();
  expect(screen.queryByText('Obsolete feature')).toBeNull(); expect(screen.queryByText('Obsolete private read failure')).toBeNull();
  if (newControl) expect(screen.getByText('Current choice')).toBeTruthy();
  expect(tableApi.action).not.toHaveBeenCalled(); expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
});

it.each([false, true])('a new pending retry invalidates an in-flight read without replacing its outbox; late failure=%s', async lateFailure => {
  const user = userEvent.setup(); vi.mocked(tableApi.view).mockResolvedValue(turn());
  let resolve!: (value: IntrinsicAttackOptions) => void; let reject!: (error: Error) => void;
  vi.mocked(tableApi.intrinsicAttackOptions).mockReturnValueOnce(new Promise((yes, no) => { resolve = yes; reject = no; }));
  vi.mocked(tableApi.action).mockRejectedValueOnce({ message: 'Dodge delivery uncertain.', retryable: true });
  select(); render(TableApp); await ready(); await user.click(screen.getByRole('button', { name: 'Choose creature attack' }));
  const observed = vi.mocked(tableApi.intrinsicAttackOptions).mock.calls[0][0];
  await user.click(screen.getByRole('button', { name: 'Dodge' })); await screen.findByText('Dodge delivery uncertain.');
  const saved = localStorage.getItem(REQUEST_KEY); expect(JSON.parse(saved!).request.action).toEqual({ Tactical: { action: 'Dodge' } });
  if (lateFailure) reject(new Error('Obsolete read failure')); else resolve(answer(observed));
  await tick(); await Promise.resolve(); await tick();
  expect(screen.queryByLabelText('Creature attack')).toBeNull(); expect(screen.queryByText('Obsolete read failure')).toBeNull();
  expect(localStorage.getItem(REQUEST_KEY)).toBe(saved); expect(tableApi.action).toHaveBeenCalledOnce();
});

it.each(['version', 'actor', 'revision'])('rejects mismatched %s in the independent response without creating a request', async field => {
  const user = userEvent.setup(); vi.mocked(tableApi.view).mockResolvedValue(turn());
  vi.mocked(tableApi.intrinsicAttackOptions).mockImplementation(async request => ({ ...answer(request), [field]: field === 'version' ? 2 : 'foreign' }) as IntrinsicAttackOptions);
  select(); render(TableApp); await ready(); await user.click(screen.getByRole('button', { name: 'Choose creature attack' }));
  await screen.findByText('The creature attack choices changed. Refresh the table.');
  expect(screen.queryByLabelText('Creature attack')).toBeNull(); expect(tableApi.action).not.toHaveBeenCalled(); expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
});

it('shows empty/read-error states without inventing a creature feature or an outbox', async () => {
  const user = userEvent.setup(); vi.mocked(tableApi.view).mockResolvedValue(turn());
  vi.mocked(tableApi.intrinsicAttackOptions).mockRejectedValueOnce({ message: 'Current choices unavailable.' })
    .mockImplementationOnce(async request => ({ ...answer(request), features: [] }));
  select(); render(TableApp); await ready(); await user.click(screen.getByRole('button', { name: 'Choose creature attack' }));
  await screen.findByText('Current choices unavailable.');
  await user.click(screen.getByRole('button', { name: 'Choose creature attack' }));
  await screen.findByText('No supported creature attack is available against a located target this turn.');
  expect(screen.queryByLabelText('Creature attack')).toBeNull(); expect(tableApi.action).not.toHaveBeenCalled(); expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
});

it('keeps source selection, attendance and pending guards distinct from shared Player visibility', () => {
  const view = turn(); const selected = intrinsicAttackContext(view, 'player', 'chimera');
  expect(selected?.request.channel).toEqual({ SourceCreature: { player_id: 'player', actor: 'chimera' } });
  expect(intrinsicAttackContext(view, 'player', '')).toBeNull();
  expect(intrinsicAttackContext(view, 'player', 'second-source')).toBeNull();
  expect(intrinsicAttackContext(view, 'other', 'chimera')).toBeNull();
  expect(intrinsicAttackContext(view, '', '')).toBeNull();
  const absent = turn(); absent.active_session!.participants[0].attendance = 'Absent';
  expect(intrinsicAttackContext(absent, 'player', 'chimera')).toBeNull();
  const host = turn(false); host.source_control!.actors[0].controller = 'Autonomous';
  expect(intrinsicAttackContext(host, '', '')?.request.channel).toBe('Host');
  const pending = turn(); pending.tactical!.continuation = { actor: 'chimera', host_adjudication: false, choices: [] };
  expect(intrinsicAttackContext(pending, 'player', 'chimera')).toBeNull();
  const spent = turn(); spent.tactical!.budget!.action_spent = true;
  expect(intrinsicAttackContext(spent, 'player', 'chimera')).toBeNull();
  const old = turn(); old.tactical!.execution = 'ShieldMissileV1';
  expect(intrinsicAttackContext(old, 'player', 'chimera')).toBeNull();
  const next = turn(); next.tactical!.round = 2;
  expect(intrinsicAttackContext(next, 'player', 'chimera')?.key).not.toBe(selected?.key);
});
