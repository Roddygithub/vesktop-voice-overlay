import { defineConfig } from 'tsup';

export default defineConfig({
  entry: ['src/index.ts'],
  format: ['cjs', 'esm'],
  dts: true,
  splitting: false,
  sourcemap: true,
  clean: true,
  minify: false,
  target: 'node20',
  platform: 'node',
  external: ['@vencord/api'],
  esbuildOptions(options) {
    options.banner = {
      js: '/* Vesktop Voice Overlay Plugin - GPL-3.0 */'
    };
  }
});
