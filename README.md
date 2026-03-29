# omni

Omni is a harness that stays out of your way, compatible with LangGraph Agent Protocol: [Storybook](https://web-harness.github.io/omni/).

[![Omni UI](./demo.png)](https://web-harness.github.io/omni/)

Current Status: Mocked replica of the original OpenWork in the Dioxus framework.

## structure

- [`omni/`](./omni) — Dioxus UI, compiled to WebAssembly and served as a static site. Contains the main app logic, components, and state management.
- [`omni-rt/`](./omni-rt) — Rust runtime crates for protocol handling, dock management, file system, deep agents, etc.
- [`omni-wc/`](./omni-wc) — Web Components version of the harness UI, for embedding in other applications.

## runtime

The Omni runtime is meant to be a reference implementation of the environment needed for the Agent Protocol and harness to be useful in. It includes a mix of Rust and Typescript components. The Typescript components are exposed as Web Components and consumed back in Dioxus and Rust.
