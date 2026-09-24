import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { svelteTesting } from '@testing-library/svelte/vite';

export default defineConfig({
  // Keep cleanup in our local setup module, including when npm dependencies are junctioned.
  plugins: [svelte(), svelteTesting({ autoCleanup: false })],
  clearScreen: false,
  build: { target: 'es2022', sourcemap: false },
  test: {
    environment: 'jsdom',
    include: ['src/**/*.test.ts'],
    setupFiles: ['src/test-setup.ts'],
    server: { deps: { inline: ['@testing-library/svelte'] } },
  },
});
