# User Interaction Flows

This directory contains documentation for user-triggered flows in the voxel engine. These flows are initiated by direct user input and typically result in changes to the game state.

## Core Flows

### Camera Controls
- [Camera Movement](./camera_movement.md) (Planned)
- [Camera Look](./camera_look.md) (Planned)

### Player Actions
- [Block Placement](./block_placement.md) (Planned)
- [Block Breaking](./block_breaking.md) (Planned)
- [Inventory Management](./inventory.md) (Planned)

### UI Interactions
- [Main Menu](./ui/main_menu.md) (Planned)
- [Pause Menu](./ui/pause_menu.md) (Planned)
- [Inventory UI](./ui/inventory_ui.md) (Planned)

## Adding New Flows

1. Create a new markdown file in the appropriate subdirectory
2. Follow the template in the root `domain-flow.workflow`
3. Update this README with a link to your new flow
4. Ensure all related flows are cross-linked

## Cross-References

- [System Flows](../system/README.md)
- [API Documentation](../../api/)
- [Controls Reference](../../controls.md)
