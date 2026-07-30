## Summary

<!-- Brief description of what this PR does and why. -->

## Changes

<!-- List specific changes. -->

- 

## Testing

<!-- How was this tested? What test coverage exists? -->

- [ ] `cargo test --all` passes
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes

## Security Checklist (for contract changes)

<!-- Complete this section if your PR adds or modifies contract functions. -->
<!-- See docs/SECURITY_REVIEW_CHECKLIST.md for the full checklist. -->

- [ ] `initialize()` uses `init_guard` to prevent re-initialization
- [ ] All state-mutating functions call `require_auth()` before any logic
- [ ] All arithmetic uses checked operations
- [ ] All inputs are validated
- [ ] Events are emitted for state-changing operations
- [ ] Unit tests cover happy path + error paths + edge cases
- [ ] Public functions have doc comments

## Migration / Rollback

<!-- Any breaking changes, migration steps, or rollback concerns? -->

## Closes

<!-- Link the issue this PR resolves. -->

#
