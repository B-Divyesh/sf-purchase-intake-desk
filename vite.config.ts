import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

const buildSha = process.env.BUILD_SHA || process.env.GIT_SHA || process.env.SOURCE_COMMIT || 'dev';

export default defineConfig({
  plugins: [svelte()],
  build: {
    target: 'es2022',
    outDir: 'dist',
    sourcemap: true,
  },
  define: {
    'import.meta.env.VITE_BUILD_SHA': JSON.stringify(buildSha),
  },
  server: {
    port: 5173,
    strictPort: true,
  },
});
