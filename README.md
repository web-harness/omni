# omni

Omni is a harness that stays out of your way, compatible with LangGraph Agent Protocol: [Storybook](https://web-harness.github.io/omni/).

[![Omni UI](./demo.png)](https://web-harness.github.io/omni/)

- [structure](#structure)
- [runtime](#runtime)
- [status](#status)

## structure

- [`omni-ui/`](./omni-ui) — Dioxus UI, compiled to WebAssembly and served as a static site. Contains the main app logic, components, and state management.
- [`omni-rt/`](./omni-rt/crates) — Rust runtime crates for protocol handling, dock management, file system, deep agents, etc.
- [`omni-wc/`](./omni-wc) — Web Components version of the harness UI, for embedding in other applications.

## runtime

The Omni runtime is meant to be a reference implementation of the environment needed for the Agent Protocol and harness to be useful in. It includes a mix of Rust and Typescript components. The Typescript components are exposed as Web Components and consumed back in Dioxus and Rust.

## status

- [x] Full replica of the original [OpenWork](https://github.com/web-harness/openwork) UI in the Dioxus framework.
- [x] Improved dock management with [DockView](https://github.com/mathuo/dockview).
- [x] Improved Markdown viewer with [MDX](https://www.npmjs.com/package/@mdxeditor/editor).
- [x] Improved PDF viewer with [PDF.js](https://www.npmjs.com/package/pdfjs-dist).
- [x] Improved code viewer with [Monaco Editor](https://www.npmjs.com/package/monaco-editor).
- [x] Improved media viewer with [Plyr](https://www.npmjs.com/package/plyr).
- [x] Improved dialog system with [Popper.js](https://www.npmjs.com/package/@popperjs/core).
- [x] Reusability with Storybook integration and Web Component export.
- [x] Automatic client generation for the Agent Protocol.
- [ ] ZenFS integration for file system access.
- [ ] Bashkit integration for sandbox shell access.
- [ ] Deep Agent integration as harness' main agent loop.
- [ ] Auto UI mocking via auto openapi mocking with [Mockoon](https://github.com/mockoon/mockoon).
- [ ] Comprehensive test suite with unit, integration, and E2E tests.

