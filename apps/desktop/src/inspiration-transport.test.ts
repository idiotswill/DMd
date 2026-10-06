import { beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import {
  loadRequest, saveRequest, tableApi,
  type RequestContext, type TableAction, type UnconfirmedRequest,
} from './table-api';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
const context: RequestContext = {
  version: 4,
  command_id: 'original-command',
  campaign_id: 'campaign',
  session_id: 'original-session',
  channel: 'Host',
  revision: 'original-visible-token',
};
beforeEach(() => {
  vi.mocked(invoke).mockReset();
  localStorage.clear();
});

describe('Inspiration durable transport', () => {
  it.each(['award', 'reroll'] as const)(
    'preserves the original %s input across uncertain acknowledgement',
    async (kind) => {
      const action: TableAction = kind === 'award'
        ? { AwardHeroicInspiration: { character_id: 'pc', reason: 'A considered GM award.' } }
        : {
            Tactical: {
              action: {
                SubmitRollWithInspiration: {
                  result: {
                    request_id: 'owned-roll',
                    source: 'Physical',
                    dice: [{ sides: 20, value: 20 }],
                  },
                  die_index: 0,
                  replacement: { sides: 20, value: 1 },
                },
              },
            },
          };
      const request = {
        ...context,
        channel: kind === 'award'
          ? 'Host' as const
          : { Player: { player_id: 'player', character_id: 'pc' } },
        action,
      };
      const saved: UnconfirmedRequest = { kind: 'action', request };
      saveRequest(saved);
      vi.mocked(invoke)
        .mockRejectedValueOnce(new Error('lost acknowledgement'))
        .mockResolvedValueOnce({
          Accepted: {
            command_id: context.command_id,
            revision: 'accepted',
            outcome: { message: 'Recorded.' },
          },
        });
      await expect(tableApi.action(request)).rejects.toThrow('lost acknowledgement');
      const retry = loadRequest();
      if (retry?.kind !== 'action') throw new Error('missing saved action');
      expect(retry).toEqual(saved);
      await tableApi.action(retry.request);
      expect(vi.mocked(invoke).mock.calls[0]).toEqual(vi.mocked(invoke).mock.calls[1]);
      const { action: original, ...binding } = request;
      expect(invoke).toHaveBeenLastCalledWith('desktop_submit_table', {
        request: { ...binding, input: { Action: original } },
      });
      expect(JSON.stringify(vi.mocked(invoke).mock.calls)).not.toContain('expected_event_sequence');
    },
  );
});
