# Update Documentation Workflow

This document outlines the workflow for systematically updating documentation for files identified in the `needs-documentation.txt` file, following the standards defined in `code-documentation.workflow`.

## 1. Preparation

1. Ensure you have the latest version of `needs-documentation.txt` from the documentation investigation process.
2. Review the `code-documentation.workflow` file to understand documentation standards.
3. Create a documentation branch:
   ```bash
   git checkout -b documentation-update
   ```

## 2. Prioritization

1. Categorize documentation needs by importance:
   - Critical: Crate-level and top-level module documentation
   - High: Public API documentation for frequently used components
   - Medium: Internal module documentation
   - Low: Helper functions and implementation details

2. Sort the files in `needs-documentation.txt` by priority.

## 3. Documentation Process

For each file in `needs-documentation.txt`:

### 3.1. File Analysis

1. Open the file and assess what documentation is missing:
   - Module-level documentation
   - Function/method documentation
   - Struct/enum/trait documentation
   - Examples
   - Parameter descriptions
   - Return value descriptions
   - Error conditions
   - Panic conditions

2. Review the surrounding code to understand:
   - The purpose of the module/item
   - How it fits into the larger system
   - Key behaviors and edge cases
   - Performance characteristics

### 3.2. Documentation Implementation

Follow the bottom-up approach from `code-documentation.workflow`:

1. If the file is part of a module hierarchy, start with the deepest files first.
2. For each file:
   - Add/update module-level documentation at the top of the file
   - Document all public items (functions, structs, enums, traits, etc.)
   - Follow the documentation standards in `code-documentation.workflow`

### 3.3. Documentation Format

Use these templates as a guide:

#### Module Documentation
```rust
//! # Module Name
//!
//! A high-level description of the module's purpose.
//!
//! ## Key Concepts
//!
//! * Concept 1 - Brief explanation
//! * Concept 2 - Brief explanation
//!
//! ## Architecture
//!
//! Explanation of how this module fits into the larger system.
//!
//! ## Implementation Details
//!
//! Important details about the implementation that users should know.
//!
//! ## Examples
//!
//! ```
//! // Example code demonstrating common usage
//! ```
```

#### Struct Documentation
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

#### Function Documentation
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

## 4. Verification

After documenting each file:

1. Run `cargo doc --no-deps` to ensure the documentation compiles without errors.
2. Check for any warnings from documentation lints.
3. Review the generated documentation by running `cargo doc --no-deps --open`.
4. Verify that:
   - All public items are documented
   - Examples compile and are up-to-date
   - Cross-references between items work correctly
   - Documentation is clear and follows the project style

## 5. Progress Tracking

1. Create a copy of `needs-documentation.txt` called `documentation-progress.txt`.
2. As you complete documentation for each file, mark it in `documentation-progress.txt`:
   ```
   [DONE] src/lib.rs (added crate-level documentation)
   [IN PROGRESS] src/engine_state/mod.rs (working on module-level documentation)
   src/engine_state/rendering/pipeline_manager.rs (not started)
   ```

## 6. Specific File Types

### 6.1. Crate-Level Documentation (lib.rs/main.rs)

For `src/lib.rs` and `src/main.rs`:
- Add comprehensive crate-level documentation
- Include:
  - Overview of the crate's purpose
  - Key modules and their relationships
  - Getting started examples
  - Common usage patterns
  - Enable documentation lints:
    ```rust
    #![warn(missing_docs)]
    #![warn(rustdoc::missing_crate_level_docs)]
    #![warn(rustdoc::missing_doc_code_examples)]
    #![warn(rustdoc::invalid_rust_codeblocks)]
    ```

### 6.2. Module Documentation (mod.rs)

For module files:
- Document the module's purpose and organization
- Explain relationships between submodules
- Provide examples of common module usage

### 6.3. Implementation Files

For implementation files:
- Focus on documenting public API
- Include implementation details that are important for users
- Document performance characteristics and constraints

## 7. Review Process

1. After completing all documentation updates:
   - Run `cargo doc --no-deps --open` to review the full documentation
   - Check for consistency in style and terminology
   - Ensure all examples are correct and up-to-date

2. Have another team member review the documentation changes for:
   - Technical accuracy
   - Clarity and completeness
   - Adherence to project documentation standards

## 8. Commit and Merge

1. Commit the documentation changes with descriptive messages:
   ```bash
   git add .
   git commit -m "docs: Add documentation for [module/component]"
   ```

2. Create a pull request for the documentation branch.

3. After review and approval, merge the documentation updates.

## 9. Follow-Up

1. Update `needs-documentation.txt` to remove the documented files.
2. Schedule regular documentation reviews to maintain quality.
3. Consider implementing automated documentation checks in CI.

## 10. Special Cases

### 10.1. Verification Files

For files marked "needs verification" in `needs-documentation.txt`:
1. Review the existing documentation
2. Verify it meets the standards in `code-documentation.workflow`
3. Update or expand documentation as needed
4. Mark the file as verified in `documentation-progress.txt`

### 10.2. Specific Documentation Issues

For files with specific documentation issues noted (e.g., "UiVertex struct needs better documentation"):
1. Focus on addressing the specific issue mentioned
2. Ensure the rest of the file's documentation also meets standards
3. Note in `documentation-progress.txt` that the specific issue was addressed

## 11. Example Workflow

For the file list in `needs-documentation.txt`:

1. Start with `src/lib.rs` (crate-level documentation)
2. Move to `src/main.rs` (application documentation)
3. Document `src/engine_state/mod.rs` (module-level documentation)
4. Continue with specific implementation files in bottom-up order
5. Verify files marked for verification
6. Address specific documentation issues

## 12. Documentation Quality Checklist

Before marking a file as complete, ensure:
- [ ] All public items have documentation
- [ ] Documentation explains purpose and behavior
- [ ] Examples are provided for public APIs
- [ ] Parameter and return value documentation is complete
- [ ] Error conditions are documented
- [ ] Panic conditions are documented
- [ ] Documentation compiles without warnings
- [ ] Documentation follows project style guidelines
- [ ] Cross-references are correct and working
