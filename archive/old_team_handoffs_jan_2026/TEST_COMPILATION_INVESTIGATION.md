# Test Compilation Fix - Detailed Investigation

## Issue
The `discovery_handler_tests.rs` file has `use tower::util::ServiceExt` on line 13, but the compiler still reports it's not in scope when `.oneshot()` is called on line 362.

## Rust Compiler Message
```
error[E0599]: no method named `oneshot` found for struct `Router` in the current scope
help: trait `ServiceExt` which provides `oneshot` is implemented but not in scope
help: trait `ServiceExt` which provides `oneshot` is implemented but not in scope; perhaps you want to import it
  5 + use tower::util::ServiceExt;
```

## Hypothesis
This could be a Rust edition or import shadowing issue. The import is on line 13, but the error points to line 5 for the suggestion.

## Possible Causes
1. **Import Order**: The import might be getting shadowed by another import
2. **Trait Not in Scope**: Despite the import, the trait might not be visible where it's used
3. **Version Mismatch**: Tower version mismatch between dependencies
4. **Module Issue**: The file might have internal modules that don't see the import

## Solution Attempt
Instead of fighting with imports, we should check if there's a pattern in other test files that works, or simplify the test structure.

The audit identified this as a critical blocker. Given complexity, I'll document this and create a workaround or deeper investigation is needed by someone with access to run cargo directly.

