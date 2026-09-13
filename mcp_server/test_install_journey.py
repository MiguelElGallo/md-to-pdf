"""Opt-in packaged-plugin install/upgrade smoke tests with a real CLI and browser.

Run after ``cargo build --locked`` with MD_TO_PDF_RUN_INSTALL_JOURNEY=1 and
MD_TO_PDF_JOURNEY_BINARY pointing to the built executable. Only release downloads
are replaced by local, checksummed archives; conversion executes the real binary.
The next patch version is simulated using the same binary, testing versioned cache
selection rather than claiming compatibility with a future release.
"""

from __future__ import annotations

import hashlib
import importlib.util
import json
import os
import shutil
import subprocess
import tarfile
import tempfile
import unittest
import zipfile
from pathlib import Path
from types import ModuleType
from unittest import mock
from urllib.parse import urlparse

REPOSITORY_ROOT = Path(__file__).resolve().parents[1]


def _browser_path() -> str:
    """Find an explicit or preinstalled Chromium-family browser on CI platforms."""
    explicit = os.environ.get("MD_TO_PDF_BROWSER")
    if explicit:
        if Path(explicit).is_file():
            return explicit
        raise RuntimeError(f"MD_TO_PDF_BROWSER is not a file: {explicit}")

    for name in (
        "google-chrome",
        "google-chrome-stable",
        "chromium",
        "chromium-browser",
        "microsoft-edge",
        "msedge",
        "chrome",
    ):
        found = shutil.which(name)
        if found:
            return found
    candidates = [
        Path("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"),
        Path("/Applications/Chromium.app/Contents/MacOS/Chromium"),
        Path("/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge"),
    ]
    for variable in ("PROGRAMFILES(X86)", "PROGRAMFILES", "LOCALAPPDATA"):
        if directory := os.environ.get(variable):
            candidates.extend(
                Path(directory) / suffix
                for suffix in (
                    "Microsoft/Edge/Application/msedge.exe",
                    "Google/Chrome/Application/chrome.exe",
                    "Chromium/Application/chrome.exe",
                )
            )
    for candidate in candidates:
        if candidate.is_file():
            return str(candidate)
    raise RuntimeError("Install Chrome/Chromium/Edge or set MD_TO_PDF_BROWSER")


def _load_packaged_server(plugin_directory: Path) -> ModuleType:
    """Load the isolated marketplace payload, never the canonical repository copy."""
    path = plugin_directory / "mcp_server/server.py"
    spec = importlib.util.spec_from_file_location("isolated_md_to_pdf_server", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"Cannot load packaged server at {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _release_fixture(
    directory: Path,
    binary: Path,
    version: str,
    target: str,
    suffix: str,
    executable: str,
) -> None:
    """Package a real executable using the published release layout and checksum."""
    stem = f"md-to-pdf-v{version}-{target}"
    archive_path = directory / f"{stem}.{suffix}"
    member = f"{stem}/{executable}"
    if suffix == "zip":
        with zipfile.ZipFile(archive_path, "w", zipfile.ZIP_DEFLATED) as archive:
            archive.write(binary, member)
    else:
        with tarfile.open(archive_path, "w:gz") as archive:
            archive.add(binary, arcname=member)
    digest = hashlib.sha256(archive_path.read_bytes()).hexdigest()
    (directory / f"{stem}.sha256").write_text(
        f"{digest}  ./{archive_path.name}\n", encoding="utf-8"
    )


