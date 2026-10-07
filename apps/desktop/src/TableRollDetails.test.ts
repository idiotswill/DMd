import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { tick } from 'svelte';
import { beforeEach, expect, it, vi } from 'vitest';
import TableApp from './TableApp.svelte';
import RollForm from './components/RollForm.svelte';
import { contract, emptyView, options } from './components/table-fixtures.test-support';
import { REQUEST_KEY, SELECTION_KEY, tableApi, type TableView } from './table-api';

vi.mock('./table-api', async original => ({ ...await original<typeof import('./table-api')>(), tableApi: {
  defaults: vi.fn(), list: vi.fn(), create: vi.fn(), view: vi.fn(), options: vi.fn(), situation: vi.fn(),
  action: vi.fn(), text: vi.fn(), rollDetails: vi.fn(), rollOptions: vi.fn(), creatureOptions: vi.fn(), sourceControlOptions: vi.fn(),
} }));

type Details = Awaited<ReturnType<typeof tableApi.rollDetails>>;
const playerChannel = { Player: { player_id: 'player', character_id: 'pc' } } as const;
const sourceChannel = { SourceCreature: { player_id: 'player', actor: 'goblin' } } as const;
function pending(version: 3 | 4 = 3): TableView {
  return { ...emptyView(), grapple: { version, choices: [], ground_drag: [] },
    players: [{ id: 'player', campaign_id: 'campaign', display_name: 'Sam' }],
    characters: [{ character_id: 'pc', entity_id: 'actor', player_id: 'player', name: 'River', profile: null, sheet: null, details: null, second_wind_remaining: null }],
    source_control: { version: 2, actors: [{ actor: 'goblin', name: 'Small armored figure', definition_id: 'goblin-warrior', controller: { Player: 'player' }, hp: 10, max_hp: 10 }] },
    active_session: { session_id: 'session', display_name: 'Evening', started_at_world: 0, participants: [{ player_id: 'player', character_id: 'pc', attendance: 'Present' }] },
    roll_channel: 'Tactical', roll: { id: 'opaque-save', roller: 'actor', dice: [{ count: 1, sides: 20 }], modifier: 5, mode: 'Normal', visibility: 'Public', reason: 'Unsupported roll' },
  };
}
function details(display_reason: string, inspired = false): Details {
  return { version: 1, options: { savage_attacker: null, heroic_inspiration: inspired }, display_reason };
}
function select(playerId: string | null = 'player', sourceActorId?: string) {
  localStorage.setItem(SELECTION_KEY, JSON.stringify({ campaignId: 'campaign', playerId, ...(sourceActorId ? { sourceActorId } : {}) }));
}
async function ready() {
  await waitFor(() => expect(screen.getByRole('button', { name: 'Refresh saved table' }).matches(':disabled')).toBe(false));
}
beforeEach(() => {
  localStorage.clear(); vi.resetAllMocks();
  vi.mocked(tableApi.defaults).mockResolvedValue(structuredClone(contract));
  vi.mocked(tableApi.list).mockResolvedValue([{ id: 'campaign', name: 'First' }, { id: 'other', name: 'Second' }]);
  vi.mocked(tableApi.options).mockResolvedValue(options);
  vi.mocked(tableApi.situation).mockResolvedValue({ title: '', description: '', challenges: [] });
  vi.mocked(tableApi.creatureOptions).mockResolvedValue([]);
  vi.mocked(tableApi.rollDetails).mockResolvedValue(details('Grapple saving throw'));
  vi.mocked(tableApi.action).mockImplementation(async request => ({ command_id: request.command_id, revision: 'accepted', outcome: { message: 'Encounter action recorded.' } }));
});

it.each(['Grapple saving throw', 'Strength (Athletics) Escape', 'Dexterity (Acrobatics) Escape'])(
  'renders owned %s with its original modifier, raw die and single unchanged submission', async label => {
    const user = userEvent.setup(); const view = pending(); const original = structuredClone(view);
    vi.mocked(tableApi.view).mockResolvedValue(view);
    vi.mocked(tableApi.rollDetails).mockResolvedValue(details(label));
    select(); render(TableApp); await ready();
    expect(screen.getByText(label)).toBeTruthy();
    expect(screen.queryByText('Unsupported roll')).toBeNull();
    expect(screen.getByText(/Normal roll.*rules modifier \+5/)).toBeTruthy();
    expect(screen.queryByLabelText('Die 2 · d20')).toBeNull();
    expect(screen.queryByLabelText('Spend Heroic Inspiration to reroll one die')).toBeNull();
    expect(tableApi.rollDetails).toHaveBeenCalledExactlyOnceWith({ version: 1, campaign_id: 'campaign', revision: 'visible-revision', roll_id: 'opaque-save', channel: playerChannel });
    expect(tableApi.rollOptions).not.toHaveBeenCalled();
    await user.type(screen.getByLabelText('Die 1 · d20'), '12');
    await user.click(screen.getByRole('button', { name: 'Report these faces' }));
    await ready();
    expect(tableApi.action).toHaveBeenCalledExactlyOnceWith({ version: 3, command_id: expect.any(String), campaign_id: 'campaign', session_id: 'session', channel: playerChannel, revision: 'visible-revision', action: { Tactical: { action: { SubmitRoll: { result: { request_id: 'opaque-save', source: 'Physical', dice: [{ sides: 20, value: 12 }] } } } } } });
    expect(view).toEqual(original); expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
  },
);

