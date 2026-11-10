import { defineConfig } from 'vite'
import svelte from 'rollup-plugin-svelte'

// https://vite.dev/config/
export default defineConfig(({ command }) => ({
  plugins: [svelte({ emitCss: command === 'build' })],
  server: {
    proxy: {
      '/api': 'http://127.0.0.1:3000',
      '/auth': 'http://127.0.0.1:3000'
    }
  }
}))
