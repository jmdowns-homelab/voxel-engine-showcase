# Domain Flow Documentation Workflow
# 
# This workflow helps document the flow of operations between different domains in the voxel engine.
# It creates hierarchical documentation that shows both high-level system interactions and detailed
# implementation flows without being tied to specific code implementations.

# =============================================
# 1. Identify the Flow to Document
# =============================================
# 
# IMPORTANT: Before proceeding, verify that you have access to the relevant source code.
# If you cannot access or verify the code implementation, DO NOT create documentation.
#
# Start by identifying the primary domain or feature area to document.
# This should be a high-level concept like "Rendering", "Chunk Loading", or "Input Processing".
#
# Determine the flow type:
# - User Interaction Flow: Triggered directly by user actions
# - System Flow: Automatic operations triggered by system state or timers
#
# Questions to answer:
# - What is the main purpose of this flow?
# - What are the entry and exit points?
# - What are the key systems/components involved?
# - Is this triggered by user action or system state?
# - Can I verify all aspects of this flow in the codebase?

# =============================================
# 2. Code Analysis and Verification
# =============================================
# 
# STRICT RULE: If you cannot find and verify the code that implements a flow,
# DO NOT document that flow. It's better to have no documentation than incorrect documentation.
#
# Before documenting, you MUST analyze the actual code to understand the implementation.
# Follow these steps:
#
# 1. Search for Relevant Code:
#    - Use `grep -r "keyword" src/` to find relevant files
    - Look for module entry points and public APIs
    - Search for test files that demonstrate usage
    - If no relevant code is found, STOP and do not document the flow

# 2. Verify Implementation:
#    - Can you trace the complete flow through the code?
    - Are all components and their interactions visible?
    - Can you verify the behavior from the code?
    - If any part is unclear or unverifiable, DO NOT document assumptions

# 3. Document Only What You Can Verify:
#    - Only include details that are explicitly in the code
    - If a feature is mentioned in comments but not implemented, do not document it
    - Use "TODO: Verify" comments for areas that need further investigation

# 4. Code Search Commands:
#    find src -name "*.rs" | xargs grep -l "keyword"
#    grep -r "fn function_name" src/
#    grep -r "struct StructName" src/

# =============================================
# 3. Document High-Level Flow (If Verifiable)
# =============================================
# 
# ONLY proceed if you have verified the complete flow in the code.
# If any part is unclear or unverifiable, document only what you can confirm.
#
# Create a high-level flow document in markdown format and save it in the appropriate directory:
# - `docs/domain_flows/user_interaction/` for user-triggered flows
# - `docs/domain_flows/system/` for system-triggered flows
#
# File naming: Use snake_case and be descriptive (e.g., `chunk_loading.md`)
#
# Document structure (only include sections you can verify):
# ```markdown
# # [Flow Name]
# 
# ## Type
# [User Interaction | System]
# 
# ## Purpose
# [Brief description of what this flow accomplishes]
# 
# ## Trigger
# [What event or action initiates this flow]
#   - For user interactions: Describe the user action (e.g., "Right-click on a block")
#   - For system flows: Describe the condition or event (e.g., "Chunk enters player's load distance")
# 
# ## Systems Involved
# - [System 1]: Role in this flow
# - [System 2]: Role in this flow
# 
# ## Flow
# 1. [First high-level step]
#    - Success: [Next steps]
#    - Error: [Error handling]
# 2. [Next high-level step]
#    - ...
# 
# ## Related Flows
# - [Related Flow 1]: [Brief description of relationship]
# - [Related Flow 2]: [Brief description of relationship]
# 
# ## Performance Considerations
# [Any important performance implications or bottlenecks]
# 
# ## See Also
# [Links to related documentation or code]
# ```

# =============================================
# 4. Document Detailed Sub-Flows (If Verifiable)
# =============================================
# 
# WARNING: Only create sub-flows for components you can fully verify in the code.
# If the implementation details are not visible or verifiable, do not create the sub-flow.
#
# For each complex step in the high-level flow, create a detailed sub-flow document ONLY if:
# 1. You can trace the complete code path
# 2. You can verify all implementation details
# 3. You can confirm the behavior matches the documentation
#
# Store sub-flows in the same directory as their parent flow.
# Naming: Append _subflow to the parent name (e.g., `chunk_loading_mesh_generation_subflow.md`)
# ```markdown
# # [Sub-Flow Name]
# 
# ## Type
# [User Interaction | System] (Should match parent flow type)
# 
# ## Parent Flow
# [Link back to parent flow]
# 
# ## Purpose
# [Detailed description of this specific sub-flow]
# [Focus on implementation details while maintaining domain-level abstraction]
# 
# ## Implementation Details
# - Key data structures used
# - Important algorithms
# - Error handling approaches
# 
# ## Dependencies
# - [Dependency 1]: [How it's used]
# - [Dependency 2]: [How it's used]
# 
# ## Performance Considerations
# [Specific performance implications]
# 
# ## See Also
# [Links to related sub-flows or implementation details]
# ```

