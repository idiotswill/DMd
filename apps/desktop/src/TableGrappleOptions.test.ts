import { render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { tick } from 'svelte';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import TableApp from './TableApp.svelte';
import { contract, emptyView, options } from './components/table-fixtures.test-support';
import { REQUEST_KEY, SELECTION_KEY, tableApi, type GrappleOptions, type GrappleOptionsRequest, type TableView } from './table-api';

vi.mock('./table-api', async original => ({ ...await original<typeof import('./table-api')>(), tableApi: {
  defaults: vi.fn(), list: vi.fn(), create: vi.fn(), view: vi.fn(), options: vi.fn(), situation: vi.fn(),
  action: vi.fn(), text: vi.fn(), rollDetails: vi.fn(), creatureOptions: vi.fn(), sourceControlOptions: vi.fn(),
  intrinsicAttackOptions: vi.fn(), grappleOptions: vi.fn(),
} }));

function table(): TableView {
  return { ...emptyView(), players: [{ id: 'player', campaign_id: 'campaign', display_name: 'Sam' }],
    characters: [{ character_id: 'pc', entity_id: 'pc-actor', player_id: 'player', name: 'River', profile: null, sheet: null, details: null, second_wind_remaining: null }],
    active_session: { session_id: 'session', display_name: 'Evening', started_at_world: 0, participants: [{ player_id: 'player', character_id: 'pc', attendance: 'Present' }] },
    source_control: { version: 2, actors: [
      { actor: 'source', name: 'Owned guard', definition_id: 'goblin-warrior', controller: { Player: 'player' }, hp: 7, max_hp: 7 },
      { actor: 'host-source', name: 'Host guard', definition_id: 'goblin-warrior', controller: 'Host', hp: 7, max_hp: 7 },
    ] },
    grapple: { version: 3, ground_drag: [], choices: ['pc-actor', 'source', 'host-source'].flatMap(actor => [
      // Deliberately misleading display text: only the queried original handle decides.
      { actor, key: `${actor}-release`, label: 'Grapple wording for the available grip' },
      { actor, key: `${actor}-attempt`, label: 'Release wording for a paid attack' },
    ]) },
    tactical: { encounter_id: 'encounter', phase: 'active', execution: 'EncounterReleaseV1', round: 1, active_actor: 'pc-actor',
      battlefield: null, participants: [], observers: [], initiative: [], ties: [], continuation: null,
      may_fail_save: null, legendary_resistance: null, legendary_action: null, combatant_sources: [],
      budget: { movement_spent: 0, attacks_remaining: 0, action_spent: true, bonus_action_spent: false, reaction_available: true },
    },
  };
}
function answer(request: GrappleOptionsRequest, paid = false): GrappleOptions {
  const actor = request.channel === 'Host' ? 'host-source' : 'SourceCreature' in request.channel ? request.channel.SourceCreature.actor : 'pc-actor';
  return { ...request, handles: [`${actor}-${paid ? 'attempt' : 'release'}`] };
}
let current: TableView;
function select(mode: 'pc' | 'source' | 'host' = 'pc') {
  localStorage.setItem(SELECTION_KEY, JSON.stringify({ campaignId: 'campaign', playerId: mode === 'host' ? null : 'player', sourceActorId: mode === 'source' ? 'source' : '' }));
}
async function ready() {
  await waitFor(() => expect(screen.getByRole('button', { name: 'Refresh saved table' }).matches(':disabled')).toBe(false));
}
async function flush() { await tick(); await Promise.resolve(); await tick(); }
beforeEach(() => {
  localStorage.clear(); vi.resetAllMocks(); current = table();
  vi.mocked(tableApi.defaults).mockResolvedValue(structuredClone(contract));
  vi.mocked(tableApi.list).mockResolvedValue([{ id: 'campaign', name: 'First' }, { id: 'other-campaign', name: 'Second' }]);
  vi.mocked(tableApi.options).mockResolvedValue(options);
  vi.mocked(tableApi.situation).mockResolvedValue({ title: '', description: '', challenges: [] });
  vi.mocked(tableApi.creatureOptions).mockResolvedValue([]);
  vi.mocked(tableApi.view).mockImplementation(async (campaign, viewer) => {
    const view = structuredClone(current); view.campaign_id = campaign;
    view.grapple!.choices = view.grapple!.choices.filter(choice => viewer === 'Host' ? choice.actor === 'host-source' : choice.actor !== 'host-source');
    return view;
  });
  vi.mocked(tableApi.grappleOptions).mockImplementation(async request => answer(request));
  vi.mocked(tableApi.action).mockResolvedValue({ command_id: 'accepted', revision: 'accepted', outcome: { message: 'Recorded.' } });
});

afterEach(() => { vi.restoreAllMocks(); });

it.each(['pc', 'source', 'host'] as const)('queries the exact %s channel and submits only its original allowed handle', async mode => {
  const user = userEvent.setup(); select(mode); render(TableApp); await ready();
  const control = await screen.findByRole('button', { name: /Grapple wording for the available grip/ });
  expect(screen.queryByRole('button', { name: /Release wording for a paid attack/ })).toBeNull();
  const channel = mode === 'host' ? 'Host' : mode === 'source' ? { SourceCreature: { player_id: 'player', actor: 'source' } } : { Player: { player_id: 'player', character_id: 'pc' } };
  const actor = mode === 'host' ? 'host-source' : mode === 'source' ? 'source' : 'pc-actor';
  expect(tableApi.grappleOptions).toHaveBeenCalledWith({ version: 1, campaign_id: 'campaign', channel, revision: 'visible-revision' });
  expect(tableApi.action).not.toHaveBeenCalled(); expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
  expect(control.textContent).not.toContain(`${actor}-release`);
  await user.click(control); await ready();
  expect(tableApi.action).toHaveBeenCalledExactlyOnceWith({ version: 3, command_id: expect.any(String), campaign_id: 'campaign', session_id: 'session', channel,
    revision: 'visible-revision', action: { Tactical: { action: { GrappleChoice: { handle: `${actor}-release` } } } } });
  expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
});

it('keeps an allowed paid attack clickable with Action spent and a remaining attack slot', async () => {
  const user = userEvent.setup(); current.tactical!.budget!.attacks_remaining = 1;
  vi.mocked(tableApi.grappleOptions).mockImplementation(async request => answer(request, true));
  select(); render(TableApp); await ready();
  await user.click(await screen.findByRole('button', { name: /Release wording for a paid attack/ })); await ready();
  expect(vi.mocked(tableApi.action).mock.calls[0][0].action).toEqual({ Tactical: { action: { GrappleChoice: { handle: 'pc-actor-attempt' } } } });
});

it.each(['version', 'campaign', 'revision', 'channel', 'unknown', 'duplicate'] as const)('fails closed on mismatched %s query data without creating an accepted request', async field => {
  vi.mocked(tableApi.grappleOptions).mockImplementation(async request => {
    const response = answer(request);
    if (field === 'version') Object.assign(response, { version: 2 });
    if (field === 'campaign') response.campaign_id = 'other-campaign';
    if (field === 'revision') response.revision = 'obsolete';
    if (field === 'channel') response.channel = 'Host';
    if (field === 'unknown') response.handles = ['not-an-original-handle'];
    if (field === 'duplicate') response.handles.push(response.handles[0]);
    return response;
  });
  select(); render(TableApp); await ready(); await screen.findByText('Mismatched Grapple options. Refresh the table.');
  expect(screen.queryByRole('button', { name: /Grapple wording|Release wording/ })).toBeNull();
  expect(tableApi.action).not.toHaveBeenCalled(); expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
});

it.each(['source', 'channel', 'campaign', 'revision', 'session', 'encounter', 'round', 'unmount'].flatMap(change => [
  { change, failure: false }, { change, failure: true },
]))('discards obsolete query after $change; late failure=$failure', async ({ change, failure }) => {
  const warnings = vi.spyOn(console, 'warn'); // Keep the original warning output visible.
  const user = userEvent.setup(); let resolve!: (value: GrappleOptions) => void; let reject!: (reason: Error) => void;
  vi.mocked(tableApi.grappleOptions).mockReturnValueOnce(new Promise((yes, no) => { resolve = yes; reject = no; }))
    .mockImplementation(async request => answer(request, true));
  select(); const mounted = render(TableApp); await ready();
  await waitFor(() => expect(tableApi.grappleOptions).toHaveBeenCalledTimes(1));
  const old = vi.mocked(tableApi.grappleOptions).mock.calls[0][0];
  if (change === 'source') await user.selectOptions(screen.getByLabelText('Controlled actor'), 'source');
  else if (change === 'channel') await user.selectOptions(screen.getByLabelText('Local viewing and input channel'), '');
  else if (change === 'unmount') { mounted.unmount(); render(TableApp); }
  else {
    if (change === 'revision') current.revision = 'next-revision';
    if (change === 'session') current.active_session!.session_id = 'next-session';
    if (change === 'encounter') current.tactical!.encounter_id = 'next-encounter';
    if (change === 'round') current.tactical!.round = 2;
    if (change === 'campaign') await user.selectOptions(screen.getByLabelText('Campaign'), 'other-campaign');
    else await user.click(screen.getByRole('button', { name: 'Refresh saved table' }));
  }
  await ready(); await screen.findByRole('button', { name: /Release wording for a paid attack/ });
  if (failure) reject(new Error('Obsolete private query failure')); else resolve(answer(old));
  await flush();
  expect(screen.queryByRole('button', { name: /Grapple wording for the available grip/ })).toBeNull();
  expect(screen.queryByText('Obsolete private query failure')).toBeNull();
  expect(screen.queryByText('Mismatched Grapple options. Refresh the table.')).toBeNull();
  expect(screen.getByRole('button', { name: /Release wording for a paid attack/ })).toBeTruthy();
  expect(tableApi.action).not.toHaveBeenCalled(); expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
  expect(warnings.mock.calls.flat().map(String).join('\n')).not.toContain('derived_inert');
});

it.each([false, true])('a pending accepted-request retry invalidates an in-flight query and unlock reloads; late failure=%s', async failure => {
  const user = userEvent.setup(); current.tactical!.budget!.action_spent = false;
  let resolve!: (value: GrappleOptions) => void; let reject!: (reason: Error) => void;
  vi.mocked(tableApi.grappleOptions).mockReturnValueOnce(new Promise((yes, no) => { resolve = yes; reject = no; }))
    .mockImplementation(async request => answer(request, true));
  vi.mocked(tableApi.action).mockRejectedValueOnce({ message: 'Dodge delivery uncertain.', retryable: true });
  select(); render(TableApp); await ready(); await waitFor(() => expect(tableApi.grappleOptions).toHaveBeenCalledTimes(1));
  const old = vi.mocked(tableApi.grappleOptions).mock.calls[0][0];
  await user.click(screen.getByRole('button', { name: 'Dodge' })); await screen.findByText('Dodge delivery uncertain.');
  const saved = localStorage.getItem(REQUEST_KEY); expect(JSON.parse(saved!).request.action).toEqual({ Tactical: { action: 'Dodge' } });
  if (failure) reject(new Error('Obsolete private query failure')); else resolve(answer(old));
  await flush(); expect(screen.queryByRole('button', { name: /Grapple wording|Release wording/ })).toBeNull();
  expect(screen.queryByText('Obsolete private query failure')).toBeNull(); expect(localStorage.getItem(REQUEST_KEY)).toBe(saved);
  await user.click(screen.getByRole('button', { name: 'Refresh and release local retry' })); await ready();
  await screen.findByRole('button', { name: /Release wording for a paid attack/ });
  expect(tableApi.action).toHaveBeenCalledOnce(); expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
});

it('retains the exact original Grapple request across uncertain delivery and reload', async () => {
  const user = userEvent.setup(); select('source');
  vi.mocked(tableApi.action).mockRejectedValueOnce({ message: 'Grapple delivery uncertain.', retryable: true });
  const mounted = render(TableApp); await ready();
  await user.click(await screen.findByRole('button', { name: /Grapple wording for the available grip/ }));
  await screen.findByText('Grapple delivery uncertain.');
  const saved = localStorage.getItem(REQUEST_KEY); const original = JSON.parse(saved!).request;
  expect(original.channel).toEqual({ SourceCreature: { player_id: 'player', actor: 'source' } });
  expect(original.action).toEqual({ Tactical: { action: { GrappleChoice: { handle: 'source-release' } } } });
  const reads = vi.mocked(tableApi.grappleOptions).mock.calls.length; mounted.unmount();
  current.revision = 'later'; current.active_session!.session_id = 'later-session'; select('host'); render(TableApp); await ready();
  expect(tableApi.grappleOptions).toHaveBeenCalledTimes(reads); expect(localStorage.getItem(REQUEST_KEY)).toBe(saved);
  await user.click(screen.getByRole('button', { name: 'Retry original request' })); await ready();
  expect(tableApi.action).toHaveBeenNthCalledWith(2, original); expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
});

it('retries a current query failure as a read only and never allocates an outbox', async () => {
  const user = userEvent.setup(); vi.mocked(tableApi.grappleOptions).mockRejectedValueOnce(new Error('Current read failed.'));
  select(); render(TableApp); await ready(); await screen.findByText('Current read failed.');
  expect(tableApi.action).not.toHaveBeenCalled(); expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
  await user.click(screen.getByRole('button', { name: 'Retry Grapple choices' }));
  await screen.findByRole('button', { name: /Grapple wording for the available grip/ });
  expect(tableApi.action).not.toHaveBeenCalled(); expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
});
