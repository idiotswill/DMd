import { fireEvent, render, screen, within } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { expect, it, vi } from 'vitest';
import BattlefieldForm from './BattlefieldForm.svelte';
import TacticalMap from './TacticalMap.svelte';
import type { CharacterView } from '../table-api';
import type { BattlefieldSetup, TacticalView } from '../tactical-api';

function character(): CharacterView {
  return {
    character_id: 'pc', player_id: 'player', entity_id: 'actor', name: 'Hero',
    profile: null, sheet: null, details: null, second_wind_remaining: null,
    equipment: { prepared: true, initial_item_count: 0, items: [], worn_armor: null,
      shield: null, hands: { hands: ['Free', 'Free'] } },
  };
}

it('preserves the ground-level setup while keeping elevation separate from body height', async () => {
  const user = userEvent.setup(); const onPrepare = vi.fn();
  render(BattlefieldForm, { characters: [character()], onPrepare });
  expect((screen.getByLabelText('Hero: elevation (feet)') as HTMLInputElement).value).toBe('0');
  expect((screen.getByLabelText('Hero: height (feet)') as HTMLInputElement).value).toBe('6');
  await user.click(screen.getByRole('button', { name: 'Prepare battlefield' }));
  expect(onPrepare).toHaveBeenCalledTimes(1);
  const setup = onPrepare.mock.calls[0][0] as BattlefieldSetup;
  expect(setup.characters).toEqual([{ character_id: 'pc', position: { x: 10, y: 10, z: 0 },
    height: 12, allies: [], enemies: [] }]);
  expect(setup.battlefield).toEqual({
    bounds: { min: { x: 0, y: 0, z: 0 }, max: { x: 100, y: 100, z: 80 } },
    floor_z: 0, floor_surface: 'ground', ambient_light: 'Bright',
    obstacles: [], terrain: [], lights: [],
  });
  expect(setup.area_grid_policy).toBeNull();
  expect(setup.geometry_ruling).toEqual({ basis: 'GmAdjudication',
    reason: 'The host established the terrain and starting positions from the current scene.' });
});

it('authors an elevated ledge, separated dry bridge tops and water through the ordinary form', async () => {
  const user = userEvent.setup(); const onPrepare = vi.fn();
  render(BattlefieldForm, { characters: [character()],
    creatures: [{ actor: 'source', name: 'Private source name', definition_id: 'chimera',
      size: 'Large', hp: 114, max_hp: 114 }], onPrepare });
  const enter = async (element: HTMLElement, value: string) => {
    await user.clear(element); await user.type(element, value);
  };
  await enter(screen.getByLabelText('Hero: elevation (feet)'), '20');
  await enter(screen.getByLabelText('Private source name: elevation (feet)'), '20');
  await enter(screen.getByLabelText('Private source name: height (feet)'), '10');
  await enter(screen.getByLabelText('Private source name: visible description'), 'Three-headed creature');
  const definitions = [
    { kind: 'wall', x: '0', y: '0', width: '10', depth: '15', bottom: '0', height: '20' },
    { kind: 'water', x: '10', y: '0', width: '30', depth: '15', bottom: '0', height: '5' },
    { kind: 'platform', x: '10', y: '0', width: '10', depth: '15', bottom: '19.5', height: '0.5' },
    { kind: 'platform', x: '25', y: '0', width: '15', depth: '15', bottom: '19.5', height: '0.5' },
  ];
  for (const [index, definition] of definitions.entries()) {
    await user.click(screen.getByRole('button', { name: 'Add terrain region' }));
    const region = within(screen.getByRole('group', { name: `Region ${index + 1}` }));
    await user.selectOptions(region.getByLabelText('Kind'), definition.kind);
    for (const [label, value] of [
      ['East (feet)', definition.x], ['South (feet)', definition.y],
      ['Width (feet)', definition.width], ['Depth (feet)', definition.depth],
      ['Height (feet)', definition.height], ['Bottom elevation (feet)', definition.bottom],
    ]) await enter(region.getByLabelText(label), value);
  }
  await user.click(screen.getByRole('checkbox', { name: /Use occupied-space sampling/ }));
  await user.click(screen.getByRole('button', { name: 'Prepare battlefield' }));
  expect(onPrepare).toHaveBeenCalledTimes(1);
  const setup = onPrepare.mock.calls[0][0] as BattlefieldSetup;
  expect(setup.characters).toEqual([{ character_id: 'pc', position: { x: 10, y: 10, z: 40 },
    height: 12, allies: [], enemies: ['source'] }]);
  expect(setup.creatures).toEqual([{ actor: 'source', public_label: 'Three-headed creature',
    position: { x: 50, y: 10, z: 40 }, height: 20, allies: [], enemies: ['actor'] }]);
  expect(JSON.stringify(setup)).not.toContain('Private source name');
  expect(setup.area_grid_policy).toBe('OccupiedCellCentersV1');
  expect(setup.battlefield.obstacles).toEqual([{ id: 'wall-0',
    volume: { min: { x: 0, y: 0, z: 0 }, max: { x: 20, y: 30, z: 40 } },
    blocks_movement: true, blocks_sight: true, observable: true, cover: 'Total' }]);
  const terrain = setup.battlefield.terrain;
  expect(terrain).toHaveLength(3);
  expect(terrain[0]).toEqual({ id: 'terrain-0',
    volume: { min: { x: 20, y: 0, z: 0 }, max: { x: 80, y: 30, z: 10 } },
    difficult: false, observable: true, water: true, climbable: false, burrowable: false,
    supports_top: false, surface: 'terrain-0', obscuration: 'None', magical_darkness: false });
  expect(terrain.slice(1).map(region => ({ volume: region.volume, water: region.water,
    supports_top: region.supports_top, difficult: region.difficult }))).toEqual([
    { volume: { min: { x: 20, y: 0, z: 39 }, max: { x: 40, y: 30, z: 40 } },
      water: false, supports_top: true, difficult: false },
    { volume: { min: { x: 50, y: 0, z: 39 }, max: { x: 80, y: 30, z: 40 } },
      water: false, supports_top: true, difficult: false },
  ]);
});