# =============================================
# 5. Strict Code Verification
# =============================================
# 
# MANDATORY: Before finalizing any documentation, verify EVERY aspect against the code:
#
# 1. Source Code Verification:
#    - Can you see the actual implementation of EVERY step?
    - Are there any "black box" components you can't verify?
    - If anything is unclear, remove or mark it as unverified

# 2. Behavior Verification:
#    - Does the code actually behave as documented?
    - Are all error cases handled as described?
    - Are there any side effects not mentioned?

# 3. Completeness Check:
#    - Is the documentation 100% accurate based on the code?
    - Are there any TODOs or placeholders that need resolution?
    - Have you verified edge cases and error conditions?

# 4. If ANY part cannot be verified:
#    - Remove unverified claims
    - Add a "Limitations" section explaining what couldn't be verified
    - Consider marking the entire document as "Partial" if significant parts are unverified

# =============================================
# 6. Update Documentation Index (If Complete)
# =============================================
# 
# ONLY update the index if the documentation is complete and verified.
# If the documentation is partial or unverified, do NOT add it to the index.
#
# For verified documentation, update the appropriate index file in `docs/domain_flows/`:
# - `user_interaction/README.md` for user interaction flows
# - `system/README.md` for system flows
#
# Add a [PARTIAL] or [UNVERIFIED] tag if the documentation is incomplete:
# ```markdown
# ## [Category]
# - [Flow Name](./flow_name.md) - [Purpose] (Type: [User/System]) [PARTIAL]
# ```

# =============================================
# 7. Final Verification and Review
# =============================================
# 
# MANDATORY: Before considering the documentation complete:
#
# 1. Code Cross-Check:
#    - Re-read the code while reviewing the documentation
    - Ensure every claim is backed by actual code
    - Remove any assumptions or guesses

# 2. Peer Review:
#    - Have another developer verify the documentation against the code
    - Focus on accuracy, not just style or grammar
    - Address all feedback before finalizing

# 3. Quality Gates:
#    - Is the documentation 100% accurate based on the code?
    - Are there any unverified claims or assumptions?
    - Would you stake your reputation on its accuracy?

# 4. If in Doubt:
#    - Remove unverified information
    - Add a note about what couldn't be verified
    - Consider delaying documentation until the code is available

# =============================================
# Example: Chunk Loading Flow Analysis
# =============================================

## 1. Code Investigation
```bash
# Find chunk-related modules
find src -name "*chunk*" -type f

# Search for chunk generation code
grep -r "generate" src/voxels/

# Look for chunk update triggers
grep -r "update.*chunk" src/

# Find rendering code for chunks
grep -r "render.*chunk" src/rendering/
```

## 2. Key Findings
- Chunk generation starts in `voxels/chunk/mod.rs`
- Mesh generation happens in `rendering/mesh/chunk_mesh.rs`
- Chunk updates are managed by `world/chunk_manager.rs`
- Rendering is handled by `rendering/renderer.rs`

## 3. Flow Documentation
Based on code analysis, document the actual flow with references to source files.

# =============================================
# Example Flows
# =============================================

## User Interaction Flow Example: Camera Input Processing

1. High-Level Flow: `docs/domain_flows/user_interaction/camera_input.md`
   - Type: User Interaction
   - Trigger: User input (mouse movement, keyboard)
   - Systems: InputManager → CameraController → Camera
   - Purpose: Process user input to update camera position/orientation

2. Detailed Sub-Flow: `docs/domain_flows/user_interaction/camera_rotation_subflow.md`
   - Type: User Interaction
   - Parent: camera_input.md
   - Details: Converts mouse movement to camera rotation
   - Dependencies: InputHandler, Camera struct, delta time

## System Flow Example: Chunk Loading

1. High-Level Flow: `docs/domain_flows/system/chunk_loading.md`
   - Type: System
   - Trigger: Player moves into new chunk area
   - Systems: World → ChunkManager → ChunkGenerator → MeshGenerator
   - Purpose: Load and prepare chunks around player position

2. Detailed Sub-Flow: `docs/domain_flows/system/chunk_mesh_generation_subflow.md`
   - Type: System
   - Parent: chunk_loading.md
   - Details: Generates mesh data for a loaded chunk
   - Dependencies: Chunk data, BlockRegistry, MeshBuilder
