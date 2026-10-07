import { render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import TableApp from './TableApp.svelte';
import { contract, emptyView, options } from './components/table-fixtures.test-support';
import { REQUEST_KEY, SELECTION_KEY, tableApi, type TableView } from './table-api';

vi.mock('./table-api', async original => ({
  ...await original<typeof import('./table-api')>(),
  tableApi: {
    defaults: vi.fn(), list: vi.fn(), create: vi.fn(), view: vi.fn(), options: vi.fn(),
    situation: vi.fn(), action: vi.fn(), text: vi.fn(), rollOptions: vi.fn(),
    creatureOptions: vi.fn(), sourceControlOptions: vi.fn(),
  },
}));

// These are DTO fixtures for the rendered desktop; the application case produces
// and checks these independent capability versions through genuine activations.
function table(grappleVersion?: 3 | 4, inspired = false): TableView {
  return {
    ...emptyView(),
    physical: { version: 5, actors: [], controls: [] },
    ...(grappleVersion ? { grapple: { version: grappleVersion, choices: [], ground_drag: [] } } : {}),
    players: [
      { id: 'owner', campaign_id: 'campaign', display_name: 'Sam' },
      { id: 'other', campaign_id: 'campaign', display_name: 'Alex' },
    ],
    characters: [
      { character_id: 'pc', entity_id: 'actor', player_id: 'owner', name: 'River', profile: {}, sheet: null, details: { heroic_inspiration: inspired }, second_wind_remaining: null },
      { character_id: 'ally', entity_id: 'ally-actor', player_id: 'other', name: 'Ash', profile: null, sheet: null, details: null, second_wind_remaining: null },
    ] as unknown as TableView['characters'],
    active_session: {
      session_id: 'session', display_name: 'Evening', started_at_world: 0,
      participants: [
        { player_id: 'owner', character_id: 'pc', attendance: 'Present' },
        { player_id: 'other', character_id: 'ally', attendance: 'Present' },
      ],
    },
  };
}
function pending(): TableView {
  return {
    ...table(4, true),
    inspiration_transfer: {
      character_id: 'pc',
      choices: [
        { key: 'owned-gift', label: 'Give to Ash' },
        { key: 'owned-decline', label: 'Decline the extra Inspiration' },
      ],
    },
  };
}
function dice(): TableView {
  return {
    ...table(4, true), roll_channel: 'Tactical',
    roll: { id: 'owned-roll', roller: 'actor', dice: [{ count: 1, sides: 20 }], modifier: 5, mode: 'Normal', visibility: 'Public', reason: 'Resist the grip' },
  };
}
function select(playerId: string | null) {
  localStorage.setItem(SELECTION_KEY, JSON.stringify({ campaignId: 'campaign', playerId }));
}
async function enterInspiredFaces(user: ReturnType<typeof userEvent.setup>) {
  await waitFor(() => expect(screen.getByLabelText('Spend Heroic Inspiration to reroll one die').matches(':disabled')).toBe(false));
  await user.type(screen.getByLabelText('Die 1 · d20'), '20');
  await user.click(screen.getByLabelText('Spend Heroic Inspiration to reroll one die'));
  await user.selectOptions(screen.getByLabelText('Die to reroll'), '0');
  await user.type(screen.getByLabelText('Inspiration replacement'), '1');
  await user.click(screen.getByRole('button', { name: 'Report these faces' }));
}
const physicalRoll = {
  version: 5, channel: { Player: { player_id: 'owner', character_id: 'pc' } },
  session_id: 'session', revision: 'visible-revision',
  action: { Tactical: { action: { SubmitRollWithInspiration: {
    result: { request_id: 'owned-roll', source: 'Physical', dice: [{ sides: 20, value: 20 }] },
    die_index: 0, replacement: { sides: 20, value: 1 },
  } } } },
};

beforeEach(() => {
  localStorage.clear(); vi.resetAllMocks();
  vi.mocked(tableApi.defaults).mockResolvedValue(structuredClone(contract));
  vi.mocked(tableApi.list).mockResolvedValue([{ id: 'campaign', name: 'Saved campaign' }]);
  vi.mocked(tableApi.options).mockResolvedValue(options);
  vi.mocked(tableApi.situation).mockResolvedValue({ title: '', description: '', challenges: [] });
  vi.mocked(tableApi.rollOptions).mockResolvedValue({ savage_attacker: null, heroic_inspiration: true });
  vi.mocked(tableApi.creatureOptions).mockResolvedValue([]);
});

describe('independent Grapple and Ground capabilities with physical presentation v5', () => {
  it('offers Grapple and then Ground separately, submitting each activation as version 5', async () => {
    const user = userEvent.setup();
    let view: TableView = {
      ...table(),
      tactical: {
        execution: 'EncounterReleaseV1', encounter_id: 'encounter', round: 1,
        active_actor: 'actor', phase: 'active', battlefield: null, participants: [],
        combatant_sources: [], observers: [], initiative: [], ties: [], continuation: null,
        may_fail_save: null, legendary_resistance: null, legendary_action: null, budget: null,
      },
    };
    vi.mocked(tableApi.view).mockImplementation(async () => structuredClone(view));
    vi.mocked(tableApi.action).mockImplementation(async request => {
      expect(request).toMatchObject({ version: 5 });
      if (request.action === 'EnableGrappleAccess') {
        expect(view.grapple).toBeUndefined();
        view = { ...view, revision: 'grapple-enabled', grapple: { version: 3, choices: [], ground_drag: [] } };
      } else {
        expect(request.action).toBe('EnableGrappleTransport');
        expect(view.grapple?.version).toBe(3);
        view = { ...view, revision: 'ground-enabled', grapple: { version: 4, choices: [], ground_drag: [] } };
      }
      return { command_id: request.command_id, revision: view.revision, outcome: { message: 'Capability enabled.' } };
    });
    select(null); render(TableApp);
    await waitFor(() => expect(screen.getByRole('button', { name: 'Enable Grapple' }).matches(':disabled')).toBe(false));
    expect(screen.queryByRole('button', { name: 'Enable dragging and Inspiration' })).toBeNull();
    expect(screen.queryByLabelText('Recipient')).toBeNull();
    expect(screen.queryByText('Extra Heroic Inspiration')).toBeNull();
    await user.click(screen.getByRole('button', { name: 'Enable Grapple' }));
    await waitFor(() => expect(screen.getByRole('button', { name: 'Enable dragging and Inspiration' }).matches(':disabled')).toBe(false));
    expect(screen.queryByLabelText('Recipient')).toBeNull();
    expect(screen.queryByText('Extra Heroic Inspiration')).toBeNull();
    await user.click(screen.getByRole('button', { name: 'Enable dragging and Inspiration' }));
    await waitFor(() => expect(screen.getByLabelText('Recipient').matches(':disabled')).toBe(false));
    expect(screen.queryByRole('button', { name: 'Enable dragging and Inspiration' })).toBeNull();
    expect(tableApi.action).toHaveBeenCalledTimes(2);
    expect(vi.mocked(tableApi.action).mock.calls[0][0]).toMatchObject({ version: 5, channel: 'Host', revision: 'visible-revision', action: 'EnableGrappleAccess' });
    expect(vi.mocked(tableApi.action).mock.calls[1][0]).toMatchObject({ version: 5, channel: 'Host', revision: 'grapple-enabled', action: 'EnableGrappleTransport' });
    expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
  });

  it.each([false, true])('submits the visible Host award once through version 5, already inspired: %s', async inspired => {
    const user = userEvent.setup();
    vi.mocked(tableApi.view).mockResolvedValue(table(4, inspired));
    vi.mocked(tableApi.action).mockImplementation(async request => ({ command_id: request.command_id, revision: 'awarded', outcome: { message: 'Award accepted.' } }));
    select(null); render(TableApp);
    await waitFor(() => expect(screen.getByLabelText('Recipient').matches(':disabled')).toBe(false));
    await user.selectOptions(screen.getByLabelText('Recipient'), 'pc');
    await user.type(screen.getByLabelText('Reason for the award'), 'For helping a companion.');
    await user.click(screen.getByRole('button', { name: 'Award Heroic Inspiration' }));
    await screen.findByText('Award accepted.');
    expect(tableApi.action).toHaveBeenCalledTimes(1);
    expect(vi.mocked(tableApi.action).mock.calls[0][0]).toMatchObject({
      version: 5, channel: 'Host', revision: 'visible-revision', session_id: 'session',
      action: { [inspired ? 'AwardExcessInspiration' : 'AwardHeroicInspiration']: { character_id: 'pc', reason: 'For helping a companion.' } },
    });
    expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
  });

  it.each([undefined, 3] as const)('does not expose owner Inspiration choices without Ground, Grapple version: %s', async grappleVersion => {
    vi.mocked(tableApi.view).mockResolvedValue(table(grappleVersion, true));
    select('owner'); render(TableApp);
    await waitFor(() => expect(screen.getByLabelText('Local viewing and input channel').matches(':disabled')).toBe(false));
    expect(screen.queryByLabelText('Recipient')).toBeNull();
    expect(screen.queryByText('Extra Heroic Inspiration')).toBeNull();
    expect(screen.queryByRole('button', { name: 'Give to Ash' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Decline the extra Inspiration' })).toBeNull();
    expect(tableApi.action).not.toHaveBeenCalled();
    expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
  });

  it.each(['gift', 'decline'] as const)('submits the actual owner %s choice through version 5', async choice => {
    const user = userEvent.setup();
    vi.mocked(tableApi.view).mockResolvedValue(pending());
    vi.mocked(tableApi.action).mockImplementation(async request => ({ command_id: request.command_id, revision: 'chosen', outcome: { message: 'Choice accepted.' } }));
    select('owner'); render(TableApp);
    const label = choice === 'gift' ? 'Give to Ash' : 'Decline the extra Inspiration';
    await waitFor(() => expect(screen.getByRole('button', { name: label }).matches(':disabled')).toBe(false));
    expect(screen.queryByLabelText('Recipient')).toBeNull();
    await user.click(screen.getByRole('button', { name: label }));
    await screen.findByText('Choice accepted.');
    expect(tableApi.action).toHaveBeenCalledTimes(1);
    expect(vi.mocked(tableApi.action).mock.calls[0][0]).toMatchObject({
      version: 5, channel: { Player: { player_id: 'owner', character_id: 'pc' } },
      revision: 'visible-revision', session_id: 'session', action: { InspirationTransfer: { handle: `owned-${choice}` } },
    });
    expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
  });

  it('submits physical Inspiration exactly once with the original and replacement faces through version 5', async () => {
    const user = userEvent.setup(); let view = dice();
    vi.mocked(tableApi.view).mockImplementation(async () => structuredClone(view));
    vi.mocked(tableApi.action).mockImplementation(async request => {
      view = { ...table(4), revision: 'roll-accepted' };
      return { command_id: request.command_id, revision: view.revision, outcome: { message: 'Physical Inspiration accepted.' } };
    });
    select('owner'); render(TableApp);
    await enterInspiredFaces(user);
    await screen.findByText('Physical Inspiration accepted.');
    await waitFor(() => expect(screen.queryByRole('button', { name: 'Report these faces' })).toBeNull());
    expect(tableApi.action).toHaveBeenCalledTimes(1);
    expect(vi.mocked(tableApi.action).mock.calls[0][0]).toMatchObject(physicalRoll);
    expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
  });

  it('retries the exact version 5 physical Inspiration request after restart and a later audience/session', async () => {
    const user = userEvent.setup();
    vi.mocked(tableApi.view).mockResolvedValue(dice());
    vi.mocked(tableApi.action)
      .mockRejectedValueOnce({ message: 'Physical dice delivery uncertain.', retryable: true })
      .mockImplementationOnce(async request => ({ command_id: request.command_id, revision: 'accepted-original', outcome: { message: 'Original dice accepted.' } }));
    select('owner'); const mounted = render(TableApp);
    await enterInspiredFaces(user);
    await screen.findByText('Physical dice delivery uncertain.');
    expect(tableApi.action).toHaveBeenCalledTimes(1);
    const saved = JSON.parse(localStorage.getItem(REQUEST_KEY)!);
    expect(saved.request).toMatchObject(physicalRoll);
    expect(screen.getByRole('button', { name: 'Report these faces' }).matches(':disabled')).toBe(true);
    mounted.unmount();
    const later = table(4);
    vi.mocked(tableApi.view).mockResolvedValue({ ...later, revision: 'later-view', active_session: { ...later.active_session!, session_id: 'later-session' } });
    select(null); render(TableApp);
    await waitFor(() => expect(screen.getByRole('button', { name: 'Retry original request' }).matches(':disabled')).toBe(false));
    await user.click(screen.getByRole('button', { name: 'Retry original request' }));
    await screen.findByText('Original dice accepted.');
    await waitFor(() => expect(localStorage.getItem(REQUEST_KEY)).toBeNull());
    expect(tableApi.action).toHaveBeenCalledTimes(2);
    expect(tableApi.action).toHaveBeenNthCalledWith(1, saved.request);
    expect(tableApi.action).toHaveBeenNthCalledWith(2, saved.request);
  });
});