it.each([
  ['Hero: elevation (feet)', '35'],
  ['Hero: elevation (feet)', '0.25'],
  ['Hero: elevation (feet)', ''],
  ['Hero: elevation (feet)', '1e309'],
  ['Hero: height (feet)', '0'],
  ['Hero: east (feet)', '49'],
])('refuses invalid occupied geometry for %s=%s even when browser validation is bypassed', async (label, value) => {
  const onPrepare = vi.fn();
  const { container } = render(BattlefieldForm, { characters: [character()], onPrepare });
  await fireEvent.input(screen.getByLabelText(label), { target: { value } });
  await fireEvent.submit(container.querySelector('form')!);
  expect(onPrepare).not.toHaveBeenCalled();
  expect(screen.getByRole('alert').textContent).toContain('including their height');
});

it('refuses terrain outside the vertical bounds and permits correction without rounding', async () => {
  const user = userEvent.setup(); const onPrepare = vi.fn();
  const { container } = render(BattlefieldForm, { characters: [character()], onPrepare });
  await user.click(screen.getByRole('button', { name: 'Add terrain region' }));
  const region = within(screen.getByRole('group', { name: 'Region 1' }));
  await user.selectOptions(region.getByLabelText('Kind'), 'platform');
  await fireEvent.input(region.getByLabelText('Bottom elevation (feet)'), { target: { value: '31' } });
  await fireEvent.submit(container.querySelector('form')!);
  expect(onPrepare).not.toHaveBeenCalled();
  expect(screen.getByRole('alert').textContent).toContain('including its top');
  await fireEvent.input(region.getByLabelText('Bottom elevation (feet)'), { target: { value: '29.75' } });
  await fireEvent.submit(container.querySelector('form')!);
  expect(onPrepare).not.toHaveBeenCalled();
  await fireEvent.input(region.getByLabelText('Bottom elevation (feet)'), { target: { value: '29.5' } });
  await user.click(screen.getByRole('button', { name: 'Prepare battlefield' }));
  expect(onPrepare).toHaveBeenCalledTimes(1);
  expect(onPrepare.mock.calls[0][0].battlefield.terrain[0].volume).toEqual({
    min: { x: 40, y: 40, z: 59 }, max: { x: 50, y: 50, z: 79 },
  });
});

it('keeps elevated preparation disabled while another operation is pending', async () => {
  const user = userEvent.setup(); const onPrepare = vi.fn();
  const { container } = render(BattlefieldForm, { characters: [character()], disabled: true, onPrepare });
  expect(screen.getByLabelText('Hero: elevation (feet)').closest('fieldset')?.disabled).toBe(true);
  await user.click(screen.getByRole('button', { name: 'Prepare battlefield' }));
  await fireEvent.submit(container.querySelector('form')!);
  expect(onPrepare).not.toHaveBeenCalled();
});

it('shows authorized elevations and remembered altitude without retaining Host terrain on a player view', async () => {
  const host: TacticalView = {
    encounter_id: 'encounter', round: 1, active_actor: 'actor', phase: 'active',
    battlefield: { bounds: { min: { x: 0, y: 0, z: 0 }, max: { x: 100, y: 100, z: 80 } },
      floor_z: 0, floor_surface: 'ground', ambient_light: 'Bright', obstacles: [], lights: [],
      terrain: [{ id: 'pool', volume: { min: { x: 0, y: 0, z: 0 }, max: { x: 40, y: 40, z: 10 } },
        difficult: false, observable: true, water: true, climbable: false, burrowable: false,
        supports_top: false, surface: 'pool', obscuration: 'None', magical_darkness: false }] },
    participants: [{ entity_id: 'secret', public_label: 'Hidden flyer', position: { x: 60, y: 50, z: 60 }, size: 'Large' }],
    observers: [], initiative: [], ties: [], budget: null, continuation: null,
    may_fail_save: null, legendary_resistance: null, legendary_action: null, combatant_sources: [],
  };
  const component = render(TacticalMap, { tactical: host, characters: [], actor: null });
  expect(screen.getByRole('listitem').textContent).toBe('Hidden flyer: 30 feet east, 25 feet south, 30 feet elevation');
  expect(screen.getByText('Water: 0 to 5 feet elevation')).toBeTruthy();
  const player: TacticalView = { ...host, battlefield: null, participants: [], observers: [{
    observer: 'actor', position: { x: 10, y: 10, z: 0 }, cells: [], contacts: [{
      entity_id: 'secret', label: 'Known flyer', position: { x: 20, y: 30, z: 20 },
      status: 'Remembered', modality: 'Sight',
    }],
  }] };
  await component.rerender({ tactical: player, actor: 'actor' });
  expect(screen.getAllByRole('listitem').map(item => item.textContent)).toEqual([
    'You: 5 feet east, 5 feet south',
    'Known flyer: 10 feet east, 15 feet south, 10 feet elevation (last known)',
  ]);
  expect(screen.queryByText(/Hidden flyer/)).toBeNull();
  expect(screen.queryByText(/Water: 0 to 5/)).toBeNull();
  expect(screen.queryByText(/30 feet elevation/)).toBeNull();
  await component.rerender({ actor: 'another-player' });
  expect(screen.queryAllByRole('listitem')).toHaveLength(0);
  expect(screen.queryByText(/Known flyer/)).toBeNull();
});
