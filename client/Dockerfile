# ── Build stage ──────────────────────────────────────────────────────────────
FROM node:20-alpine AS builder

WORKDIR /app

COPY package.json package-lock.json* ./
RUN npm ci --ignore-scripts

COPY svelte.config.js vite.config.ts tsconfig.json postcss.config.js tailwind.config.js* ./
COPY src ./src
COPY static* ./static/

RUN npm run build

# ── Runtime stage (serve with node adapter output) ────────────────────────────
FROM node:20-alpine

WORKDIR /app

COPY --from=builder /app/build ./build
COPY --from=builder /app/package.json ./

RUN addgroup -S client && adduser -S client -G client && chown -R client:client /app
USER client

EXPOSE 3001

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD wget -qO- http://localhost:3001/ || exit 1

CMD ["node", "build"]
