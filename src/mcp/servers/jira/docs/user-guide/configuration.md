# Configuration Guide

This document covers the configuration options for the Jira MCP Server.

## Environment Variables

The Jira MCP Server requires the following environment variables:

### Required Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `JIRA_URL` | The base URL of your Jira instance | For Cisco employees: `https://cisco-jira.atlassian.net` |
| `JIRA_API_USER` | Your Jira username (usually email address) | None |
| `JIRA_API_TOKEN` | Your Jira API token | None |

### Setting Environment Variables

#### Method 1: Shell Environment

```bash
# Unix/macOS
export JIRA_URL="https://your-domain.atlassian.net"
export JIRA_API_USER="user@example.com" 
export JIRA_API_TOKEN="your-api-token"

# Windows (CMD)
set JIRA_URL=https://your-domain.atlassian.net
set JIRA_API_USER=user@example.com
set JIRA_API_TOKEN=your-api-token

# Windows (PowerShell)
$env:JIRA_URL="https://your-domain.atlassian.net"
$env:JIRA_API_USER="user@example.com"
$env:JIRA_API_TOKEN="your-api-token"
```

#### Method 2: .env File

Create a file named `.env` in the root directory of your project:

```
JIRA_URL=https://your-domain.atlassian.net
JIRA_API_USER=user@example.com
JIRA_API_TOKEN=your-api-token
```

The server will automatically load these variables from the `.env` file if the `python-dotenv` package is installed (included in the package dependencies).

## Obtaining Jira Credentials

### Jira API Token

To obtain a Jira API token:

1. Log in to the Atlassian account linked to your Jira instance
2. Navigate to Account Settings > Security > API tokens
3. Click "Create API token"
4. Give it a name (e.g., "Jira MCP Server")
5. Copy the token value (you won't be able to see it again!)

### Testing Your Credentials

You can test your credentials by running the server:

```bash
python -m jira_mcp
```

Then sending a test command:

```bash
echo '{"command": "jira.list-assigned-tickets"}' | python -m jira_mcp
```

If the credentials are correct, you should receive a JSON response with your assigned tickets.

## Command-line Options

When running the server, you can use the following command-line options:

| Option | Description | Default |
|--------|-------------|---------|
| `--mode` | Server mode: `standard` or `web` | `standard` |
| `--port` | Port to use for web server mode | `8000` |
| `--host` | Host to bind to in web server mode | `0.0.0.0` (all interfaces) |
| `--debug` | Enable debug logging | `False` |

Example:

```bash
# Run in web mode on port 9000 with debug logging
python -m jira_mcp --mode web --port 9000 --debug
```

## Logging

The server uses Python's logging module and configures logging based on the `--debug` flag:

- Without `--debug`: Logs INFO level and above
- With `--debug`: Logs DEBUG level and above

Log format:
```
%(asctime)s - %(name)s - %(levelname)s - %(message)s
```

## Advanced Configuration

### FastAPI CORS Configuration

The web server mode uses FastAPI with CORS (Cross-Origin Resource Sharing) middleware enabled. By default, it allows all origins (`"*"`).

In a production environment, you may want to specify the allowed origins explicitly by modifying the source code in `src/jira_mcp/fastmcp_server.py`:

```python
app.add_middleware(
    CORSMiddleware,
    allow_origins=["https://your-frontend-domain.com"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)
```

### Cisco Default Settings

For Cisco employees, the server includes default settings for connecting to the Cisco Jira instance. These defaults are used if the `JIRA_URL` environment variable is not set:

- Jira URL: `https://cisco-jira.atlassian.net`

## Next Steps

- [Standard MCP Mode](./standard-mode.md)
- [Web Server Mode](./web-mode.md)
- [Troubleshooting](./troubleshooting.md)