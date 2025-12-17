# Deployment Guide

How to deploy the PWA.

*This is a placeholder document.*

## Build Process

1.  Build the WASM module: `pnpm build:wasm`
2.  Build the PWA: `pnpm build`

## Hosting

-   The output is in `apps/pwa/build`.
-   Host the static files on a secure server.
-   Ensure correct HTTP security headers are set.
