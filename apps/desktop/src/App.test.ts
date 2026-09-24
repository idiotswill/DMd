import { render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, expect, test, vi } from 'vitest';
import App from './App.svelte';
import { loadDesktopStatus } from './bridge';

// Renderer failure handling only. These mocks are never packaged or gameplay acceptance.
vi.mock('./bridge', () => ({ loadDesktopStatus: vi.fn() }));
const load = vi.mocked(loadDesktopStatus);
beforeEach(() => load.mockReset());

test('connection failure remains recoverable through a keyboard-operable button', async () => {
  load.mockRejectedValueOnce(new Error('IPC disconnected'));
  load.mockResolvedValueOnce({ appName: 'DMd', version: '0.1.0', runtimeReady: false, message: 'Table controls are not connected in this build.' });
  render(App);
  const retry = await screen.findByRole('button', { name: 'Retry connection' });
  retry.focus();
  await userEvent.keyboard('{Enter}');
  await screen.findByText('Table controls are not connected in this build.');
  expect(load).toHaveBeenCalledTimes(2);
  expect(screen.queryByRole('button', { name: 'Retry connection' })).toBeNull();
});

test('pending connection cannot offer gameplay or duplicate retry', async () => {
  load.mockReturnValue(new Promise(() => {}));
  render(App);
  await waitFor(() => expect(load).toHaveBeenCalledTimes(1));
  expect(screen.getByRole('status').textContent).toContain('Connecting');
  expect(screen.queryByRole('button')).toBeNull();
  expect(screen.getByRole('link', { name: 'Skip to main content' }).getAttribute('href')).toBe('#main');
});
