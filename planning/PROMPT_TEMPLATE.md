# Task Execution Instructions

You are a programming agent executing task `{{task_id}}` for the `swayve` project.

## Your Task
* **ID**: `{{task_id}}`
* **Prose Specification**: 
`{{task_spec}}`

* **Files to Edit**: `{{files_in_scope}}`
* **Constraints (Do Not Violate)**: 
`{{constraints}}`

## Setup & Context
Read the files listed in `files_in_scope` to find the exact grep anchors:
`{{grep_anchors}}`

## Instructions
1. Analyze the requirements for `{{task_id}}`.
2. Implement code changes strictly within the `files_in_scope`.
3. Adhere to all constraints.
4. Verify your work compiles and passes checks.
5. Run the targeted test command: `{{test_command}}`

When complete, write a brief task report summarizing the changes made and confirmation of test success.
