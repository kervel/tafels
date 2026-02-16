# 3dt Quality Standards

**Last Updated:** 2026-02-16
**Quality Level:** Relaxed (demo/prototype)

---

## Quality Gates

| Gate                | Threshold | Enforced |
|---------------------|-----------|----------|
| Min Quality Score   | 70/100    | Yes      |
| Min Test Coverage   | 70%       | Yes      |
| Clippy Warnings     | 0         | Yes      |
| Compiler Warnings   | 0         | Yes      |

---

## Code Quality

### Complexity

- Max cyclomatic complexity per function: **15**
- Max file length: **400 lines**
- Max function length: **60 lines**
- Max function parameters: **6**

### Formatting

- `cargo fmt` MUST pass with no changes required
- `cargo clippy` MUST pass with zero warnings

---

## Testing

### Requirements

- All public API functions MUST have at least one test
- Bevy systems SHOULD have integration tests using `App::new()`
- Visual/rendering correctness is validated manually (screenshot comparison optional)

### Test Organization

- Unit tests: `#[cfg(test)] mod tests` in the same file
- Integration tests: `tests/` directory
- Benchmarks (optional): `benches/` directory using `criterion`

---

## Performance Budgets

| Metric               | Budget   | Enforced |
|----------------------|----------|----------|
| Target FPS           | 60       | Manual   |
| Max frame time       | 16ms     | Manual   |
| Max startup time     | 5s       | Manual   |
| Max RAM usage        | 2 GB     | Manual   |

---

## Code Review

- Require code review: **Recommended** (not mandatory for solo dev)
- Min reviewers: **1** (if collaborating)

---

## CI/CD

- `cargo build` MUST succeed
- `cargo test` MUST pass
- `cargo clippy` MUST pass
- `cargo fmt --check` MUST pass
- Block merge on failure: **Yes**

---

## Custom Checks

### GPU Compatibility

- Test on at least one Vulkan-capable and one Metal-capable backend
- Ensure WebGPU/wgpu compatibility for potential web builds

### Asset Validation

- All committed assets MUST have open/permissive licenses
- No assets larger than 50MB without LFS

---

## Exemptions

*No exemptions currently granted.*

---

## Notes

- Quality level: Relaxed (demo/prototype focus)
- Created by `/specswarm:init`
- Enforced by `/specswarm:ship` before merge
- Adjust thresholds as the project matures
