import { fileURLToPath, URL } from 'url';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import environment from 'vite-plugin-environment';
import dotenv from 'dotenv';
import inject from '@rollup/plugin-inject';

dotenv.config({ path: '../../.env' });

export default defineConfig({
	build: {
		emptyOutDir: true,
		rollupOptions: {
			plugins: [
				inject({
					modules: { Buffer: ['buffer', 'Buffer'] }
				})
			]
		}
	},
	optimizeDeps: {
		esbuildOptions: {
			define: {
				global: 'globalThis'
			}
		}
	},
	server: {
		proxy: {
			'/api': {
				target: 'http://127.0.0.1:8080',
				changeOrigin: true
			}
		}
	},
	plugins: [
		sveltekit(),
		environment('all', { prefix: 'CANISTER_' }),
		environment('all', { prefix: 'DFX_' })
	],
	resolve: {
		alias: [
			{
				find: 'declarations',
				replacement: fileURLToPath(new URL('../declarations', import.meta.url))
			}
		]
	}
});
