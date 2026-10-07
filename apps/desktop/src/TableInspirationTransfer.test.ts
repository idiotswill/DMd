import { render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, it, expect, vi } from 'vitest';
import TableApp from './TableApp.svelte';
import { contract, emptyView, options } from './components/table-fixtures.test-support';
import { REQUEST_KEY, SELECTION_KEY, tableApi, type TableView } from './table-api';

vi.mock('./table-api', async original => ({ ...await original<typeof import('./table-api')>(), tableApi: { defaults: vi.fn(), list: vi.fn(), create: vi.fn(), view: vi.fn(), options: vi.fn(), situation: vi.fn(), action: vi.fn(), text: vi.fn(), rollOptions: vi.fn(), creatureOptions: vi.fn(), sourceControlOptions: vi.fn() } }));
function table(): TableView {
  return { ...emptyView(), grapple: { version: 4, choices: [], ground_drag: [] },
    players: [{ id: 'owner', campaign_id: 'campaign', display_name: 'Sam' }, { id: 'other', campaign_id: 'campaign', display_name: 'Alex' }],
    characters: [{ character_id: 'pc', entity_id: 'actor', player_id: 'owner', name: 'River', profile: {}, sheet: null, details: { heroic_inspiration: true }, second_wind_remaining: null }, { character_id: 'ally', entity_id: 'ally-actor', player_id: 'other', name: 'Ash', profile: null, sheet: null, details: null, second_wind_remaining: null }] as unknown as TableView['characters'],
    active_session: { session_id: 'session', display_name: 'Evening', started_at_world: 0, participants: [{ player_id: 'owner', character_id: 'pc', attendance: 'Present' }, { player_id: 'other', character_id: 'ally', attendance: 'Present' }] } };
}
function pending(): TableView {
  return { ...table(), inspiration_transfer: { character_id: 'pc', choices: [{ key: 'original-gift', label: 'Give to Ash' }, { key: 'original-decline', label: 'Decline the extra Inspiration' }] } };
}
beforeEach(() => {
  localStorage.clear(); vi.resetAllMocks();
  vi.mocked(tableApi.defaults).mockResolvedValue(structuredClone(contract));
  vi.mocked(tableApi.list).mockResolvedValue([{ id: 'campaign', name: 'Saved campaign' }]);
  vi.mocked(tableApi.options).mockResolvedValue(options);
  vi.mocked(tableApi.situation).mockResolvedValue({ title: '', description: '', challenges: [] });
  vi.mocked(tableApi.rollOptions).mockResolvedValue({ savage_attacker: null });
  vi.mocked(tableApi.creatureOptions).mockResolvedValue([]);
});