it('keeps the G4 Inspiration options, original faces and exact retry independent of its live label', async () => {
  const user = userEvent.setup(); const view = pending(4); const original = structuredClone(view);
  vi.mocked(tableApi.view).mockResolvedValue(view);
  vi.mocked(tableApi.rollDetails).mockResolvedValue(details('Grapple saving throw', true));
  vi.mocked(tableApi.action).mockRejectedValueOnce({ message: 'Dice delivery uncertain.', retryable: true })
    .mockImplementationOnce(async request => ({ command_id: request.command_id, revision: 'accepted', outcome: { message: 'Encounter action recorded.' } }));
  select(); const mounted = render(TableApp); await ready();
  expect(screen.getByText('Grapple saving throw')).toBeTruthy();
  await user.type(screen.getByLabelText('Die 1 · d20'), '1');
  await user.click(screen.getByLabelText('Spend Heroic Inspiration to reroll one die'));
  await user.selectOptions(screen.getByLabelText('Die to reroll'), '0');
  await user.type(screen.getByLabelText('Inspiration replacement'), '20');
  await user.click(screen.getByRole('button', { name: 'Report these faces' }));
  await screen.findByText('Dice delivery uncertain.');
  const saved = JSON.parse(localStorage.getItem(REQUEST_KEY)!);
  expect(tableApi.action).toHaveBeenCalledExactlyOnceWith(saved.request);
  expect(saved.request).toEqual({ version: 4, command_id: expect.any(String), campaign_id: 'campaign', session_id: 'session', channel: playerChannel, revision: 'visible-revision', action: { Tactical: { action: { SubmitRollWithInspiration: { result: { request_id: 'opaque-save', source: 'Physical', dice: [{ sides: 20, value: 1 }] }, die_index: 0, replacement: { sides: 20, value: 20 } } } } } });
  expect(view).toEqual(original); mounted.unmount();
  vi.mocked(tableApi.view).mockResolvedValue({ ...view, revision: 'later', roll: null, roll_channel: null });
  select(null); render(TableApp); await ready();
  await user.click(screen.getByRole('button', { name: 'Retry original request' }));
  await ready();
  expect(tableApi.action).toHaveBeenCalledTimes(2);
  expect(tableApi.action).toHaveBeenNthCalledWith(2, saved.request);
  expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
});

it.each(['Grapple saving throw', 'Strength (Athletics) Escape', 'Dexterity (Acrobatics) Escape'])(
  'loads %s only after selecting the actual source owner and retains that submission channel', async label => {
    const user = userEvent.setup(); const view = pending(); view.roll = { ...view.roll!, roller: 'goblin', modifier: 2 };
    vi.mocked(tableApi.view).mockResolvedValue(view);
    vi.mocked(tableApi.rollDetails).mockResolvedValue(details(label));
    select(); render(TableApp); await ready();
    expect(tableApi.rollDetails).not.toHaveBeenCalled(); expect(screen.queryByLabelText('Die 1 · d20')).toBeNull();
    await user.selectOptions(screen.getByLabelText('Controlled actor'), 'goblin'); await ready();
    expect(screen.getByText(label)).toBeTruthy(); expect(screen.getByText(/rules modifier \+2/)).toBeTruthy();
    expect(tableApi.rollDetails).toHaveBeenCalledExactlyOnceWith({ version: 1, campaign_id: 'campaign', revision: 'visible-revision', roll_id: 'opaque-save', channel: sourceChannel });
    await user.type(screen.getByLabelText('Die 1 · d20'), '14');
    await user.click(screen.getByRole('button', { name: 'Report these faces' })); await ready();
    expect(tableApi.action).toHaveBeenCalledExactlyOnceWith({ version: 3, command_id: expect.any(String), campaign_id: 'campaign', session_id: 'session', channel: sourceChannel, revision: 'visible-revision', action: { Tactical: { action: { SubmitRoll: { result: { request_id: 'opaque-save', source: 'Physical', dice: [{ sides: 20, value: 14 }] } } } } } });
    expect(view.roll.reason).toBe('Unsupported roll');
  },
);

it('recovers the pending source selection and live label after closing with unsubmitted faces', async () => {
  const user = userEvent.setup(); const view = pending(); view.roll = { ...view.roll!, roller: 'goblin' };
  vi.mocked(tableApi.view).mockResolvedValue(view);
  vi.mocked(tableApi.rollDetails).mockResolvedValue(details('Dexterity (Acrobatics) Escape'));
  select('player', 'goblin'); const mounted = render(TableApp); await ready();
  await user.type(screen.getByLabelText('Die 1 · d20'), '9'); mounted.unmount();
  render(TableApp); await ready();
  expect(screen.getByText('Dexterity (Acrobatics) Escape')).toBeTruthy();
  expect((screen.getByLabelText('Controlled actor') as HTMLSelectElement).value).toBe('goblin');
  expect((screen.getByLabelText('Die 1 · d20') as HTMLInputElement).value).toBe('');
  expect(tableApi.rollDetails).toHaveBeenCalledTimes(2);
  expect(tableApi.rollDetails).toHaveBeenLastCalledWith({ version: 1, campaign_id: 'campaign', revision: 'visible-revision', roll_id: 'opaque-save', channel: sourceChannel });
  expect(tableApi.action).not.toHaveBeenCalled(); expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
  expect(view.roll.reason).toBe('Unsupported roll');
});

