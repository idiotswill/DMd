# ADR 001 — Initial Technology Stack

**Status:** Accepted for Gate 0

## Decision

Use a modular Rust core, a Tauri desktop shell, a Svelte + TypeScript frontend, SQLite persistence, and pluggable local inference adapters.

## Core

- Rust 2024 edition
- Tokio for async/concurrency
- Serde for typed serialization
- thiserror for domain errors
- tracing for structured diagnostics

## Persistence

- SQLite as the campaign-local transactional database
- SQLx as the Rust access layer
- normalized current-state tables + append-only event journal + periodic snapshots

## Desktop UI

- Tauri 2 shell
- Svelte + TypeScript frontend
- Vite build pipeline

## Local inference

All inference systems are adapters behind stable interfaces:

- Speech-to-text: native/local provider, with whisper.cpp as an early candidate
- Language model: pluggable local/server provider; llama.cpp-compatible backends are an early candidate
- Text-to-speech: pluggable native/ONNX-capable provider

No inference provider may directly mutate authoritative game state.

## Python

Python is allowed for development tooling, benchmark/evaluation scripts, content conversion, and ML experimentation. It is not a required runtime dependency for players.

## Deployment model

DMd begins as a modular monolith. AI inference may run in isolated worker processes for crash containment and CPU/GPU scheduling, but the project will not begin as a distributed microservice system.

## Rationale

The architecture prioritizes state safety, predictable latency, native packaging, concurrency, testability, and long-term maintainability while retaining modern UI ergonomics.