describe('optional excess award through the actual desktop', () => {
  it('uses the same Host form for an inspired PC and preserves the excess award across restart', async () => {
    const user = userEvent.setup(); const view = table();
    vi.mocked(tableApi.view).mockResolvedValue(view);
    vi.mocked(tableApi.action).mockRejectedValueOnce({ message: 'Delivery uncertain.', retryable: true }).mockImplementationOnce(async request => ({ command_id: request.command_id, revision: 'accepted', outcome: { message: 'Extra awarded.' } }));
    localStorage.setItem(SELECTION_KEY, JSON.stringify({ campaignId: 'campaign', playerId: null }));
    const mounted = render(TableApp);
    await waitFor(() => expect(screen.getByLabelText('Recipient').matches(':disabled')).toBe(false));
    expect((screen.getByRole('option', { name: 'River — already inspired' }) as HTMLOptionElement).disabled).toBe(false);
    await user.selectOptions(screen.getByLabelText('Recipient'), 'pc');
    await user.type(screen.getByLabelText('Reason for the award'), 'For helping a companion.');
    await user.click(screen.getByRole('button', { name: 'Award Heroic Inspiration' }));
    await screen.findByText('Delivery uncertain.');
    const saved = JSON.parse(localStorage.getItem(REQUEST_KEY)!);
    expect(saved.request).toMatchObject({ version: 4, channel: 'Host', revision: view.revision, session_id: 'session', action: { AwardExcessInspiration: { character_id: 'pc', reason: 'For helping a companion.' } } });
    mounted.unmount();
    vi.mocked(tableApi.view).mockResolvedValue({ ...pending(), revision: 'later', active_session: { ...view.active_session!, session_id: 'later-session' } });
    render(TableApp);
    await waitFor(() => expect(screen.getByRole('button', { name: 'Retry original request' }).matches(':disabled')).toBe(false));
    await user.click(screen.getByRole('button', { name: 'Retry original request' }));
    await waitFor(() => expect(localStorage.getItem(REQUEST_KEY)).toBeNull());
    expect(tableApi.action).toHaveBeenNthCalledWith(1, saved.request);
    expect(tableApi.action).toHaveBeenNthCalledWith(2, saved.request);
  });

  it.each(['give', 'decline'] as const)('saves and retries the owner’s original %s handle after a later view and local actor preference', async kind => {
    const user = userEvent.setup(); const view = pending();
    view.source_control = { version: 2, actors: [{ actor: 'mage', name: 'Mage', definition_id: 'mage', controller: { Player: 'owner' }, hp: 81, max_hp: 81 }] };
    vi.mocked(tableApi.view).mockResolvedValue(view);
    vi.mocked(tableApi.action).mockRejectedValueOnce({ message: 'Choice delivery uncertain.', retryable: true }).mockImplementationOnce(async request => ({ command_id: request.command_id, revision: 'accepted', outcome: { message: 'Choice recorded.' } }));
    localStorage.setItem(SELECTION_KEY, JSON.stringify({ campaignId: 'campaign', playerId: 'owner' }));
    const mounted = render(TableApp);
    const label = kind === 'give' ? 'Give to Ash' : 'Decline the extra Inspiration';
    await waitFor(() => expect(screen.getByRole('button', { name: label }).matches(':disabled')).toBe(false));
    expect(screen.getByText(/Your original Inspiration stays/)).toBeTruthy();
    await user.click(screen.getByRole('button', { name: label }));
    await screen.findByText('Choice delivery uncertain.');
    const saved = JSON.parse(localStorage.getItem(REQUEST_KEY)!);
    expect(saved.request).toMatchObject({ version: 4, channel: { Player: { player_id: 'owner', character_id: 'pc' } }, revision: view.revision, session_id: 'session', action: { InspirationTransfer: { handle: kind === 'give' ? 'original-gift' : 'original-decline' } } });
    mounted.unmount();
    localStorage.setItem(SELECTION_KEY, JSON.stringify({ campaignId: 'campaign', playerId: 'owner', sourceActorId: 'mage' }));
    vi.mocked(tableApi.view).mockResolvedValue({ ...table(), source_control: view.source_control, revision: 'later', active_session: { ...view.active_session!, session_id: 'later-session' } });
    render(TableApp);
    await waitFor(() => expect(screen.getByRole('button', { name: 'Retry original request' }).matches(':disabled')).toBe(false));
    expect((screen.getByLabelText('Controlled actor') as HTMLSelectElement).value).toBe('');
    await user.click(screen.getByRole('button', { name: 'Retry original request' }));
    await waitFor(() => expect(localStorage.getItem(REQUEST_KEY)).toBeNull());
    expect(tableApi.action).toHaveBeenNthCalledWith(1, saved.request);
    expect(tableApi.action).toHaveBeenNthCalledWith(2, saved.request);
  });

  it('gives Host no selection, prevents session closure, and clears recipient exposure when switching to another player', async () => {
    const user = userEvent.setup();
    vi.mocked(tableApi.view).mockImplementation(async (_campaign, viewer) => {
      if (viewer === 'Host') return { ...pending(), inspiration_transfer: { character_id: 'pc', choices: [] } };
      return typeof viewer === 'object' && viewer.Player === 'owner' ? pending() : table();
    });
    localStorage.setItem(SELECTION_KEY, JSON.stringify({ campaignId: 'campaign', playerId: null }));
    render(TableApp);
    await screen.findByText(/River's player must choose/);
    expect(screen.queryByRole('button', { name: 'Give to Ash' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Decline the extra Inspiration' })).toBeNull();
    expect(screen.getByLabelText('Recipient').matches(':disabled')).toBe(true);
    await user.click(screen.getByRole('button', { name: 'Setup and host controls' }));
    expect(screen.getByRole('button', { name: 'End and save session' }).matches(':disabled')).toBe(true);
    await user.selectOptions(screen.getByLabelText('Local viewing and input channel'), 'owner');
    await waitFor(() => expect(screen.getByRole('button', { name: 'Give to Ash' }).matches(':disabled')).toBe(false));
    expect(screen.getByRole('button', { name: 'Send to the table' }).matches(':disabled')).toBe(false);
    await user.selectOptions(screen.getByLabelText('Local viewing and input channel'), 'other');
    await waitFor(() => expect(tableApi.view).toHaveBeenLastCalledWith('campaign', { Player: 'other' }));
    expect(screen.queryByText('Extra Heroic Inspiration')).toBeNull();
    expect(screen.queryByRole('button', { name: 'Give to Ash' })).toBeNull();
    expect(tableApi.action).not.toHaveBeenCalled();
  });

  it('keeps the choice on the PC channel and replaces unsubmitted handles when refreshed', async () => {
    const user = userEvent.setup(); let view = pending();
    view.source_control = { version: 2, actors: [{ actor: 'mage', name: 'Mage', definition_id: 'mage', controller: { Player: 'owner' }, hp: 81, max_hp: 81 }] };
    vi.mocked(tableApi.view).mockImplementation(async () => structuredClone(view));
    vi.mocked(tableApi.action).mockRejectedValue({ message: 'Delivery uncertain.', retryable: true });
    localStorage.setItem(SELECTION_KEY, JSON.stringify({ campaignId: 'campaign', playerId: 'owner' }));
    render(TableApp);
    await waitFor(() => expect(screen.getByLabelText('Controlled actor').matches(':disabled')).toBe(false));
    await user.selectOptions(screen.getByLabelText('Controlled actor'), 'mage');
    await screen.findByText('Select your player character to choose what happens to the extra Inspiration.');
    expect(screen.queryByRole('button', { name: 'Give to Ash' })).toBeNull();
    await user.selectOptions(screen.getByLabelText('Controlled actor'), '');
    view = { ...view, revision: 'new-choice-view', inspiration_transfer: { character_id: 'pc', choices: [{ key: 'new-decline', label: 'Decline the extra Inspiration' }] } };
    await user.click(screen.getByRole('button', { name: 'Refresh saved table' }));
    await waitFor(() => expect(screen.queryByRole('button', { name: 'Give to Ash' })).toBeNull());
    await waitFor(() => expect(screen.getByRole('button', { name: 'Decline the extra Inspiration' }).matches(':disabled')).toBe(false));
    await user.click(screen.getByRole('button', { name: 'Decline the extra Inspiration' }));
    await screen.findByText('Delivery uncertain.');
    expect(vi.mocked(tableApi.action).mock.calls[0][0]).toMatchObject({ revision: 'new-choice-view', channel: { Player: { player_id: 'owner', character_id: 'pc' } }, action: { InspirationTransfer: { handle: 'new-decline' } } });
  });
});
