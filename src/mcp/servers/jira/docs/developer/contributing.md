# Contributing Guidelines

Thank you for your interest in contributing to the Jira MCP Server! This document outlines the process for contributing to the project and provides guidelines to follow.

## Code of Conduct

Please be respectful to all contributors and users. We aim to foster an inclusive and welcoming community.

## Getting Started

### Prerequisites

Before you begin, ensure you have:

- Python 3.11 or higher
- `uv` or `pip` package manager
- Git
- A Jira instance for testing (or use mocks)

### Setting Up the Development Environment

1. Fork the repository on GitHub.
2. Clone your fork locally:

```bash
git clone https://github.com/your-username/jira-mcp-server.git
cd jira-mcp-server
```

3. Set up a virtual environment:

```bash
# Using uv
uv venv

# Activate the virtual environment
source .venv/bin/activate  # On Unix/macOS
.venv\Scripts\activate     # On Windows
```

4. Install the package in development mode:

```bash
uv pip install -e .
```

5. Install development dependencies:

```bash
uv pip install pytest pytest-cov flake8 mypy black isort
```

## Development Workflow

### Creating a Feature Branch

Create a new branch for your feature or bugfix:

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b bugfix/issue-number
```

### Coding Standards

We follow these coding standards:

1. **PEP 8**: Follow the [PEP 8](https://pep8.org/) style guide.
2. **Type Hints**: Use Python type hints for all functions and methods.
3. **Docstrings**: Add docstrings to all functions, methods, and classes.
4. **Comments**: Add comments for complex or non-obvious code.
5. **Formatting**: Use [Black](https://black.readthedocs.io/) for code formatting.

Example:

```python
def process_data(input_data: Dict[str, Any]) -> List[str]:
    """
    Process the input data and return a list of results.
    
    Args:
        input_data: Dictionary containing the data to process.
        
    Returns:
        A list of processed results.
    """
    results = []
    # Process the data
    for key, value in input_data.items():
        # Process each item
        result = f"{key}: {process_value(value)}"
        results.append(result)
    return results
```

### Running Linters and Formatters

Before committing code, run the following tools:

```bash
# Format code
black src tests

# Sort imports
isort src tests

# Check for linting errors
flake8 src tests

# Check for type errors
mypy src
```

### Running Tests

Run tests to ensure your changes don't break existing functionality:

```bash
# Run all tests
export PYTHON_SRC=./src
uv run pytest

# Run tests with coverage
export PYTHON_SRC=./src
uv run pytest --cov=src/jira_mcp
```

### Committing Changes

1. Make small, focused commits.
2. Write clear commit messages with a subject line and, if necessary, a body.
3. Reference issue numbers when relevant.

Example commit message:

```
Fix error when handling empty ticket lists

The application was crashing when the Jira API returned an empty list of
tickets. This commit adds a null check and returns an appropriate message.

Fixes #42
```

### Submitting a Pull Request

1. Push your branch to GitHub:

```bash
git push origin feature/your-feature-name
```

2. Submit a pull request (PR) on GitHub.
3. In the PR description, explain your changes and reference any issues they address.
4. Wait for code review and address any feedback.

## Pull Request Guidelines

### PR Title and Description

- Use a clear, descriptive title.
- Describe the changes in detail.
- Reference related issues.

### PR Checklist

Ensure your PR:

- [ ] Passes all tests
- [ ] Includes tests for new functionality
- [ ] Updates documentation
- [ ] Follows coding standards
- [ ] Addresses the issue it claims to fix

### Code Review

All PRs require review before merging. Be open to feedback and respond to review comments.

## Testing Guidelines

### Writing Tests

- Write tests for all new functionality.
- Cover edge cases and error scenarios.
- Follow the [Testing Guide](./testing.md).

### Test Coverage

Aim for high test coverage, especially for critical components. Use the coverage report to identify untested code:

```bash
export PYTHON_SRC=./src
uv run pytest --cov=src/jira_mcp --cov-report=html
```

## Documentation Guidelines

### Code Documentation

- Add docstrings to all public functions, methods, and classes.
- Document parameters, return values, and exceptions.
- Explain complex algorithms with comments.

### User Documentation

When adding new features, update:

- User guide documentation
- API reference
- Configuration guide (if applicable)
- Example usage (if applicable)

### Developer Documentation

When changing internal architecture, update:

- Architecture overview
- Module documentation
- Extension guide (if applicable)

## Feature Request Process

To suggest a new feature:

1. Check if a similar feature request already exists.
2. Open a new issue with the "feature request" template.
3. Clearly describe the feature and its benefits.
4. If possible, outline an implementation approach.

## Bug Report Process

To report a bug:

1. Check if the bug has already been reported.
2. Open a new issue with the "bug report" template.
3. Provide steps to reproduce the bug.
4. Include error messages, stack traces, and logs if available.
5. Describe expected vs. actual behavior.

## Release Process

### Versioning

We use [Semantic Versioning](https://semver.org/) (SemVer):

- MAJOR version for incompatible API changes
- MINOR version for backward-compatible new functionality
- PATCH version for backward-compatible bug fixes

### Release Process

1. Update the version number in appropriate files.
2. Update the changelog.
3. Create a new release on GitHub with release notes.
4. Publish to PyPI (if applicable).

## Project Structure

```
jira-mcp-server/
├── docs/
│   ├── user-guide/
│   └── developer/
├── src/
│   └── jira_mcp/
│       ├── __init__.py
│       ├── __main__.py
│       ├── client.py
│       ├── formatters.py
│       ├── server.py
│       ├── tools.py
│       └── fastmcp_server.py
├── tests/
│   ├── __init__.py
│   ├── test_client.py
│   ├── test_formatters.py
│   ├── test_server.py
│   ├── test_tools.py
│   └── test_fastmcp.py
├── pyproject.toml
└── README.md
```

## Developer Tips

### Working with the Jira API

The `jira` Python package provides comprehensive access to the Jira API. Some useful resources:

- [jira-python Documentation](https://jira.readthedocs.io/en/latest/)
- [Jira REST API Documentation](https://developer.atlassian.com/cloud/jira/platform/rest/v3/intro/)

### Working with FastAPI

For FastAPI-related development:

- [FastAPI Documentation](https://fastapi.tiangolo.com/)
- [Pydantic Documentation](https://docs.pydantic.dev/)

### Working with MCP

For MCP-related development:

- [MCP Repository](https://github.com/openai/mcp)
- [MCP Specification](https://github.com/openai/mcp/blob/main/SPEC.md)

### Debugging Tips

1. Enable debug logging:

```bash
python -m jira_mcp --debug
```

2. Use Python's `pdb` debugger to step through code:

```python
import pdb; pdb.set_trace()
```

3. Use logging for visibility into runtime behavior:

```python
import logging
logger = logging.getLogger(__name__)
logger.debug("Variable value: %s", variable)
```

## Getting Help

If you need help with contributing:

1. Check the documentation
2. Open an issue with the "question" label
3. Reach out to the maintainers

## Thank You!

Your contributions help make this project better for everyone. Thank you for your time and effort!