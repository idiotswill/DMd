import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { expect, it, vi } from 'vitest';
import type { TacticalView } from '../tactical-api';
import EncounterPanel from './EncounterPanel.svelte';

function tactical(): TacticalView {
  return { encounter_id: 'encounter', round: 1, active_actor: 'other-turn', phase:'active',execution:'ReactionsV1', battlefield: null, participants: [], combatant_sources: [], observers: [], initiative: [], ties: [], continuation: null, may_fail_save: null, legendary_resistance: null, legendary_action: null, budget: null, liquid_landing: { actor: 'falling' } };
}

it('offers each explicit owned landing choice without sending geometry or a target', async () => {
  const user = userEvent.setup(); const onAction = vi.fn();
  const view = tactical();
  const component = render(EncounterPanel, { tactical: view, characters: [], host: false, actor: 'falling', player: 'owner', onAction });
  await user.click(screen.getByRole('button', { name: 'Use Athletics' }));
  expect(onAction).toHaveBeenLastCalledWith({ ChooseLiquidLanding: { choice: 'Athletics' } });
  await user.click(screen.getByRole('button', { name: 'Use Acrobatics' }));
  expect(onAction).toHaveBeenLastCalledWith({ ChooseLiquidLanding: { choice: 'Acrobatics' } });
  await user.click(screen.getByRole('button', { name: 'Decline landing Reaction' }));
  expect(onAction).toHaveBeenLastCalledWith({ ChooseLiquidLanding: { choice: null } });
  expect(screen.queryByRole('button', { name: 'Choose to fail this save' })).toBeNull();
  await component.rerender({ actor: 'other-turn', player: 'other' });
  expect(screen.queryByText('Landing in liquid')).toBeNull();
  await component.rerender({ actor: null, host: true });
  expect(screen.getByText('Landing in liquid')).toBeTruthy();
  await component.rerender({ host: false, actor: 'falling', player: 'owner', tactical: { ...view, liquid_landing: null } });
  expect(screen.queryByText('Landing in liquid')).toBeNull();
});

it('blocks input while a command or raw roll is pending and retains no automatic selection', async () => {
  const user = userEvent.setup(); const onAction = vi.fn();
  const component = render(EncounterPanel, { tactical: tactical(), characters: [], host: false, actor: 'falling', player: 'owner', disabled: true, onAction });
  for (const name of ['Use Athletics', 'Use Acrobatics', 'Decline landing Reaction']) {
    const button = screen.getByRole('button', { name });
    expect(button.matches(':disabled')).toBe(true);
    await user.click(button);
  }
  expect(onAction).not.toHaveBeenCalled();
  await component.rerender({ disabled: false, pendingRoll: true });
  expect(screen.getByRole('button', { name: 'Use Athletics' }).matches(':disabled')).toBe(true);
  await component.rerender({ pendingRoll: false });
  expect(onAction).not.toHaveBeenCalled();
  await user.click(screen.getByRole('button', { name: 'Use Acrobatics' }));
  expect(onAction).toHaveBeenCalledTimes(1);
});
