import { resolve } from 'node:path';
import { fileURLToPath, URL } from 'node:url';

import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

const r = (p: string) => fileURLToPath(new URL(p, import.meta.url));

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  envPrefix: ['VITE_', 'TAURI_ENV_'],
  resolve: {
    alias: {
      '@': r('./src'),
      '@components': r('./src/components'),
      '@features': r('./src/features'),
      '@windows': r('./src/windows'),
      '@lib': r('./src/lib'),
      '@i18n': r('./src/i18n'),
      '@styles': r('./src/styles'),
    },
  },
  server: {
    port: 1420,
    strictPort: true,
    host: process.env.TAURI_DEV_HOST || false,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
  build: {
    target: 'safari14',
    minify: !process.env.TAURI_ENV_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'src/windows/main/index.html'),
        popup: resolve(__dirname, 'src/windows/popup/index.html'),
        menubar: resolve(__dirname, 'src/windows/menubar/index.html'),
      },
    },
  },
});
