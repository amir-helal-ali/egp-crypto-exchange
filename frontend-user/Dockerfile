# syntax=docker/dockerfile:1.6

# ============================================================================
# Stage 1 — Build the SvelteKit app
# ============================================================================
FROM node:20-alpine AS builder

WORKDIR /build

# Install dependencies (cached layer).
COPY package.json package-lock.json* ./
RUN npm install --no-audit --no-fund --ignore-scripts

# Copy source.
COPY . .

# Generate SvelteKit types (.svelte-kit/tsconfig.json) before building.
RUN npx svelte-kit sync

# Build with environment overrides baked in.
ENV NODE_ENV=production
ENV PUBLIC_API_URL=http://localhost:8080
ENV PUBLIC_WS_URL=ws://localhost:8080
RUN npm run build

# ============================================================================
# Stage 2 — Runtime (Node adapter)
# ============================================================================
FROM node:20-alpine AS runtime

LABEL org.opencontainers.image.title="egp-exchange-user-frontend" \
      org.opencontainers.image.description="EGP Exchange — SvelteKit user frontend"

WORKDIR /app

ENV NODE_ENV=production

# Copy package files and install only production deps.
COPY package.json package-lock.json* ./
RUN npm install --omit=dev --no-audit --no-fund --ignore-scripts

# Copy the built output.
COPY --from=builder /build/build ./build

# Create non-root user.
RUN addgroup -S app && adduser -S app -G app
USER app

EXPOSE 3000

# SvelteKit adapter-node entry point.
CMD ["node", "build/index.js"]
