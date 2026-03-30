import adapter from '@sveltejs/adapter-auto';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	kit: {
		adapter: adapter(),
		// CONTENT SECURITY POLICY (CSP)
		// Helps prevent XSS and data injection attacks by restricting resource origins.
		csp: {
			directives: {
				'default-src': ["'self'"],
				'script-src': ["'self'", "'unsafe-inline'"], // Minimal allowance for essential scripts
				'style-src': ["'self'", "'unsafe-inline'"],
				'img-src': ["'self'", 'data:', 'https:'],
				'connect-src': [
					"'self'",
					'https://horizon.stellar.org',
					'https://horizon-testnet.stellar.org'
				]
			}
		}
	}
};

export default config;
