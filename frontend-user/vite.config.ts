import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [sveltekit()],
  server: {
    host: '0.0.0.0',
    port: 3000,
  },
  define: {
    'import.meta.env.VITE_API_URL': JSON.stringify(process.env.PUBLIC_API_URL || 'http://localhost:8080'),
    'import.meta.env.VITE_WS_URL': JSON.stringify(process.env.PUBLIC_WS_URL || 'ws://localhost:8080'),
  },
});
