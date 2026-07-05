# syntax=docker/dockerfile:1.6

# ============================================================================
# Stage 1 — Build the SvelteKit admin app
# ============================================================================
FROM node:20-alpine AS builder

WORKDIR /build

COPY package.json package-lock.json ./
RUN npm ci --no-audit --no-fund --ignore-scripts

COPY . .

# Generate SvelteKit types (.svelte-kit/tsconfig.json) before building.
RUN npx svelte-kit sync

ENV NODE_ENV=production
ENV PUBLIC_API_URL=http://localhost:8080
ENV PUBLIC_WS_URL=ws://localhost:8080
RUN npm run build

# ============================================================================
# Stage 2 — Runtime (Node adapter)
# ============================================================================
FROM node:20-alpine AS runtime

LABEL org.opencontainers.image.title="egp-exchange-admin-frontend" \
      org.opencontainers.image.description="EGP Exchange — SvelteKit admin frontend"

WORKDIR /app

ENV NODE_ENV=production

COPY package.json package-lock.json ./
RUN npm ci --omit=dev --no-audit --no-fund --ignore-scripts

COPY --from=builder /build/build ./build

RUN addgroup -S app && adduser -S app -G app
USER app

EXPOSE 3001

CMD ["node", "build/index.js"]
