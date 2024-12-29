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
	// Polyfill Buffer for production build. The hardware wallet needs Buffer.
	plugins: [
		sveltekit(),
		environment('all', { prefix: 'CANISTER_' }),
		environment('all', { prefix: 'DFX_' }),
		inject({
			include: ['node_modules/@ledgerhq/**'],
			modules: { Buffer: ['buffer', 'Buffer'] }
		})
	],
	resolve: {
		alias: [
			{
				find: '$declarations',
				replacement: fileURLToPath(new URL('./src/declarations', import.meta.url))
			},
			{
				find: '$lib',
				replacement: fileURLToPath(new URL('./src/lib', import.meta.url))
			}
		],
		extensions: ['.js', '.json', '.ts', '.svelte', '.did.d.ts']
	}
});
