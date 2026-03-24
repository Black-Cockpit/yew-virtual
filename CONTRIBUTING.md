# Contributing to yew-virtual

Thank you for your interest in contributing. This document outlines the coding standards and conventions that **must** be followed for all contributions.

## Source code rules

All source code in this project follows a strict set of rules. These rules are **non-negotiable** and will be enforced during code review.

### RULE 1: One type per file

Never create two types (`struct`, `enum`, `trait`) in the same file. Each type gets its own file named in `snake_case`.

**Exception:** A type with a generic variant can share a file (for example, a type alias alongside its source type).

### RULE 2: Directory structure

- Use `snake_case` for all directory and file names.
- Organize by domain or feature.
- Create a `mod.rs` in each directory.

### RULE 3: File naming

| Type | File name |
|------|-----------|
| Struct `VirtualItem` | `virtual_item.rs` |
| Enum `ScrollDirection` | `scroll_direction.rs` |
| Struct `Virtualizer` | `virtualizer.rs` |

### RULE 4: Visibility

Use the minimum visibility needed:

- Default: private (no keyword)
- `pub(crate)`: when used in other modules but not in the public API
- `pub`: only when part of the public API

### RULE 5: Module documentation

Every `mod` declaration in `mod.rs` must have its own `///` doc comment directly above it:

```rust
/// Brief description of this module.
///
/// Expanded description of what this module does.
pub mod my_module;
```

### RULE 6: Type documentation

Every `struct`, `enum`, and `trait` must have a `///` doc comment. Every field must have its own `///` doc comment. Separate fields with blank lines for readability.

### RULE 7: Method documentation

Every public method must have a clear `///` doc comment covering:

- Brief description
- `# Parameters` section (when parameters exist)
- `# Returns` section (when a value is returned)
- `# Errors` section (when errors can occur)

### RULE 8: Inline comments

Add a comment before **every** logical step inside method bodies using imperative mood:

```rust
pub fn process(&mut self) {
    // Validate the input parameters.
    if !self.is_valid() {
        return;
    }

    // Apply the transformation.
    self.transform();
}
```

### RULE 9: Import organization

Organize imports in this exact order with blank lines between groups:

```rust
// 1. Standard library
use std::collections::HashMap;

// 2. External crates (alphabetically)
use wasm_bindgen::prelude::*;

// 3. Internal crate
use crate::core::virtualizer::Virtualizer;
```

**Never** put `use` statements inside functions. Always at the top of the file.

**Always** import with `use crate::` instead of `use super::` or `use prelude::` (exception: `prelude.rs` files use `super::`).

### RULE 9.1: Prohibited comment styles

Never use separator comments with equal signs or dashes:

```rust
// ============================================  ← PROHIBITED
// --------------------------------------------  ← PROHIBITED
// --- Section Name ---                         ← PROHIBITED
```

### RULE 10: No types in mod.rs

`mod.rs` files must only contain module declarations and documentation comments. All types must be in their own dedicated files.

### RULE 11: Prelude module for re-exports

All `pub use` statements must be placed in a `prelude` module, **not** in `mod.rs` files.

Every `prelude.rs` file must include a `//!` module-level doc comment explaining what is re-exported.

### RULE 12: No full path type references

Never call types using their full path inline. Always import types at the top of the file.

### RULE 14: No tests in source files

Never add `#[cfg(test)]` modules inside source files in `src/`. All tests must be placed in the `tests/` directory.

### Test documentation

Integration tests follow [TEST_RULES.md](TEST_RULES.md): module docs, per-test docs, naming, and import order.

### No unwrap() or expect()

Never use `unwrap()` or `expect()` anywhere in the codebase, including test code. Use safe alternatives like `unwrap_or`, `match`, or the `?` operator.

## Pull request process

1. **Fork** the repository and create a feature branch from `main` or `master`.
2. **Follow all rules** listed above without exception.
3. **Add tests** for any new functionality in the `tests/` directory.
4. **Run the full test suite** before submitting:

   ```bash
   cargo test -p yew-virtual
   ```

5. **Run formatting and linting**:

   ```bash
   cargo fmt --all
   cargo clippy --workspace --all-targets -- -D warnings
   ```

6. **Open a pull request** with a clear description of the changes.

## Architecture

The workspace contains:

- **`yew-virtual`** — Headless core (`core/`) plus Yew hooks (`hooks/`) for container and window scrolling.
- **`examples/tailwind-virtual-list`** — Trunk-built demo; not published to crates.io.

Shared dependency versions live in the root **`[workspace.dependencies]`**; member crates reference them with `{ workspace = true }`.

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
