# Domain Flows

This directory contains documentation for the various domain flows in the voxel engine. Flows are organized by their trigger type:

## Flow Types

### [User Interaction Flows](./user_interaction/README.md)
Documentation for flows triggered by direct user input, such as:
- Camera controls
- Block placement/breaking
- UI interactions
- Inventory management

### [System Flows](./system/README.md)
Documentation for system-triggered flows, including:
- Chunk loading and management
- World generation
- Physics simulation
- AI behavior

## Documentation Guidelines

### Creating New Flows
1. Determine if the flow is user-initiated or system-initiated
2. Create a new markdown file in the appropriate directory
3. Follow the template in `domain-flow.workflow`
4. Update the relevant README files
5. Cross-reference related flows

### Naming Conventions
- Use descriptive, hyphenated names (e.g., `chunk-loading.md`)
- Group related flows in subdirectories when necessary
- Use consistent terminology across all flows

### Maintenance
- Keep flows up to date with code changes
- Update related flows when making changes
- Verify accuracy against implementation regularly

## Related Documentation

- [API Reference](../api/)
- [Architecture Overview](../architecture.md)
- [Performance Guidelines](../performance.md)
