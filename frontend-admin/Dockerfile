# syntax=docker/dockerfile:1.6

# ============================================================================
# Stage 1 — Build the SvelteKit admin app
# ============================================================================
FROM node:20-alpine AS builder

WORKDIR /build

COPY package.json package-lock.json* ./
RUN npm install --no-audit --no-fund

COPY . .
ENV NODE_ENV=production
RUN npm run build

# ============================================================================
# Stage 2 — Runtime (Node adapter)
# ============================================================================
FROM node:20-alpine AS runtime

LABEL org.opencontainers.image.title="egp-exchange-admin-frontend" \
      org.opencontainers.image.description="EGP Exchange — SvelteKit admin frontend"

WORKDIR /app

ENV NODE_ENV=production

COPY package.json package-lock.json* ./
RUN npm install --omit=dev --no-audit --no-fund

COPY --from=builder /build/build ./build
COPY --from=builder /build/.svelte-kit/output ./.svelte-kit/output

RUN addgroup -S app && adduser -S app -G app
USER app

EXPOSE 3001

CMD ["node", "build/index.js"]
