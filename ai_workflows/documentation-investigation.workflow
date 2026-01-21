# Documentation Investigation Workflow

This document outlines the process for systematically investigating and identifying modules and files that need documentation in the voxel engine project.

## 0. Preparation

1. Create a new file called `needs-documentation.txt` in the project root:
   ```bash
   touch needs-documentation.txt
   ```

2. Ensure you have access to the `code-documentation.workflow` file for reference on what constitutes sufficient documentation.

## 1. Directory Traversal (Top-Down)

Start at the `src` directory and perform a breadth-first traversal:

1. Begin with the top-level modules directly under `src/`
2. For each module, scan its files before moving to its submodules
3. For each submodule, repeat the process recursively

## 2. File Analysis Process

For each file encountered during traversal:

### Module Files (`mod.rs`)
- Check for module-level documentation at the top of the file
- Documentation should include:
  - A high-level description of the module's purpose
  - Key concepts and architecture
  - Important implementation details
  - Examples of common usage (if applicable)

### Regular Rust Files (`.rs`)
For each public item (functions, structs, enums, traits, etc.):
- Check for documentation comments (`///`)
- Verify the documentation includes:
  - High-level description
  - Parameter documentation (`# Arguments` section) for functions/methods
  - Return value documentation (`# Returns` section) for functions/methods
  - Error conditions (`# Errors` section) if applicable
  - Panic conditions (`# Panics` section) if applicable
  - Examples for public APIs (`# Examples` section) if applicable

## 3. Documentation Deficiency Reporting

When insufficient documentation is found:
1. Determine the nesting level of the file relative to `src/`
2. Add an entry to `needs-documentation.txt` with:
   - Indentation: one space per nesting level
   - The relative path from the project root
   - A brief note about what documentation is missing

Example format:
```
 src/engine_state/rendering.rs (missing module documentation)
  src/engine_state/rendering/pipeline_manager.rs (missing function documentation for render())
   src/engine_state/rendering/ui/primitives.rs (missing struct documentation for UiVertex)
```

## 4. Verification Process

After the initial scan is complete:
1. Review each entry in `needs-documentation.txt`
2. Open the corresponding file and verify that documentation is indeed missing
3. Remove any false positives from the list
4. Prioritize documentation needs based on:
   - Public API importance
   - Complexity of the code
   - Frequency of use

## 5. Implementation Guidelines

When implementing the investigation:

### Command-Line Implementation
```bash
#!/bin/bash

# Clear or create the needs-documentation file
echo "" > needs-documentation.txt

# Function to check a file for documentation
check_file() {
  local file=$1
  local indent=$2
  local missing=false
  
  # Check for missing documentation using grep/regex
  # If documentation is missing, set missing=true
  
  if $missing; then
    echo "${indent}${file} (missing documentation)" >> needs-documentation.txt
  fi
}

# Function to traverse directories
traverse_dir() {
  local dir=$1
  local indent=$2
  
  # Process files in current directory first
  for file in "$dir"/*.rs; do
    if [ -f "$file" ]; then
      check_file "$file" "$indent"
    fi
  done
  
  # Then process subdirectories
  for subdir in "$dir"/*/; do
    if [ -d "$subdir" ]; then
      traverse_dir "$subdir" "$indent "
    fi
  done
}

# Start traversal from src directory
traverse_dir "src" ""

echo "Documentation investigation complete. Results in needs-documentation.txt"
```

### Programmatic Implementation
For a more sophisticated approach, consider using a Rust program that:
1. Parses the AST of each Rust file
2. Identifies public items without documentation
3. Generates a detailed report

## 6. Follow-Up Actions

After identifying files needing documentation:
1. Refer to `code-documentation.workflow` for documentation standards
2. Create a prioritized plan for adding missing documentation
3. Consider implementing documentation lints in the project:
   ```rust
   #![warn(missing_docs)]
   #![warn(rustdoc::missing_crate_level_docs)]
   ```
4. Set up CI checks to prevent new undocumented code from being merged

## 7. Regular Maintenance

Schedule regular documentation audits:
1. Run this workflow periodically (e.g., monthly)
2. Focus especially on new or heavily modified modules
3. Update documentation standards as the project evolves
4. Consider automated tools to maintain documentation quality
