# Troubleshooting Guide

This guide helps you identify and fix common issues when using the Jira MCP Server.

## Connection Issues

### Cannot Connect to Jira

**Symptoms:**
- Errors like `Failed to connect to Jira`
- Empty lists returned when tickets should be present
- Error message: `Error: getaddrinfo ENOTFOUND`

**Possible Causes and Solutions:**

1. **Incorrect URL**
   - **Issue:** The Jira URL is incorrect or malformed.
   - **Solution:** Double-check the `JIRA_URL` environment variable.
   ```bash
   # Format example
   export JIRA_URL="https://your-domain.atlassian.net"
   ```

2. **Network Issues**
   - **Issue:** No internet connection or firewall blocking access.
   - **Solution:** Check your internet connection and try accessing Jira directly via browser.

3. **VPN Requirements**
   - **Issue:** Your Jira instance requires VPN access.
   - **Solution:** Connect to the required VPN before starting the server.

### Authentication Failures

**Symptoms:**
- Error messages containing `401 Unauthorized`
- Error: `Invalid credentials`

**Possible Causes and Solutions:**

1. **Incorrect Credentials**
   - **Issue:** Username or API token is incorrect.
   - **Solution:** Verify your `JIRA_API_USER` and `JIRA_API_TOKEN` environment variables.

2. **Expired Token**
   - **Issue:** Your Jira API token has expired.
   - **Solution:** Generate a new API token in your Atlassian account settings.

3. **Permission Issues**
   - **Issue:** Your account doesn't have the required permissions.
   - **Solution:** Ensure your Jira account has permission to view and access tickets.

## Data Issues

### No Tickets Found

**Symptoms:**
- Empty lists returned when tickets should be present
- Message: `No assigned tickets found`

**Possible Causes and Solutions:**

1. **No Assigned Tickets**
   - **Issue:** There are genuinely no tickets assigned to your user.
   - **Solution:** This may not be an issue. Try assigning a ticket to yourself for testing.

2. **Wrong User**
   - **Issue:** You're authenticated as a different user than you expect.
   - **Solution:** Verify your `JIRA_API_USER` environment variable.

3. **JQL Query Issues**
   - **Issue:** The JQL query used internally might not match your Jira configuration.
   - **Solution:** Enable debug logs (`--debug` flag) to see the JQL queries being used.

### Ticket Not Found

**Symptoms:**
- Error: `Ticket not found: PROJECT-123`

**Possible Causes and Solutions:**

1. **Invalid Ticket ID**
   - **Issue:** The ticket ID doesn't exist or is incorrectly formatted.
   - **Solution:** Verify the ticket ID and ensure it exists in your Jira instance.

2. **Permission Issues**
   - **Issue:** You don't have permission to view this specific ticket.
   - **Solution:** Check your permissions in Jira for this project/ticket.

## Server Issues

### Server Won't Start

**Symptoms:**
- Error messages when starting the server
- Process terminates immediately

**Possible Causes and Solutions:**

1. **Environment Variables Not Set**
   - **Issue:** Required environment variables are missing.
   - **Solution:** Ensure all required environment variables are set (see [Configuration Guide](./configuration.md)).

2. **Port Already in Use**
   - **Issue:** When running in web mode, the port is already being used by another process.
   - **Solution:** Specify a different port with the `--port` option.
   ```bash
   python -m jira_mcp --mode web --port 9000
   ```

3. **Python Environment Issues**
   - **Issue:** Missing dependencies or incorrect Python version.
   - **Solution:** 
     - Make sure you're using Python 3.11 or higher
     - Reinstall dependencies: `uv pip install -e .`

### Performance Issues

**Symptoms:**
- Slow response times
- Timeouts

**Possible Causes and Solutions:**

1. **Large Number of Tickets**
   - **Issue:** Processing many tickets can be slow.
   - **Solution:** Limit the number of tickets retrieved by using more specific queries.

2. **Network Latency**
   - **Issue:** Slow connection to Jira.
   - **Solution:** Ensure you have a stable internet connection, or run the server closer to your Jira instance if possible.

3. **Jira API Rate Limiting**
   - **Issue:** You're hitting Jira API rate limits.
   - **Solution:** Implement caching or reduce the frequency of requests.

## Error Messages

### Common Error Messages and Fixes

#### "Failed to import JIRA module"
**Solution:** Ensure the `jira` package is installed: `uv pip install jira`

#### "No module named 'fastapi'"
**Solution:** Ensure you've installed the package with all dependencies: `uv pip install -e .`

#### "No module named 'dotenv'"
**Solution:** Ensure the `python-dotenv` package is installed: `uv pip install python-dotenv`

#### "Unknown command: jira.unknown-command"
**Solution:** Check the available commands using `list-commands` and ensure you're using the correct command name.

#### "HTTPError: 400 Client Error: Bad Request"
**Solution:** This usually indicates an issue with the JQL query or other API parameters. Enable debug logging to see the full error details.

## Debug Mode

Enable debug logging to get more information about errors:

```bash
# Standard mode with debug logging
python -m jira_mcp --debug

# Web mode with debug logging
python -m jira_mcp --mode web --debug
```

In debug mode, you'll see:
- Detailed error messages
- API requests and responses
- JQL queries being used
- Connection attempts

## Getting Help

If you're still experiencing issues, try the following:

1. **Check Logs**: Enable debug logging and look for error messages.
2. **Check Jira Status**: Ensure the Jira instance is online and accessible.
3. **Update Dependencies**: Ensure you have the latest version of the package and its dependencies.
4. **Report Issues**: If you believe you've found a bug, report it on the project's issue tracker with detailed information:
   - The command or endpoint you're trying to use
   - The error message you're receiving
   - Your environment (OS, Python version)
   - Steps to reproduce the issue (if possible)