#!/usr/bin/env python3
"""MCP stdio server that wraps the md-to-pdf CLI tool."""

from __future__ import annotations

import hashlib
import json
import os
import platform
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import urllib.request
import zipfile
from pathlib import Path
from typing import Any

# ---------------------------------------------------------------------------
# Minimal MCP stdio implementation (no external MCP SDK required)
# Implements the JSON-RPC 2.0 subset used by MCP over stdio.
# ---------------------------------------------------------------------------

TOOL_NAME = "convert_markdown_to_pdf"
SERVER_NAME = "md-to-pdf"
SERVER_VERSION = "0.2.2"
BINARY_VERSION = "0.2.2"
PROTOCOL_VERSION = "2024-11-05"
REPOSITORY = "MiguelElGallo/md-to-pdf"

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
                "css": {
                    "type": "string",
                    "description": "Path to an extra CSS file to append after the built-in print styles.",
                },
                "mermaid_url": {
                    "type": "string",
                    "description": "Mermaid ES module URL for rendering diagrams. Overrides the default CDN URL.",
                },
            },
            "required": ["input"],
        },
    }
]


def _release_asset() -> tuple[str, str, str]:
    """Return the release target, archive suffix, and executable name."""
    system = platform.system()
    machine = platform.machine().lower()

    if system == "Darwin" and machine in {"arm64", "aarch64"}:
        return "aarch64-apple-darwin", "zip", "md-to-pdf"
    if system == "Darwin" and machine in {"x86_64", "amd64"}:
        return "x86_64-apple-darwin", "zip", "md-to-pdf"
    if system == "Linux" and machine in {"x86_64", "amd64"}:
        return "x86_64-unknown-linux-gnu", "tar.gz", "md-to-pdf"
    if system == "Windows" and machine in {"x86_64", "amd64"}:
        return "x86_64-pc-windows-msvc", "zip", "md-to-pdf.exe"

    raise RuntimeError(
        f"automatic installation is not available for {system} {platform.machine()}"
    )


def _plugin_data_dir() -> Path:
    """Return writable storage for the downloaded release binary."""
    plugin_data = os.environ.get("PLUGIN_DATA")
    if plugin_data and plugin_data != "${PLUGIN_DATA}":
        return Path(plugin_data)

    if sys.platform == "win32":
        cache_root = os.environ.get("LOCALAPPDATA")
        if cache_root:
            return Path(cache_root) / "md-to-pdf"

    cache_root = os.environ.get("XDG_CACHE_HOME")
    if cache_root:
        return Path(cache_root) / "md-to-pdf"
    return Path.home() / ".cache" / "md-to-pdf"


def _download_file(url: str, destination: Path) -> None:
    request = urllib.request.Request(url, headers={"User-Agent": "md-to-pdf-agent-plugin"})
    with (
        urllib.request.urlopen(request, timeout=60) as response,
        destination.open("wb") as output,
    ):
        shutil.copyfileobj(response, output)


def _expected_checksum(checksum_path: Path, archive_name: str) -> str:
    parts = checksum_path.read_text(encoding="utf-8").strip().split()
    if len(parts) < 2 or not re.fullmatch(r"[0-9a-fA-F]{64}", parts[0]):
        raise RuntimeError("release checksum file has an invalid format")
    if Path(parts[-1].lstrip("*")).name != archive_name:
        raise RuntimeError("release checksum does not name the downloaded archive")
    return parts[0].lower()


def _extract_binary(
    archive_path: Path, member_name: str, archive_suffix: str, destination: Path
) -> None:
    if archive_suffix == "zip":
        with (
            zipfile.ZipFile(archive_path) as archive,
            archive.open(member_name) as source,
            destination.open("wb") as output,
        ):
            shutil.copyfileobj(source, output)
        return

    with tarfile.open(archive_path, "r:gz") as archive:
        member = archive.getmember(member_name)
        if not member.isfile():
            raise RuntimeError("release archive binary entry is not a file")
        source = archive.extractfile(member)
        if source is None:
            raise RuntimeError("release archive binary could not be read")
        with source, destination.open("wb") as output:
            shutil.copyfileobj(source, output)


