# posthog-wasm

A very experimental WebAssembly library for PostHog.

## Why WASM?

The goal is to reduce the cost of implementing new Client SDK features.

We considered compiling Rust to portable libraries, but each library has to be.

## Other ideas

- Sidecar service (called through http):
- Compile to platform specific dlls: deployment is painful.