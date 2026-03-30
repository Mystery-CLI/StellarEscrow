// Enable client-side navigation with preloading on hover/tap (SvelteKit default is 'hover').
// Setting preloadingStrategy here makes it explicit and easy to tune.
export const prerender = false;
export const ssr = false; // SPA mode — all routes are client-side only
