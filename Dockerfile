FROM node:22-alpine AS web-builder
ARG BUILD_SHA=dev
ARG GIT_SHA=dev
ARG SOURCE_COMMIT=dev
WORKDIR /source
COPY package.json package-lock.json ./
RUN npm ci
COPY index.html svelte.config.js tsconfig.json tsconfig.app.json tsconfig.node.json vite.config.ts ./
COPY public ./public
COPY src ./src
RUN BUILD_SHA="${BUILD_SHA:-${GIT_SHA:-${SOURCE_COMMIT:-dev}}}" npm run build:web

FROM rust:1.98-bookworm AS api-builder
ARG BUILD_SHA=dev
ARG GIT_SHA=dev
ARG SOURCE_COMMIT=dev
WORKDIR /source
COPY api/Cargo.toml api/Cargo.lock ./api/
COPY api/src ./api/src
RUN BUILD_SHA="${BUILD_SHA:-${GIT_SHA:-${SOURCE_COMMIT:-dev}}}" \
    cargo build --manifest-path api/Cargo.toml --release --locked

FROM gcr.io/distroless/cc-debian12:nonroot
WORKDIR /app
COPY --from=api-builder /source/api/target/release/intake-desk-api /app/intake-desk-api
COPY --from=web-builder /source/dist /app/dist
ENV PORT=8080
ENV STATIC_DIR=/app/dist
EXPOSE 8080
USER nonroot:nonroot
ENTRYPOINT ["/app/intake-desk-api"]
