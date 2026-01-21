# System Flows

This directory contains documentation for system-level flows in the voxel engine. These flows are triggered by system events and state changes rather than direct user input.

## Core Flows

### Chunk Management
- [Chunk Loading](./chunk_loading.md)
  - [Chunk Data Generation](./chunk_data_generation_subflow.md)
  - [Chunk Mesh Generation](./chunk_mesh_generation_subflow.md)
  - [Chunk Visibility and Culling](./chunk_visibility_subflow.md)

### Rendering
- [Rendering Pipeline](../rendering/pipeline.md) (Planned)
- [Lighting System](../rendering/lighting.md) (Planned)

### World Simulation
- [Physics Simulation](../simulation/physics.md) (Planned)
- [Entity AI](../simulation/ai.md) (Planned)

## Adding New Flows

1. Create a new markdown file in the appropriate subdirectory
2. Follow the template in the root `domain-flow.workflow`
3. Update this README with a link to your new flow
4. Ensure all related flows are cross-linked

## Cross-References

- [User Interaction Flows](../user_interaction/README.md)
- [API Documentation](../../api/)
- [Architecture Overview](../../architecture.md)
