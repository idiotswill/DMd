import { render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { tick } from 'svelte';
import { beforeEach, expect, it, vi } from 'vitest';
import TableApp from './TableApp.svelte';
import { ownedTurn, ordinaryChoice } from './owned-attack-fixtures.test-support';
import { contract, options } from './components/table-fixtures.test-support';
import { REQUEST_KEY, SELECTION_KEY, tableApi, type TableView } from './table-api';

vi.mock('./table-api', async original => ({ ...await original<typeof import('./table-api')>(), tableApi: {
  defaults: vi.fn(), list: vi.fn(), create: vi.fn(), view: vi.fn(), options: vi.fn(), situation: vi.fn(),
  action: vi.fn(), text: vi.fn(), rollDetails: vi.fn(), creatureOptions: vi.fn(), sourceControlOptions: vi.fn(), intrinsicAttackOptions: vi.fn(),
} }));

function select(playerId: string | null = 'player', sourceActorId = '') {
  localStorage.setItem(SELECTION_KEY, JSON.stringify({ campaignId: 'campaign', playerId, sourceActorId }));
}
async function ready() {
  await waitFor(() => expect(screen.getByRole('button', { name: 'Refresh saved table' }).matches(':disabled')).toBe(false));
}
async function review(user: ReturnType<typeof userEvent.setup>, text = 'I attack the Visible guard with my Greatsword') {
  await user.type(screen.getByLabelText('Attack declaration'), text);
  await user.click(screen.getByRole('button', { name: 'Review attack declaration' }));
}
beforeEach(() => {
  localStorage.clear(); vi.resetAllMocks();
  vi.mocked(tableApi.defaults).mockResolvedValue(structuredClone(contract));
  vi.mocked(tableApi.list).mockResolvedValue([{ id: 'campaign', name: 'First' }, { id: 'other-campaign', name: 'Second' }]);
  vi.mocked(tableApi.options).mockResolvedValue(options);
  vi.mocked(tableApi.situation).mockResolvedValue({ title: '', description: '', challenges: [] });
  vi.mocked(tableApi.creatureOptions).mockResolvedValue([]);
  vi.mocked(tableApi.rollDetails).mockResolvedValue({ version: 1, options: { savage_attacker: null }, display_reason: 'Attack' });
  vi.mocked(tableApi.view).mockResolvedValue(ownedTurn());
  vi.mocked(tableApi.action).mockImplementation(async request => ({ command_id: request.command_id, revision: 'accepted', outcome: { message: 'Encounter action recorded.' } }));
});

it('turns an ordinary declaration into the original PC Attack/outbox only after explicit confirmation', async () => {
  const user = userEvent.setup(); const view = ownedTurn(); const original = structuredClone(view);
  vi.mocked(tableApi.view).mockResolvedValue(view);
  vi.mocked(tableApi.action).mockImplementation(async request => {
    expect(JSON.parse(localStorage.getItem(REQUEST_KEY)!)).toEqual({ kind: 'action', request });
    return { command_id: request.command_id, revision: 'accepted', outcome: { message: 'Encounter action recorded.' } };
  });
  select(); render(TableApp); await ready(); await review(user);
  expect(screen.getByText(/ordinary Attack action/)).toBeTruthy();
  expect(tableApi.action).not.toHaveBeenCalled(); expect(tableApi.text).not.toHaveBeenCalled();
  expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
  await user.click(screen.getByRole('button', { name: 'Attack as described' })); await ready();
  expect(tableApi.action).toHaveBeenCalledExactlyOnceWith({ version: 5, command_id: expect.any(String), campaign_id: 'campaign', session_id: 'session',
    channel: { Player: { player_id: 'player', character_id: 'pc' } }, revision: 'visible-revision', action: { Tactical: { action: { Attack: { choice: ordinaryChoice() } } } } });
  expect(localStorage.getItem(REQUEST_KEY)).toBeNull(); expect(view).toEqual(original);
});

it('asks which target a bare attack means before confirming even a unique offered pair', async () => {
  const user = userEvent.setup(); select(); render(TableApp); await ready(); await review(user, 'I attack');
  expect((screen.getByLabelText('Attack declaration target') as HTMLSelectElement).value).toBe('');
  expect(screen.queryByLabelText('Attack declaration weapon')).toBeNull();
  const confirm = screen.getByRole('button', { name: 'Attack as described' });
  expect((confirm as HTMLButtonElement).disabled).toBe(true); await user.click(confirm);
  expect(tableApi.action).not.toHaveBeenCalled(); expect(tableApi.text).not.toHaveBeenCalled();
  expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
  await user.selectOptions(screen.getByLabelText('Attack declaration target'), 'guard');
  expect((screen.getByLabelText('Attack declaration weapon') as HTMLSelectElement).value).toBe('sword');
  expect(tableApi.action).not.toHaveBeenCalled();
  await user.click(screen.getByRole('button', { name: 'Attack as described' })); await ready();
  expect(tableApi.action).toHaveBeenCalledExactlyOnceWith(expect.objectContaining({ action: { Tactical: { action: { Attack: { choice: ordinaryChoice() } } } } }));
});

it('requires an explicit versatile grip, and editing the sentence withdraws its old preview', async () => {
  const user = userEvent.setup(); const view = ownedTurn(); const weapon = view.tactical!.attack_options!.weapons[0];
  weapon.name = 'Longsword'; weapon.grips = [{ OneHand: 'Left' }, 'TwoHands'];
  vi.mocked(tableApi.view).mockResolvedValue(view); select(); render(TableApp); await ready();
  await review(user, 'I attack Visible guard with Longsword');
  expect((screen.getByRole('button', { name: 'Attack as described' }) as HTMLButtonElement).disabled).toBe(true);
  await user.selectOptions(screen.getByLabelText('Attack declaration grip'), JSON.stringify({ OneHand: 'Left' }));
  expect((screen.getByRole('button', { name: 'Attack as described' }) as HTMLButtonElement).disabled).toBe(false);
  await user.type(screen.getByLabelText('Attack declaration'), ' and flee');
  expect(screen.queryByRole('button', { name: 'Attack as described' })).toBeNull();
  await user.click(screen.getByRole('button', { name: 'Review attack declaration' }));
  expect(screen.queryByRole('button', { name: 'Attack as described' })).toBeNull(); expect(tableApi.action).not.toHaveBeenCalled();
  await user.clear(screen.getByLabelText('Attack declaration')); await review(user, 'I attack Visible guard with Longsword');
  expect((screen.getByLabelText('Attack declaration grip') as HTMLSelectElement).value).toBe('');
});

it('does not choose the first Finesse ability before sending an otherwise complete held attack', async () => {
  const user = userEvent.setup(); const view = ownedTurn(); const weapon = view.tactical!.attack_options!.weapons[0];
  weapon.name = 'Dagger'; weapon.abilities = ['Strength', 'Dexterity']; weapon.grips = [{ OneHand: 'Left' }, { OneHand: 'Right' }];
  vi.mocked(tableApi.view).mockResolvedValue(view); select(); render(TableApp); await ready(); await review(user, 'I attack Visible guard with Dagger');
  expect((screen.getByLabelText('Attack declaration ability') as HTMLSelectElement).value).toBe('');
  expect((screen.getByRole('button', { name: 'Attack as described' }) as HTMLButtonElement).disabled).toBe(true);
  await user.selectOptions(screen.getByLabelText('Attack declaration ability'), 'Dexterity');
  expect(tableApi.action).not.toHaveBeenCalled();
  await user.click(screen.getByRole('button', { name: 'Attack as described' })); await ready();
  const expected = ordinaryChoice(); expected.ability = 'Dexterity'; expected.grip = { OneHand: 'Left' };
  expect(tableApi.action).toHaveBeenCalledWith(expect.objectContaining({ action: { Tactical: { action: { Attack: { choice: expected } } } } }));
});

it('requires explicit duplicate-label target and item choices using only own current positions and held hands', async () => {
  const user = userEvent.setup(); const view = ownedTurn(); const attack = view.tactical!.attack_options!;
  attack.targets.push({ actor: 'second-guard', label: 'Visible guard' });
  attack.weapons[0].name = 'Dagger'; attack.weapons[0].grips = [{ OneHand: 'Left' }, { OneHand: 'Right' }];
  attack.weapons.push({ ...attack.weapons[0], item: 'second-dagger' });
  attack.hands.hands = [{ Item: 'sword' }, { Item: 'second-dagger' }];
  view.tactical!.observers = [{ observer: 'pc-actor', position: null, cells: [], contacts: [
    { entity_id: 'guard', label: 'Visible guard', position: { x: 10, y: 0, z: 0 }, status: 'Seen', modality: 'Sight' },
    { entity_id: 'second-guard', label: 'Visible guard', position: { x: 20, y: 0, z: 0 }, status: 'Located', modality: 'Hearing' },
  ] }];
  vi.mocked(tableApi.view).mockResolvedValue(view); select(); render(TableApp); await ready(); await review(user, 'I attack Visible guard with Dagger');
  expect((screen.getByLabelText('Attack declaration target') as HTMLSelectElement).value).toBe('');
  expect(screen.getByRole('option', { name: 'Visible guard · 10, 0 feet; height 0 feet' })).toBeTruthy();
  await user.selectOptions(screen.getByLabelText('Attack declaration target'), 'second-guard');
  expect((screen.getByLabelText('Attack declaration weapon') as HTMLSelectElement).value).toBe('');
  expect(screen.getByRole('option', { name: 'Dagger · held in right hand' })).toBeTruthy();
  await user.selectOptions(screen.getByLabelText('Attack declaration weapon'), 'second-dagger');
  expect(tableApi.action).not.toHaveBeenCalled();
  await user.click(screen.getByRole('button', { name: 'Attack as described' })); await ready();
  const expected = ordinaryChoice(); expected.target = 'second-guard'; expected.weapon = 'second-dagger'; expected.grip = { OneHand: 'Right' };
  expect(tableApi.action).toHaveBeenCalledWith(expect.objectContaining({ action: { Tactical: { action: { Attack: { choice: expected } } } } }));
});

it('blocks indistinguishable duplicate names even when another observer or old memory has locations', async () => {
  const user = userEvent.setup(); const view = ownedTurn(); view.tactical!.attack_options!.targets.push({ actor: 'second-guard', label: 'Visible guard' });
  const contacts = [
    { entity_id: 'guard', label: 'Visible guard', position: { x: 10, y: 0, z: 0 }, status: 'Seen' as const, modality: 'Sight' },
    { entity_id: 'second-guard', label: 'Visible guard', position: { x: 20, y: 0, z: 0 }, status: 'Seen' as const, modality: 'Sight' },
  ];
  view.tactical!.observers = [
    { observer: 'source', position: null, cells: [], contacts },
    { observer: 'pc-actor', position: null, cells: [], contacts: contacts.map(contact => ({ ...contact, status: 'Remembered' as const })) },
  ];
  vi.mocked(tableApi.view).mockResolvedValue(view); select(); render(TableApp); await ready(); await review(user);
  expect(screen.getByText(/cannot be distinguished from your current view/)).toBeTruthy();
  expect(screen.queryByRole('button', { name: 'Attack as described' })).toBeNull();
  expect(tableApi.action).not.toHaveBeenCalled(); expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
});

it('does not resolve a name from Host participants, another owned observer or remembered contacts', async () => {
  const user = userEvent.setup(); const view = ownedTurn();
  view.tactical!.participants.push({ entity_id: 'hidden', public_label: 'Secret guard', position: { x: 20, y: 0, z: 0 }, size: 'Medium' });
  view.tactical!.observers = [
    { observer: 'source', position: null, cells: [], contacts: [{ entity_id: 'other-seen', label: 'Other observer guard', position: { x: 20, y: 0, z: 0 }, status: 'Seen', modality: 'Sight' }] },
    { observer: 'pc-actor', position: null, cells: [], contacts: [{ entity_id: 'remembered', label: 'Old guard', position: { x: 20, y: 0, z: 0 }, status: 'Remembered', modality: 'Sight' }] },
  ];
  vi.mocked(tableApi.view).mockResolvedValue(view); select(); render(TableApp); await ready();
  for (const label of ['Secret guard', 'Other observer guard', 'Old guard']) {
    await user.clear(screen.getByLabelText('Attack declaration')); await review(user, `I attack ${label} with Greatsword`);
    expect(screen.queryByRole('button', { name: 'Attack as described' })).toBeNull();
  }
  expect(tableApi.action).not.toHaveBeenCalled(); expect(tableApi.text).not.toHaveBeenCalled(); expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
});

it.each(['host', 'source', 'absent', 'foreign-turn', 'pending'] as const)('does not render a PC attack affordance for %s', async kind => {
  const view = ownedTurn();
  if (kind === 'absent') view.active_session!.participants[0].attendance = 'Absent';
  if (kind === 'foreign-turn') view.tactical!.active_actor = 'source';
  if (kind === 'pending') view.tactical!.continuation = { actor: 'pc-actor', host_adjudication: false, choices: [] };
  vi.mocked(tableApi.view).mockResolvedValue(view); select(kind === 'host' ? null : 'player', kind === 'source' ? 'source' : '');
  render(TableApp); await ready(); expect(screen.queryByLabelText('Attack declaration')).toBeNull();
  expect(tableApi.action).not.toHaveBeenCalled(); expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
});

it('clears context/options and A-to-ineligible-to-A drafts instead of reviving confirmation', async () => {
  const user = userEvent.setup(); select(); render(TableApp); await ready(); await review(user);
  const changed = ownedTurn(); changed.tactical!.attack_options!.hands.hands = ['Free', 'Free'];
  vi.mocked(tableApi.view).mockResolvedValue(changed);
  await user.click(screen.getByRole('button', { name: 'Refresh saved table' })); await ready();
  expect(screen.queryByLabelText('Attack declaration')).toBeNull();
  vi.mocked(tableApi.view).mockResolvedValue(ownedTurn());
  await user.click(screen.getByRole('button', { name: 'Refresh saved table' })); await ready();
  expect((screen.getByLabelText('Attack declaration') as HTMLInputElement).value).toBe('');
  expect(screen.queryByRole('button', { name: 'Attack as described' })).toBeNull();
  await review(user); const pending = ownedTurn(); pending.inspiration_transfer = { character_id: 'pc', choices: [] };
  vi.mocked(tableApi.view).mockResolvedValue(pending); await user.click(screen.getByRole('button', { name: 'Refresh saved table' })); await ready();
  expect(screen.queryByLabelText('Attack declaration')).toBeNull(); expect(tableApi.action).not.toHaveBeenCalled();
});

it('discards a reviewed PC draft across same-player source selection and a pending physical roll', async () => {
  const user = userEvent.setup(); select(); render(TableApp); await ready(); await review(user);
  await user.selectOptions(screen.getByLabelText('Controlled actor'), 'source'); await ready();
  expect(screen.queryByLabelText('Attack declaration')).toBeNull();
  await user.selectOptions(screen.getByLabelText('Controlled actor'), ''); await ready();
  expect((screen.getByLabelText('Attack declaration') as HTMLInputElement).value).toBe('');
  await review(user);
  const pending = ownedTurn(); pending.roll_channel = 'Tactical';
  pending.roll = { id: 'pending-attack', roller: 'pc-actor', dice: [{ count: 1, sides: 20 }], modifier: 5, mode: 'Normal', visibility: 'Public', reason: 'Attack' };
  vi.mocked(tableApi.view).mockResolvedValue(pending);
  await user.click(screen.getByRole('button', { name: 'Refresh saved table' })); await ready();
  expect(screen.queryByLabelText('Attack declaration')).toBeNull(); expect(tableApi.action).not.toHaveBeenCalled();
  expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
});

it.each([false, true])('clears confirmation through a delayed refresh and later campaign change; failure=%s', async failure => {
  const user = userEvent.setup(); select(); render(TableApp); await ready(); await review(user);
  let resolve!: (value: TableView) => void; let reject!: (error: Error) => void;
  const late = new Promise<TableView>((yes, no) => { resolve = yes; reject = no; });
  const second = ownedTurn(); second.campaign_id = 'other-campaign'; second.revision = 'second-revision';
  vi.mocked(tableApi.view).mockReturnValueOnce(late).mockResolvedValue(second);
  // Starting a refresh removes the old component; its late response cannot
  // restore the submitted-for-review draft, even with an unchanged revision.
  await user.click(screen.getByRole('button', { name: 'Refresh saved table' }));
  expect(screen.queryByRole('button', { name: 'Attack as described' })).toBeNull();
  // The campaign selector is locked during refresh, so wait for this response;
  // even a same-revision success may only produce a fresh, empty declaration.
  if (failure) reject(new Error('Read failed')); else resolve(ownedTurn());
  await ready();
  await user.selectOptions(screen.getByLabelText('Campaign'), 'other-campaign'); await ready();
  expect(screen.queryByRole('button', { name: 'Attack as described' })).toBeNull();
  expect(tableApi.action).not.toHaveBeenCalled(); expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
});

it('drops a selected duplicate target when its own current contact changes without an audience revision change', async () => {
  const user = userEvent.setup(); const view = ownedTurn(); view.tactical!.attack_options!.targets.push({ actor: 'second-guard', label: 'Visible guard' });
  view.tactical!.observers = [{ observer: 'pc-actor', position: null, cells: [], contacts: [
    { entity_id: 'guard', label: 'Visible guard', position: { x: 10, y: 0, z: 0 }, status: 'Seen', modality: 'Sight' },
    { entity_id: 'second-guard', label: 'Visible guard', position: { x: 20, y: 0, z: 0 }, status: 'Seen', modality: 'Sight' },
  ] }];
  vi.mocked(tableApi.view).mockResolvedValue(view); select(); render(TableApp); await ready(); await review(user);
  await user.selectOptions(screen.getByLabelText('Attack declaration target'), 'second-guard');
  const next = structuredClone(view); next.tactical!.observers[0].contacts[1].status = 'Remembered';
  vi.mocked(tableApi.view).mockResolvedValue(next); await user.click(screen.getByRole('button', { name: 'Refresh saved table' })); await ready();
  expect((screen.getByLabelText('Attack declaration') as HTMLInputElement).value).toBe('');
  expect(screen.queryByRole('button', { name: 'Attack as described' })).toBeNull();
  await review(user); expect(screen.getByText(/cannot be distinguished from your current view/)).toBeTruthy();
  expect(tableApi.action).not.toHaveBeenCalled();
});

it('preserves the exact accepted-candidate request through uncertain delivery, cleared UI and reload', async () => {
  const user = userEvent.setup(); select(); const mounted = render(TableApp); await ready(); await review(user);
  vi.mocked(tableApi.action).mockRejectedValueOnce({ message: 'Attack delivery uncertain.', retryable: true });
  await user.click(screen.getByRole('button', { name: 'Attack as described' })); await screen.findByText('Attack delivery uncertain.');
  const bytes = localStorage.getItem(REQUEST_KEY)!; const saved = JSON.parse(bytes);
  expect(saved.request.action).toEqual({ Tactical: { action: { Attack: { choice: ordinaryChoice() } } } });
  expect(screen.queryByLabelText('Attack declaration')).toBeNull(); mounted.unmount();
  const later = ownedTurn(); later.revision = 'later'; later.active_session!.session_id = 'later-session';
  later.tactical!.attack_options!.targets = [];
  vi.mocked(tableApi.view).mockResolvedValue(later); select(null, 'source'); render(TableApp); await ready();
  expect(localStorage.getItem(REQUEST_KEY)).toBe(bytes); expect(screen.queryByLabelText('Attack declaration')).toBeNull();
  await user.click(screen.getByRole('button', { name: 'Retry original request' })); await ready();
  expect(tableApi.action).toHaveBeenNthCalledWith(2, saved.request); expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
});

it('clears an unsubmitted draft when another action enters the outbox and preserves that other request', async () => {
  const user = userEvent.setup(); select(); render(TableApp); await ready(); await review(user);
  vi.mocked(tableApi.action).mockRejectedValueOnce({ message: 'Dodge delivery uncertain.', retryable: true });
  await user.click(screen.getByRole('button', { name: 'Dodge' })); await screen.findByText('Dodge delivery uncertain.');
  const saved = localStorage.getItem(REQUEST_KEY)!;
  expect(JSON.parse(saved).request.action).toEqual({ Tactical: { action: 'Dodge' } });
  expect(screen.queryByLabelText('Attack declaration')).toBeNull();
  await tick(); expect(localStorage.getItem(REQUEST_KEY)).toBe(saved); expect(tableApi.action).toHaveBeenCalledOnce();
});

it('keeps existing Talk at the table and saved Text retries on their exact original path', async () => {
  const user = userEvent.setup(); const request = { command_id: 'original-text', campaign_id: 'campaign', version: 5, revision: 'old-revision',
    session_id: 'session', channel: { Player: { player_id: 'player', character_id: 'pc' } }, text: 'I attack Visible guard with Greatsword' };
  localStorage.setItem(REQUEST_KEY, JSON.stringify({ kind: 'text', request })); select();
  vi.mocked(tableApi.text).mockImplementation(async sent => ({ Accepted: { command_id: sent.command_id, revision: 'accepted', outcome: { message: 'Original text received.' } } }));
  render(TableApp); await ready(); expect(screen.queryByLabelText('Attack declaration')).toBeNull();
  await user.click(screen.getByRole('button', { name: 'Retry original request' })); await ready();
  expect(tableApi.text).toHaveBeenCalledExactlyOnceWith(request); expect(tableApi.action).not.toHaveBeenCalled();
  await user.type(screen.getByLabelText('Your declaration, question or correction'), 'Can I attack Visible guard?');
  await user.click(screen.getByRole('button', { name: 'Send to the table' })); await ready();
  expect(tableApi.text).toHaveBeenLastCalledWith(expect.objectContaining({ text: 'Can I attack Visible guard?' }));
  expect(tableApi.action).not.toHaveBeenCalled();
});
