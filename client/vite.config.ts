import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import { VitePWA } from 'vite-plugin-pwa';

export default defineConfig({
  plugins: [
    sveltekit(),
    VitePWA({
      registerType: 'autoUpdate',
      devOptions: { enabled: true },
      manifest: {
        name: 'StellarEscrow',
        short_name: 'StellarEscrow',
        description: 'Decentralized escrow on Stellar/Soroban',
        theme_color: '#6366f1',
        background_color: '#f9fafb',
        display: 'standalone',
        start_url: '/',
        icons: [
          { src: '/icon.svg', sizes: 'any', type: 'image/svg+xml', purpose: 'any maskable' }
        ]
      },
      workbox: {
        globPatterns: ['**/*.{js,css,html,svg,png,ico,woff2}'],
        runtimeCaching: [
          {
            urlPattern: ({ url }) => url.pathname.startsWith('/events') || url.pathname.startsWith('/search'),
            handler: 'NetworkFirst',
            options: {
              cacheName: 'api-cache',
              networkTimeoutSeconds: 5,
              expiration: { maxEntries: 50, maxAgeSeconds: 300 }
            }
          }
        ]
      }
    })
  ],

  build: {
    // Raise chunk warning threshold to 600kb (stellar-sdk is large)
    chunkSizeWarningLimit: 600,
    rollupOptions: {
      output: {
        // Manual code splitting: vendor libs into separate chunks
        manualChunks(id) {
          if (id.includes('node_modules/@stellar')) return 'stellar-sdk';
          if (id.includes('node_modules')) return 'vendor';
        }
      }
    }
  },

  // Aggressive dependency pre-bundling for faster dev cold starts
  optimizeDeps: {
    include: ['@stellar/stellar-sdk']
  }
});
