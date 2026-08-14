#!/usr/bin/env python3
"""MCP stdio server that wraps the md-to-pdf CLI tool."""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Any

# ---------------------------------------------------------------------------
# Minimal MCP stdio implementation (no external MCP SDK required)
# Implements the JSON-RPC 2.0 subset used by MCP over stdio.
# ---------------------------------------------------------------------------

TOOL_NAME = "convert_markdown_to_pdf"
SERVER_NAME = "md-to-pdf"
SERVER_VERSION = "0.1.3"
PROTOCOL_VERSION = "2024-11-05"

_TOOLS = [
    {
        "name": TOOL_NAME,
        "description": (
            "Convert a Markdown file (with optional Mermaid diagrams) to a PDF "
            "document using a headless browser renderer."
        ),
        "inputSchema": {
            "type": "object",
            "properties": {
                "input": {
                    "type": "string",
                    "description": "Path to the Markdown file to convert.",
                },
                "output": {
                    "type": "string",
                    "description": (
                        "Output PDF path. Defaults to the input filename with a "
                        ".pdf extension."
                    ),
                },
                "title": {
                    "type": "string",
                    "description": (
                        "Document title stored in PDF metadata. Defaults to the "
                        "input filename without extension."
                    ),
                },
                "page_size": {
                    "type": "string",
                    "description": "CSS page size (e.g. A4, Letter, Legal). Defaults to A4.",
                    "default": "A4",
                },
                "allow_html": {
                    "type": "boolean",
                    "description": "Allow raw HTML in Markdown to pass through. Defaults to false.",
                    "default": False,
                },
                "allow_local_files": {
                    "type": "boolean",
                    "description": (
                        "Allow Chrome to access local files for assets referenced "
                        "in Markdown. Defaults to false."
                    ),
                    "default": False,
                },
                "browser": {
                    "type": "string",
                    "description": (
                        "Path to Chrome, Chromium, or Edge executable. Falls back "
                        "to MD_TO_PDF_BROWSER environment variable."
                    ),
                },
            },
            "required": ["input"],
        },
    }
]


def _find_md_to_pdf() -> str:
    """Locate the md-to-pdf executable."""
    # 1. Explicit env var override
    env_val = os.environ.get("MD_TO_PDF_BIN")
    if env_val:
        return env_val

    # 2. Sibling to this script (installed next to it)
    script_dir = Path(__file__).parent
    for candidate in [
        script_dir.parent / "target" / "release" / "md-to-pdf",
        script_dir.parent / "target" / "debug" / "md-to-pdf",
    ]:
        if candidate.is_file():
            return str(candidate)

    # 3. PATH
    import shutil
    found = shutil.which("md-to-pdf")
    if found:
        return found

    raise FileNotFoundError(
        "md-to-pdf binary not found. Build it with `cargo build --release` or "
        "set the MD_TO_PDF_BIN environment variable."
    )


def _run_tool(arguments: dict[str, Any]) -> dict[str, Any]:
    """Invoke md-to-pdf with the given arguments and return a result dict."""
    binary = _find_md_to_pdf()

    cmd: list[str] = [binary]

    input_path = arguments.get("input", "")
    if not input_path:
        return {"isError": True, "content": [{"type": "text", "text": "Missing required argument: input"}]}

    cmd.append(input_path)

    if output := arguments.get("output"):
        cmd.extend(["--output", output])

    if title := arguments.get("title"):
        cmd.extend(["--title", title])

    page_size = arguments.get("page_size", "A4")
    cmd.extend(["--page-size", page_size])

    if arguments.get("allow_html"):
        cmd.append("--allow-html")

    if arguments.get("allow_local_files"):
        cmd.append("--allow-local-files")

    if browser := arguments.get("browser"):
        cmd.extend(["--browser", browser])

    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=120,
        )
    except FileNotFoundError as exc:
        return {
            "isError": True,
            "content": [{"type": "text", "text": f"md-to-pdf binary not found: {exc}"}],
        }
    except subprocess.TimeoutExpired:
        return {
            "isError": True,
            "content": [{"type": "text", "text": "md-to-pdf timed out after 120 seconds."}],
        }

    if result.returncode != 0:
        stderr = result.stderr.strip()
        return {
            "isError": True,
            "content": [{"type": "text", "text": f"md-to-pdf failed:\n{stderr}"}],
        }

    output_msg = result.stdout.strip() or "Conversion complete."
    return {"content": [{"type": "text", "text": output_msg}]}


# ---------------------------------------------------------------------------
# JSON-RPC / MCP stdio transport
# ---------------------------------------------------------------------------

def _send(response: dict[str, Any]) -> None:
    line = json.dumps(response, ensure_ascii=False)
    sys.stdout.write(line + "\n")
    sys.stdout.flush()


def _error_response(id: Any, code: int, message: str) -> dict[str, Any]:
    return {"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}}


def _handle(request: dict[str, Any]) -> dict[str, Any] | None:
    """Return a response dict, or None if the request is a notification."""
    req_id = request.get("id")
    method = request.get("method", "")
    params = request.get("params") or {}

    if method == "initialize":
        return {
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": {"tools": {}},
                "serverInfo": {"name": SERVER_NAME, "version": SERVER_VERSION},
            },
        }

    if method == "initialized":
        return None  # notification, no response

    if method == "tools/list":
        return {
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {"tools": _TOOLS},
        }

    if method == "tools/call":
        name = params.get("name")
        arguments = params.get("arguments") or {}
        if name != TOOL_NAME:
            return _error_response(req_id, -32601, f"Unknown tool: {name}")
        result = _run_tool(arguments)
        return {"jsonrpc": "2.0", "id": req_id, "result": result}

    if method == "ping":
        return {"jsonrpc": "2.0", "id": req_id, "result": {}}

    # Unknown method
    if req_id is not None:
        return _error_response(req_id, -32601, f"Method not found: {method}")
    return None  # unknown notification


def main() -> None:
    for raw_line in sys.stdin:
        raw_line = raw_line.strip()
        if not raw_line:
            continue
        try:
            request = json.loads(raw_line)
        except json.JSONDecodeError as exc:
            _send(_error_response(None, -32700, f"Parse error: {exc}"))
            continue

        try:
            response = _handle(request)
        except Exception as exc:  # noqa: BLE001
            req_id = request.get("id")
            _send(_error_response(req_id, -32603, f"Internal error: {exc}"))
            continue

        if response is not None:
            _send(response)


if __name__ == "__main__":
    main()
