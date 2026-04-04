# Exercise Tracker

## Architectural Decision Records

Before implementing anything, search the ADRs using the `search_adrs` and `list_adrs` MCP tools to check for relevant decisions. Follow any applicable ADRs. If a decision contradicts what you're about to do, flag it to the user before proceeding.

Key ADRs that are likely relevant:
- **ADR-001**: Python Project Structure and Tooling
- **ADR-002**: AWS CDK Infrastructure
- **ADR-004**: Python Package Management (PyPI and uv)
- **ADR-008**: Lambda Packaging with Flat Layout and uv Workspaces
- **ADR-009**: Monorepo with Shared Packages for Multi-App Reuse

## Development

- Use `uv` for Python package management and virtual environments
- Follow the project structure conventions from the ADRs
- Run tests with `pytest`
- Run linting with `ruff check` and formatting with `ruff format`
