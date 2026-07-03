import adapter from '@sveltejs/adapter-node';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({
      precompress: false,
      envPrefix: 'PUBLIC_',
    }),
    csrf: {
      checkOrigin: true,
    },
    alias: {
      $lib: './src/lib',
      $components: './src/lib/components',
    },
  },
  vitePlugin: {
    inspector: false,
  },
};

export default config;
