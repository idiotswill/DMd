import { afterEach, beforeEach, vi } from 'vitest';
import { cleanup } from '@testing-library/svelte';

beforeEach(()=>{
  // A direct mock stays inert when a test resets all mocks; resetting a spy
  // restores jsdom's unimplemented scrollTo implementation.
  window.scrollTo=vi.fn();
  // jsdom has no layout/scrolling; native packaged verification covers geometry.
  Element.prototype.scrollIntoView=vi.fn();
});
afterEach(()=>{cleanup();vi.restoreAllMocks();});