it('keeps Host viewing a PC save on the waiting message without an owned details request or dice form', async () => {
  vi.mocked(tableApi.view).mockResolvedValue(pending()); select(null); render(TableApp); await ready();
  expect(screen.getByText("A physical roll is pending. Select the attending player's channel to report their dice.")).toBeTruthy();
  expect(tableApi.rollDetails).not.toHaveBeenCalled(); expect(tableApi.action).not.toHaveBeenCalled();
  expect(screen.queryByLabelText('Die 1 · d20')).toBeNull();
  expect(screen.queryByText('Grapple saving throw')).toBeNull();
});

it.each(['campaign', 'channel', 'revision', 'roll-id', 'source'] as const)(
  'discards both late details and late errors after a %s change without leaking options or labels', async change => {
    const user = userEvent.setup();
    for (const fail of [false, true]) {
      const first = pending(4);
      vi.mocked(tableApi.view).mockResolvedValue(first);
      let resolve!: (answer: Details) => void; let reject!: (reason: unknown) => void;
      const late = new Promise<Details>((yes, no) => { resolve = yes; reject = no; });
      vi.mocked(tableApi.rollDetails).mockReturnValueOnce(late);
      vi.mocked(tableApi.rollDetails).mockResolvedValue(details('Dexterity (Acrobatics) Escape'));
      select(); const mounted = render(TableApp);
      await waitFor(() => expect(screen.queryByText('Unsupported roll')).not.toBeNull());
      expect(screen.queryByText('Grapple saving throw')).toBeNull();
      // Dispatch a selection/refresh event already queued before the busy state
      // disabled its control. Ordinary subsequent source selection uses userEvent.
      if (change === 'campaign') {
        vi.mocked(tableApi.view).mockResolvedValue({ ...first, campaign_id: 'other' });
        await fireEvent.change(screen.getByLabelText('Campaign'), { target: { value: 'other' } });
      } else if (change === 'channel') {
        await fireEvent.change(screen.getByLabelText('Local viewing and input channel'), { target: { value: '' } });
      } else {
        const next = { ...first, revision: change === 'roll-id' ? first.revision : 'next-revision', roll: { ...first.roll!, id: 'next-roll', roller: change === 'source' ? 'goblin' : 'actor' } };
        vi.mocked(tableApi.view).mockResolvedValue(next);
        await fireEvent.click(screen.getByRole('button', { name: 'Refresh saved table' }));
        await ready();
        if (change === 'source') { await user.selectOptions(screen.getByLabelText('Controlled actor'), 'goblin'); }
      }
      await ready();
      if (fail) reject('Old private roll details failed.');
      else resolve(details('Old private Grapple choice', true));
      // Await the old read and its catch/finally, rather than an already-true assertion.
      await late.catch(() => undefined); await tick();
      await waitFor(() => {
        expect(screen.queryByText('Old private Grapple choice')).toBeNull();
        expect(screen.queryByText('Old private roll details failed.')).toBeNull();
        expect(screen.queryByLabelText('Spend Heroic Inspiration to reroll one die')).toBeNull();
      });
      if (change === 'campaign' || change === 'channel') expect(screen.queryByLabelText('Die 1 · d20')).toBeNull();
      else {
        expect(screen.getByText('Dexterity (Acrobatics) Escape')).toBeTruthy();
        expect(tableApi.rollDetails).toHaveBeenLastCalledWith({ version: 1, campaign_id: 'campaign', revision: change === 'roll-id' ? first.revision : 'next-revision', roll_id: 'next-roll', channel: change === 'source' ? sourceChannel : playerChannel });
      }
      expect(tableApi.action).not.toHaveBeenCalled(); expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
      mounted.unmount(); vi.mocked(tableApi.rollDetails).mockClear();
    }
  },
);

it.each([undefined, 'Strength (Athletics) Escape'])('keeps the optional RollForm label %s separate from request and raw submission', async displayReason => {
  const user = userEvent.setup(); const request = pending().roll!; const original = structuredClone(request); const submit = vi.fn();
  render(RollForm, { request, displayReason, onSubmit: submit });
  expect(screen.getByText(displayReason ?? 'Unsupported roll')).toBeTruthy();
  await user.type(screen.getByLabelText('Die 1 · d20'), '6');
  await user.click(screen.getByRole('button', { name: 'Report these faces' }));
  expect(submit).toHaveBeenCalledExactlyOnceWith([6]); expect(request).toEqual(original);
});