def _install_md_to_pdf() -> str:
    """Download and verify the release matching this plugin version."""
    target, archive_suffix, executable_name = _release_asset()
    tag = f"v{BINARY_VERSION}"
    archive_name = f"md-to-pdf-{tag}-{target}.{archive_suffix}"
    checksum_name = f"md-to-pdf-{tag}-{target}.sha256"
    base_url = f"https://github.com/{REPOSITORY}/releases/download/{tag}"
    install_dir = _plugin_data_dir() / "bin"
    installed_binary = install_dir / f"md-to-pdf-{tag}-{target}"
    if executable_name.endswith(".exe"):
        installed_binary = installed_binary.with_suffix(".exe")
    if installed_binary.is_file():
        return str(installed_binary)

    install_dir.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=install_dir) as temporary_dir:
        temporary_path = Path(temporary_dir)
        archive_path = temporary_path / archive_name
        checksum_path = temporary_path / checksum_name
        _download_file(f"{base_url}/{archive_name}", archive_path)
        _download_file(f"{base_url}/{checksum_name}", checksum_path)

        expected = _expected_checksum(checksum_path, archive_name)
        actual = hashlib.sha256(archive_path.read_bytes()).hexdigest()
        if actual != expected:
            raise RuntimeError("release archive checksum verification failed")

        member_name = f"md-to-pdf-{tag}-{target}/{executable_name}"
        staged_binary = temporary_path / executable_name
        _extract_binary(archive_path, member_name, archive_suffix, staged_binary)
        staged_binary.chmod(0o755)
        os.replace(staged_binary, installed_binary)

    return str(installed_binary)


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
    found = shutil.which("md-to-pdf")
    if found:
        return found

    if os.environ.get("MD_TO_PDF_AUTO_INSTALL", "1").lower() not in {
        "0",
        "false",
        "no",
    }:
        try:
            return _install_md_to_pdf()
        except Exception as exc:
            raise RuntimeError(
                "md-to-pdf binary was not found and automatic installation failed: "
                f"{exc}. Install it manually or set MD_TO_PDF_BIN."
            ) from exc

    raise FileNotFoundError(
        "md-to-pdf binary not found and automatic installation is disabled. "
        "Install it manually or set MD_TO_PDF_BIN."
    )


def _str_arg(value: Any, default: str = "") -> str:
    """Coerce an argument value to a non-empty string, or return default."""
    if value is None:
        return default
    s = str(value).strip()
    return s if s else default


def _run_tool(arguments: dict[str, Any]) -> dict[str, Any]:
    """Invoke md-to-pdf with the given arguments and return a result dict."""
    input_path = _str_arg(arguments.get("input"))
    if not input_path:
        return {"isError": True, "content": [{"type": "text", "text": "Missing required argument: input"}]}

    try:
        binary = _find_md_to_pdf()
    except (FileNotFoundError, RuntimeError) as exc:
        return {
            "isError": True,
            "content": [{"type": "text", "text": str(exc)}],
        }

    cmd: list[str] = [binary]

    if output := _str_arg(arguments.get("output")):
        cmd.extend(["--output", output])

    if title := _str_arg(arguments.get("title")):
        cmd.extend(["--title", title])

    page_size = _str_arg(arguments.get("page_size"), "A4")
    cmd.extend(["--page-size", page_size])

    if arguments.get("allow_html"):
        cmd.append("--allow-html")

    if arguments.get("allow_local_files"):
        cmd.append("--allow-local-files")

    if browser := _str_arg(arguments.get("browser")):
        cmd.extend(["--browser", browser])

    if css := _str_arg(arguments.get("css")):
        cmd.extend(["--css", css])

    if mermaid_url := _str_arg(arguments.get("mermaid_url")):
        cmd.extend(["--mermaid-url", mermaid_url])

    # Emit -- before the positional to prevent a leading-dash input value
    # from being parsed as a flag by clap.
    cmd.extend(["--", input_path])

    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=120,
            check=False,
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
        detail = result.stderr.strip() or result.stdout.strip() or f"exit code {result.returncode}"
        return {
            "isError": True,
            "content": [{"type": "text", "text": f"md-to-pdf failed:\n{detail}"}],
        }

    # Normalize output: if the CLI printed "Wrote <path>", extract just the path.
    stdout = result.stdout.strip()
    match = re.match(r"^Wrote (.+)$", stdout)
    output_msg = match.group(1) if match else (stdout or "Conversion complete.")
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
    if not isinstance(request, dict):
        return _error_response(None, -32600, "Invalid Request: expected a JSON object")

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

    if method == "initialized" or method.startswith("notifications/"):
        return None  # lifecycle/other notifications — no response

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

        if not isinstance(request, dict):
            _send(_error_response(None, -32600, "Invalid Request: expected a JSON object"))
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