@unittest.skipUnless(
    os.environ.get("MD_TO_PDF_RUN_INSTALL_JOURNEY") == "1",
    "set MD_TO_PDF_RUN_INSTALL_JOURNEY=1 for real browser install/upgrade tests",
)
class InstallJourneyTests(unittest.TestCase):
    def test_manifest_launcher_initializes_and_converts_over_stdio(self) -> None:
        """Run the shipped launcher unchanged, including python3 lookup on Windows."""
        binary_value = os.environ.get("MD_TO_PDF_JOURNEY_BINARY")
        self.assertTrue(binary_value, "Set MD_TO_PDF_JOURNEY_BINARY to a built CLI")
        assert binary_value is not None
        binary = Path(binary_value).resolve()
        self.assertTrue(binary.is_file(), f"Build the CLI first: {binary}")
        browser = _browser_path()

        with tempfile.TemporaryDirectory(prefix="md-to-pdf-launcher-") as directory:
            root = Path(directory).resolve()
            plugin = root / "isolated-plugin"
            shutil.copytree(REPOSITORY_ROOT / "plugins/md-to-pdf", plugin)
            manifest = json.loads((plugin / ".mcp.json").read_text(encoding="utf-8"))
            configuration = manifest["mcpServers"]["md-to-pdf"]
            command = [
                value.replace("${PLUGIN_ROOT}", str(plugin))
                for value in [configuration["command"], *configuration.get("args", [])]
            ]
            environment = os.environ.copy()
            environment.update(
                {
                    name: value.replace("${PLUGIN_ROOT}", str(plugin))
                    for name, value in configuration.get("env", {}).items()
                }
            )
            # The install journey above covers managed downloads. This test must
            # exercise the actual declared launcher without hitting an unpublished
            # release or replacing python3 with the test runner's interpreter.
            environment.update(
                PLUGIN_ROOT=str(plugin),
                PLUGIN_DATA=str(root / "plugin-data"),
                MD_TO_PDF_BIN=str(binary),
                MD_TO_PDF_AUTO_INSTALL="0",
                MD_TO_PDF_BROWSER=browser,
            )
            source = root / "launcher document.md"
            source.write_text(
                "# Manifest launcher\n\nReal stdio conversion.\n", encoding="utf-8"
            )
            output = root / "launcher document.pdf"
            requests = [
                {
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "initialize",
                    "params": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {},
                        "clientInfo": {"name": "install-journey", "version": "1.0"},
                    },
                },
                {"jsonrpc": "2.0", "method": "notifications/initialized"},
                {
                    "jsonrpc": "2.0",
                    "id": 2,
                    "method": "tools/call",
                    "params": {
                        "name": "convert_markdown_to_pdf",
                        "arguments": {"input": str(source), "output": str(output)},
                    },
                },
            ]
            completed = subprocess.run(
                command,
                input="".join(json.dumps(request) + "\n" for request in requests),
                text=True,
                capture_output=True,
                cwd=root,
                env=environment,
                timeout=180,
                check=False,
            )
            self.assertEqual(completed.returncode, 0, completed.stderr)
            responses = [json.loads(line) for line in completed.stdout.splitlines()]
            self.assertEqual(len(responses), 2, responses)
            for response, request_id in zip(responses, (1, 2), strict=True):
                self.assertEqual(response["jsonrpc"], "2.0")
                self.assertEqual(response["id"], request_id)
                self.assertNotIn("error", response, response)
            version = json.loads((plugin / "plugin.json").read_text(encoding="utf-8"))[
                "version"
            ]
            self.assertEqual(responses[0]["result"]["serverInfo"]["version"], version)
            result = responses[1]["result"]
            self.assertFalse(result.get("isError", False), result)
            self.assertEqual(Path(result["content"][0]["text"]), output)
            self.assertTrue(output.read_bytes().startswith(b"%PDF-"))
            self.assertGreater(output.stat().st_size, 1000)

    def test_fresh_install_cached_conversion_and_patch_upgrade(self) -> None:
        binary_value = os.environ.get("MD_TO_PDF_JOURNEY_BINARY")
        self.assertTrue(binary_value, "Set MD_TO_PDF_JOURNEY_BINARY to a built CLI")
        assert binary_value is not None
        binary = Path(binary_value).resolve()
        self.assertTrue(binary.is_file(), f"Build the CLI first: {binary}")
        explicit_browser = os.environ.get("MD_TO_PDF_BROWSER")
        browser = _browser_path()

        with tempfile.TemporaryDirectory(prefix="md-to-pdf-journey-") as directory:
            root = Path(directory).resolve()
            plugin = root / "isolated-plugin"
            shutil.copytree(REPOSITORY_ROOT / "plugins/md-to-pdf", plugin)
            server = _load_packaged_server(plugin)
            assert server.__file__ is not None
            self.assertEqual(Path(server.__file__).resolve().parent.parent, plugin)
            target, suffix, executable = server._release_asset()
            initial_version = server.BINARY_VERSION
            major, minor, patch = map(int, initial_version.split("."))
            next_version = f"{major}.{minor}.{patch + 1}"
            fixtures = root / "release-fixtures"
            fixtures.mkdir()
            for version in (initial_version, next_version):
                _release_fixture(fixtures, binary, version, target, suffix, executable)

            def copy_download(url: str, destination: Path) -> None:
                source = fixtures / Path(urlparse(url).path).name
                self.assertTrue(source.is_file(), f"Unexpected release download: {url}")
                shutil.copyfile(source, destination)

            source = root / "first document.md"
            source.write_text(
                "# Installation journey\n\nA real PDF from an isolated plugin cache.\n",
                encoding="utf-8",
            )
            cache = root / "fresh managed cache"
            environment = os.environ.copy()
            environment.pop("MD_TO_PDF_BIN", None)
            environment.update(
                PLUGIN_DATA=str(cache),
                MD_TO_PDF_AUTO_INSTALL="1",
                MD_TO_PDF_BROWSER=browser,
            )
            self.assertFalse(cache.exists())
            with (
                mock.patch.dict(os.environ, environment, clear=True),
                mock.patch.object(
                    server, "_download_file", side_effect=copy_download
                ) as download,
            ):
                outputs = []
                installed_paths = []
                for index, version in enumerate(
                    (initial_version, initial_version, next_version)
                ):
                    if index == 2 and not explicit_browser:
                        # Cover native browser discovery on the upgrade, notably
                        # Windows installations that do not put Edge on PATH,
                        # while retaining a caller's custom browser override.
                        os.environ.pop("MD_TO_PDF_BROWSER", None)
                    server.__dict__["BINARY_VERSION"] = version
                    output = root / f"conversion {index}.pdf"
                    response = server._handle(
                        {
                            "jsonrpc": "2.0",
                            "id": index + 1,
                            "method": "tools/call",
                            "params": {
                                "name": server.TOOL_NAME,
                                "arguments": {
                                    "input": str(source),
                                    "output": str(output),
                                },
                            },
                        }
                    )
                    self.assertIsNotNone(response)
                    assert response is not None
                    self.assertNotIn("error", response, response)
                    result = response["result"]
                    self.assertFalse(result.get("isError", False), result)
                    self.assertEqual(Path(result["content"][0]["text"]), output)
                    self.assertTrue(output.read_bytes().startswith(b"%PDF-"))
                    self.assertGreater(output.stat().st_size, 1000)
                    installed = Path(server._find_md_to_pdf())
                    self.assertEqual(installed.parent, cache / "bin")
                    self.assertIn(f"v{version}-{target}", installed.name)
                    if executable.endswith(".exe"):
                        self.assertTrue(installed.name.endswith(".exe"))
                    installed_paths.append(installed)
                    outputs.append(output)
                    self.assertEqual(download.call_count, 2 if index < 2 else 4)

                self.assertEqual(installed_paths[0], installed_paths[1])
                self.assertNotEqual(installed_paths[0], installed_paths[2])
                self.assertTrue(all(path.is_file() for path in installed_paths))
                self.assertEqual(len(set(outputs)), 3)


if __name__ == "__main__":
    unittest.main()
