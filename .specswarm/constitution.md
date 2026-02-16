<!--
Sync Impact Report
- Version: 1.0.0 (initial)
- Added sections: All (initial creation)
- Removed sections: None
- Templates requiring updates: N/A (fresh project)
- Follow-up TODOs: None
-->

# 3dt Project Constitution

**Version:** 1.0.0
**Ratified:** 2026-02-16
**Last Amended:** 2026-02-16

## Mission Statement

3dt is a Rust demo application showcasing a 3D outdoor landscape with a
walking persona, built on the Bevy engine. The project demonstrates
realistic daylight rendering with grass, vegetation, and character
animation in a performant, idiomatic Rust codebase.

---

## Principles

### Principle 1: Idiomatic Rust

All code MUST follow Rust idioms and conventions. Prefer ownership and
borrowing over reference counting. Use `clippy` lints at the `warn`
level. Avoid `unsafe` blocks unless performance-critical and documented
with a safety comment. Leverage the type system to encode invariants at
compile time rather than runtime checks.

**Rationale:** Idiomatic Rust prevents entire classes of bugs at compile
time and produces maintainable, readable code.

### Principle 2: ECS-First Architecture

All game logic MUST be expressed through Bevy's Entity Component System.
Components hold data, systems hold logic, resources hold shared state.
Avoid storing behavior in components or coupling systems to specific
entity structures. Keep systems small and focused on a single
responsibility.

**Rationale:** ECS architecture enables parallelism, composability, and
makes the codebase easy to extend with new features.

### Principle 3: Performance-Aware Design

Rendering MUST target 60 FPS on mid-range hardware. Vegetation and
terrain systems MUST use LOD (Level of Detail) or instancing where
appropriate. Profile before optimizing - do not prematurely optimize, but
do not ignore obvious performance pitfalls like per-frame allocations or
unnecessary draw calls.

**Rationale:** A 3D landscape demo is only compelling when it runs
smoothly. Performance is a feature, not an afterthought.

### Principle 4: Modularity and Separation

The codebase MUST be organized into clear Bevy plugins: terrain,
vegetation, character, camera, lighting. Each plugin MUST be
self-contained with its own components, systems, and resources. Cross-
plugin communication MUST use events or shared resources, not direct
system coupling.

**Rationale:** Plugin-based architecture makes it easy to develop, test,
and iterate on individual features independently.

### Principle 5: Readable Over Clever

Code MUST prioritize readability. Prefer explicit code over macro magic.
Name systems, components, and resources descriptively. Add comments only
where the intent is not obvious from the code itself.

**Rationale:** A demo project serves as a learning resource. Clarity
enables others to understand and build upon the work.

---

## Governance

### Amendment Procedure

1. Propose changes via a discussion or PR description.
2. Changes to principles require justification and review.
3. Version is bumped according to semantic versioning:
   - **MAJOR:** Removing or fundamentally redefining a principle.
   - **MINOR:** Adding a new principle or expanding guidance.
   - **PATCH:** Wording clarifications and typo fixes.

### Compliance

All code contributions MUST align with these principles. The
`/specswarm:ship` workflow validates compliance before merge.
