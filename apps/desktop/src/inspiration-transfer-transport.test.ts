import { beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { loadRequest, saveRequest, tableApi, REQUEST_KEY, type RequestContext, type TableAction, type UnconfirmedRequest } from './table-api';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
const context: RequestContext = { version: 4, command_id: 'original-command', campaign_id: 'campaign', session_id: 'original-session', channel: 'Host', revision: 'original-view' };
beforeEach(() => { localStorage.clear(); vi.mocked(invoke).mockReset(); });

describe('excess Inspiration durable transport', () => {
  it.each(['award', 'give', 'decline'] as const)('keeps the complete original %s request after uncertain acknowledgement', async kind => {
    const action: TableAction = kind === 'award'
      ? { AwardExcessInspiration: { character_id: 'pc', reason: 'For protecting a companion.' } }
      : { InspirationTransfer: { handle: kind === 'give' ? 'opaque-gift' : 'opaque-decline' } };
    const request = { ...context, channel: kind === 'award' ? 'Host' as const : { Player: { player_id: 'owner', character_id: 'pc' } }, action };
    const saved: UnconfirmedRequest = { kind: 'action', request };
    saveRequest(saved);
    vi.mocked(invoke).mockRejectedValueOnce(new Error('lost acknowledgement')).mockResolvedValueOnce({ Accepted: { command_id: context.command_id, revision: 'accepted', outcome: { message: 'Recorded.' } } });
    await expect(tableApi.action(request)).rejects.toThrow('lost acknowledgement');
    const retry = loadRequest();
    if (retry?.kind !== 'action') throw new Error('missing original request');
    expect(retry).toEqual(saved);
    await tableApi.action(retry.request);
    expect(vi.mocked(invoke).mock.calls[0]).toEqual(vi.mocked(invoke).mock.calls[1]);
    const { action: original, ...binding } = request;
    expect(invoke).toHaveBeenLastCalledWith('desktop_submit_table', { request: { ...binding, input: kind === 'award' ? { Action: original } : { InspirationTransfer: { handle: kind === 'give' ? 'opaque-gift' : 'opaque-decline' } } } });
    expect(JSON.stringify(vi.mocked(invoke).mock.calls)).not.toContain('expected_event_sequence');
    expect(JSON.stringify(vi.mocked(invoke).mock.calls)).not.toContain('ResolveHostInspirationTransfer');
  });

  it.each([
    ['old version', { version: 3 }],
    ['no session', { session_id: null }],
    ['Host choice', { channel: 'Host' }],
    ['source choice', { channel: { SourceCreature: { player_id: 'owner', actor: 'mage' } } }],
    ['missing handle', { action: { InspirationTransfer: {} } }],
    ['blank handle', { action: { InspirationTransfer: { handle: ' ' } } }],
    ['invented recipient', { action: { InspirationTransfer: { handle: 'opaque', recipient: 'pc' } } }],
    ['canonical choice', { action: { ResolveHostInspirationTransfer: { choice: { award: 'award', character_id: 'pc', recipient: 'other' } } } }],
  ] as const)('retains incompatible %s input without invoking it', (_label, patch) => {
    const raw = JSON.stringify({ kind: 'action', request: { ...context, channel: { Player: { player_id: 'owner', character_id: 'pc' } }, action: { InspirationTransfer: { handle: 'opaque' } }, ...patch } });
    localStorage.setItem(REQUEST_KEY, raw);
    expect(() => loadRequest()).toThrow('The saved retry is incomplete or incompatible.');
    expect(localStorage.getItem(REQUEST_KEY)).toBe(raw);
    expect(invoke).not.toHaveBeenCalled();
  });

  it.each([
    ['old version', { version: 3 }],
    ['no session', { session_id: null }],
    ['Player issuer', { channel: { Player: { player_id: 'owner', character_id: 'pc' } } }],
    ['blank reason', { action: { AwardExcessInspiration: { character_id: 'pc', reason: ' ' } } }],
    ['oversized UTF-8 reason', { action: { AwardExcessInspiration: { character_id: 'pc', reason: 'é'.repeat(1001) } } }],
    ['Host-picked recipient', { action: { AwardExcessInspiration: { character_id: 'pc', reason: 'A gift.', recipient: 'other' } } }],
  ] as const)('retains incompatible excess award %s without inventing a replacement', (_label, patch) => {
    const raw = JSON.stringify({ kind: 'action', request: { ...context, action: { AwardExcessInspiration: { character_id: 'pc', reason: 'A gift.' } }, ...patch } });
    localStorage.setItem(REQUEST_KEY, raw);
    expect(() => loadRequest()).toThrow('The saved retry is incomplete or incompatible.');
    expect(localStorage.getItem(REQUEST_KEY)).toBe(raw);
    expect(invoke).not.toHaveBeenCalled();
  });
});
