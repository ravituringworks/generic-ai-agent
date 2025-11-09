# Employee Name to ID Mappings

This document provides a reference for the employee name replacements made in the example files.

All employee names in the examples have been replaced with unique random IDs (EMP001-EMP031) to anonymize the data.

## Name Mapping Table

| Employee ID | Original Name          |
|-------------|------------------------|
| EMP001      | Alice Chen             |
| EMP002      | Bob Martinez           |
| EMP003      | Carol Kim              |
| EMP004      | David Johnson          |
| EMP005      | Emily Zhang            |
| EMP006      | Frank Wilson           |
| EMP007      | Grace Lee              |
| EMP008      | Henry Patel            |
| EMP009      | Iris Anderson          |
| EMP010      | Jack Thompson          |
| EMP011      | Kate Brown             |
| EMP012      | Leo Garcia             |
| EMP013      | Maya Nguyen            |
| EMP014      | Noah Davis             |
| EMP015      | Olivia Torres          |
| EMP016      | Paul Chen              |
| EMP017      | Quinn Rivera           |
| EMP018      | Rachel Kim             |
| EMP019      | Sam Johnson            |
| EMP020      | Tina Martinez          |
| EMP021      | Uma Patel              |
| EMP022      | Victor Wong            |
| EMP023      | Wendy Anderson         |
| EMP024      | Xavier Lopez           |
| EMP025      | Yara Hassan            |
| EMP026      | Zack Thompson          |
| EMP027      | Dr. Sarah Chen         |
| EMP028      | Dr. James Park         |
| EMP029      | Dr. Lisa Wang          |
| EMP030      | Marcus Johnson         |
| EMP031      | John Doe               |

## Files Modified

The following files were updated with the name replacements:

### Rust Files (.rs)

- `simple_coordinator_test.rs`
- `collaborative_robotics_workspace.rs`
- `collaborative_robotics_complex.rs`
- `humanoid_robot_project.rs`
- `robotech_industries_organization_example.rs`
- `agent_network_system.rs`

### Markdown Documentation (.md)

- `docs/MULTI_MODEL_CONFIG.md`
- `docs/COLLABORATIVE_WORKSPACE_SUCCESS.md`
- `docs/COLLABORATIVE_RESULTS.md`
- `docs/COMPLEX_WORKSPACE.md`
- `docs/WORKSPACE_COMPARISON.md`
- `docs/SCALING_ENGINEER_TASKS.md`
- `docs/COLLABORATIVE_COMPARISON.md`
- `docs/ROBOTECH-ORGANIZATION.md`
- `docs/ROBOTECH_ARTIFACTS.md`

### Configuration Files (.toml)

- `collaborative_workspace_config.toml`

## Notes

- Technical terms like "Grace CPU" and "Jetson" were preserved and not replaced
- Generic references like "Johnson, A." in citations were left unchanged as they are not specific employee names
- Agent naming patterns like `SimulationEngineer_Alice` were converted to `SimulationEngineer_EMP001`

## Replacement Script

The replacement was performed using the Python script located at:
`scripts/replace_employee_names.py`

This script can be re-run if additional files need to be processed in the future.
