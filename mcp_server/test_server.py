from __future__ import annotations

import hashlib
import io
import json
import os
import shutil
import subprocess
import sys
import tarfile
import tempfile
import tomllib
import unittest
from pathlib import Path
from unittest import mock

from mcp_server import server

REPOSITORY_ROOT = Path(__file__).resolve().parents[1]


class ReleaseAssetTests(unittest.TestCase):
    def test_selects_apple_silicon_asset(self) -> None:
        with (
            mock.patch.object(server.platform, "system", return_value="Darwin"),
            mock.patch.object(server.platform, "machine", return_value="arm64"),
        ):
            self.assertEqual(
                server._release_asset(),
                ("aarch64-apple-darwin", "zip", "md-to-pdf"),
            )

    def test_selects_windows_asset(self) -> None:
        with (
            mock.patch.object(server.platform, "system", return_value="Windows"),
            mock.patch.object(server.platform, "machine", return_value="AMD64"),
        ):
            self.assertEqual(
                server._release_asset(),
                ("x86_64-pc-windows-msvc", "zip", "md-to-pdf.exe"),
            )


class AutomaticInstallTests(unittest.TestCase):
    def test_downloads_verifies_and_caches_release_binary(self) -> None:
        target = "x86_64-unknown-linux-gnu"
        tag = f"v{server.BINARY_VERSION}"
        archive_name = f"md-to-pdf-{tag}-{target}.tar.gz"
        member_name = f"md-to-pdf-{tag}-{target}/md-to-pdf"

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture_archive = root / archive_name
            payload = b"test release binary"
            with tarfile.open(fixture_archive, "w:gz") as archive:
                info = tarfile.TarInfo(member_name)
                info.size = len(payload)
                info.mode = 0o755
                archive.addfile(info, io.BytesIO(payload))
            fixture_checksum = root / f"md-to-pdf-{tag}-{target}.sha256"
            fixture_checksum.write_text(
                f"{hashlib.sha256(fixture_archive.read_bytes()).hexdigest()}  ./{archive_name}\n",
                encoding="utf-8",
            )

            def copy_download(url: str, destination: Path) -> None:
                source = fixture_checksum if url.endswith(".sha256") else fixture_archive
                shutil.copyfile(source, destination)

            plugin_data = root / "plugin-data"
            with (
                mock.patch.dict(os.environ, {"PLUGIN_DATA": str(plugin_data)}, clear=False),
                mock.patch.object(
                    server,
                    "_release_asset",
                    return_value=(target, "tar.gz", "md-to-pdf"),
                ),
                mock.patch.object(server, "_download_file", side_effect=copy_download) as download,
            ):
                installed = Path(server._install_md_to_pdf())
                cached = Path(server._install_md_to_pdf())

            self.assertEqual(installed, cached)
            self.assertEqual(installed.read_bytes(), payload)
            self.assertEqual(download.call_count, 2)

    def test_rejects_a_checksum_mismatch(self) -> None:
        target = "x86_64-unknown-linux-gnu"
        tag = f"v{server.BINARY_VERSION}"
        archive_name = f"md-to-pdf-{tag}-{target}.tar.gz"

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)

            def bad_download(url: str, destination: Path) -> None:
                if url.endswith(".sha256"):
                    destination.write_text(
                        f"{'0' * 64}  ./{archive_name}\n", encoding="utf-8"
                    )
                else:
                    destination.write_bytes(b"not the expected archive")

            with (
                mock.patch.dict(os.environ, {"PLUGIN_DATA": str(root)}, clear=False),
                mock.patch.object(
                    server,
                    "_release_asset",
                    return_value=(target, "tar.gz", "md-to-pdf"),
                ),
                mock.patch.object(server, "_download_file", side_effect=bad_download),
                self.assertRaisesRegex(RuntimeError, "checksum verification failed"),
            ):
                server._install_md_to_pdf()


class ToolInvocationTests(unittest.TestCase):
    def test_reports_automatic_install_failure_as_a_tool_error(self) -> None:
        with mock.patch.object(
            server, "_find_md_to_pdf", side_effect=RuntimeError("download failed")
        ):
            result = server._run_tool({"input": "input.md"})

        self.assertTrue(result["isError"])
        self.assertEqual(result["content"][0]["text"], "download failed")

    def test_builds_each_optional_argument_once(self) -> None:
        completed = subprocess.CompletedProcess([], 0, "Wrote output.pdf\n", "")
        arguments = {
            "input": "input.md",
            "output": "output.pdf",
            "allow_html": True,
            "browser": "/browser",
        }
        with (
            mock.patch.object(server, "_find_md_to_pdf", return_value="/md-to-pdf"),
            mock.patch.object(server.subprocess, "run", return_value=completed) as run,
        ):
            result = server._run_tool(arguments)

        command = run.call_args.args[0]
        self.assertEqual(command.count("--allow-html"), 1)
        self.assertEqual(command.count("--browser"), 1)
        self.assertEqual(command[-2:], ["--", "input.md"])
        self.assertEqual(result["content"][0]["text"], "output.pdf")


class PluginPackagingTests(unittest.TestCase):
    def test_release_versions_match(self) -> None:
        expected = server.SERVER_VERSION
        cargo = tomllib.loads(
            (REPOSITORY_ROOT / "Cargo.toml").read_text(encoding="utf-8")
        )
        manifests = [
            REPOSITORY_ROOT / "plugin.json",
            REPOSITORY_ROOT / "plugins/md-to-pdf/plugin.json",
            REPOSITORY_ROOT / "plugins/md-to-pdf/.codex-plugin/plugin.json",
        ]
        marketplace = json.loads(
            (REPOSITORY_ROOT / ".github/plugin/marketplace.json").read_text(
                encoding="utf-8"
            )
        )

        self.assertEqual(cargo["package"]["version"], expected)
        self.assertEqual(server.BINARY_VERSION, expected)
        for manifest in manifests:
            data = json.loads(manifest.read_text(encoding="utf-8"))
            self.assertEqual(data["version"], expected, manifest)
        self.assertEqual(marketplace["metadata"]["version"], expected)
        self.assertEqual(marketplace["plugins"][0]["version"], expected)

    def test_marketplace_server_runs_from_packaged_copy(self) -> None:
        wrapper = REPOSITORY_ROOT / "plugins/md-to-pdf/mcp_server/server.py"
        request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {},
        }
        completed = subprocess.run(
            [sys.executable, str(wrapper)],
            input=json.dumps(request) + "\n",
            text=True,
            capture_output=True,
            check=True,
        )
        response = json.loads(completed.stdout)

        self.assertEqual(response["result"]["serverInfo"]["name"], "md-to-pdf")
        self.assertEqual(response["result"]["serverInfo"]["version"], server.SERVER_VERSION)

    def test_marketplace_server_matches_canonical_server(self) -> None:
        canonical = REPOSITORY_ROOT / "mcp_server/server.py"
        packaged = REPOSITORY_ROOT / "plugins/md-to-pdf/mcp_server/server.py"

        self.assertEqual(packaged.read_bytes(), canonical.read_bytes())


if __name__ == "__main__":
    unittest.main()
