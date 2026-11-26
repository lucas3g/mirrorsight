import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const { version } = JSON.parse(readFileSync(new URL('./package.json', import.meta.url), 'utf8')) as {
  version: string;
};

// Tauri espera porta fixa e não quer que o Vite limpe o terminal (logs do Rust).
export default defineConfig({
  plugins: [svelte()],
  // Versão vem do package.json (o workflow de release atualiza todos os manifestos juntos).
  define: { __APP_VERSION__: JSON.stringify(version) },
  resolve: {
    alias: { $lib: fileURLToPath(new URL('./src/lib', import.meta.url)) },
  },
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  envPrefix: ['VITE_', 'TAURI_ENV_'],
  build: { target: 'es2022', sourcemap: false },
});
