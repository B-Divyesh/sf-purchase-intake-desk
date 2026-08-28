import { defineConfig, type Plugin } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';

const buildSha = process.env.BUILD_SHA || process.env.GIT_SHA || process.env.SOURCE_COMMIT || 'dev';

const versionServiceWorker = (): Plugin => ({
  name: 'version-service-worker',
  apply: 'build',
  async closeBundle() {
    const path = resolve('dist/sw.js');
    const source = await readFile(path, 'utf8');
    await writeFile(path, source.replaceAll('__BUILD_SHA__', buildSha.slice(0, 16)));
  },
});

export default defineConfig({
  plugins: [svelte(), versionServiceWorker()],
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
