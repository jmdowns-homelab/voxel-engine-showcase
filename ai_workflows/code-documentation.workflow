# Module Documentation Workflow

This document outlines the workflow for documenting Rust modules in the voxel engine project.

## 1. Module Traversal (Bottom-Up)

When documenting a module, always follow a bottom-up approach:

1. Start at the specified module's directory
2. Perform a post-order depth-first traversal:
   - For each file in the module:
     1. If it's a directory/module, document it first (recursively)
     2. If it's a Rust file, document its contents
   - Then document the current module's `mod.rs` or `lib.rs`

## 2. File Documentation Process

For each file being documented:

### Module Documentation
- Add/update module-level documentation at the top of the file
- Include:
  - A high-level description of the module's purpose
  - Key concepts and architecture
  - Important implementation details
  - Examples of common usage

### Item Documentation
Document all public items (functions, structs, enums, traits, etc.) with:
- High-level description
- Parameter documentation (`# Arguments` section)
- Return value documentation (`# Returns` section)
- Error conditions (`# Errors` section)
- Panic conditions (`# Panics` section)
- Examples for public APIs (`# Examples` section)

### Documentation Standards
- Use Rust's `///` for documentation comments
- Follow Rust API documentation guidelines
- Include code examples for public APIs
- Keep documentation concise but complete
- Use markdown for formatting
- Include links to related items using `[`ident`]` syntax

## 3. Special Cases

### Macros
- Document the macro's expansion
- Include examples of generated code
- Document any special syntax or patterns

### Unsafe Code
- Clearly document safety requirements
- Explain why unsafe is necessary
- Document invariants that must be maintained

### Performance
- Note any significant performance characteristics
- Document time/space complexity
- Mention any caching or memoization

## 4. Example Workflow

For a module structure like:
```
my_module/
├── mod.rs
├── submodule_a/
│   ├── mod.rs
│   └── helper.rs
└── submodule_b/
    ├── mod.rs
    └── utils.rs
```

The documentation order would be:
1. `my_module/submodule_a/helper.rs`
2. `my_module/submodule_a/mod.rs`
3. `my_module/submodule_b/utils.rs`
4. `my_module/submodule_b/mod.rs`
5. `my_module/mod.rs`

## 5. Documentation Lints

Enable these lints in `lib.rs` or `main.rs`:
```rust
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::missing_doc_code_examples)]
#![warn(rustdoc::invalid_rust_codeblocks)]
```

## 6. Review Process

1. Run `cargo doc --no-deps --open` to verify documentation
2. Check for any warnings from documentation lints
3. Verify all public items are documented
4. Ensure examples compile and are up-to-date
5. Check cross-references between items

## 7. Common Patterns

### Struct Documentation
```rust
/// A brief description of what this struct represents.
///
/// A more detailed explanation that might span multiple lines and explain the
/// purpose, behavior, and important implementation details of the struct.
///
/// # Examples
/// ```
/// let example = MyStruct::new();
/// assert_eq!(example.method(), expected_value);
/// ```
pub struct MyStruct {
    // Fields...
}
```

### Function Documentation
```rust
/// Performs a specific operation.
///
/// # Arguments
/// * `param1` - Description of the first parameter
/// * `param2` - Description of the second parameter
///
/// # Returns
/// A `Result` containing the computed value or an error if the operation fails.
///
/// # Errors
/// Returns `MyError` if:
/// - The input is invalid
/// - A system resource is unavailable
///
/// # Panics
/// Panics if the input is out of bounds.
///
/// # Examples
/// ```
/// let result = my_function(42, "test")?;
/// assert_eq!(result, expected_value);
/// ```
pub fn my_function(param1: i32, param2: &str) -> Result<SomeType, MyError> {
    // Implementation...
}
```

## 8. Style Guide

- Use full sentences with proper punctuation
- Prefer active voice ("Returns" not "Is returned")
- Use backticks for code elements
- Start descriptions with a capital letter and end with a period
- Be consistent with terminology
- Document `unsafe` blocks thoroughly
- Include examples that demonstrate common use cases