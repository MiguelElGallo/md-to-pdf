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
import zipfile
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
    def test_windows_patch_upgrades_have_distinct_cache_paths(self) -> None:
        target = "x86_64-pc-windows-msvc"
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)

            def download(url: str, destination: Path) -> None:
                tag = f"v{server.BINARY_VERSION}"
                archive_name = f"md-to-pdf-{tag}-{target}.zip"
                if url.endswith(".sha256"):
                    archive = destination.parent / archive_name
                    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
                    destination.write_text(
                        f"{digest}  {archive_name}\n", encoding="utf-8"
                    )
                else:
                    with zipfile.ZipFile(destination, "w") as archive:
                        archive.writestr(f"md-to-pdf-{tag}-{target}/md-to-pdf.exe", tag)

            with (
                mock.patch.object(server, "_plugin_data_dir", return_value=root),
                mock.patch.object(
                    server,
                    "_release_asset",
                    return_value=(target, "zip", "md-to-pdf.exe"),
                ),
                mock.patch.object(
                    server, "_download_file", side_effect=download
                ) as fetch,
            ):
                with mock.patch.object(server, "BINARY_VERSION", "0.5.0"):
                    first = Path(server._install_md_to_pdf())
                with mock.patch.object(server, "BINARY_VERSION", "0.5.1"):
                    second = Path(server._install_md_to_pdf())
                    self.assertEqual(str(second), server._install_md_to_pdf())
            self.assertNotEqual(first, second)
            self.assertEqual(first.name, f"md-to-pdf-v0.5.0-{target}.exe")
            self.assertEqual(first.read_text(), "v0.5.0")
            self.assertEqual(second.read_text(), "v0.5.1")
            self.assertEqual(fetch.call_count, 4)

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
                source = (
                    fixture_checksum if url.endswith(".sha256") else fixture_archive
                )
                shutil.copyfile(source, destination)

            plugin_data = root / "plugin-data"
            with (
                mock.patch.dict(
                    os.environ, {"PLUGIN_DATA": str(plugin_data)}, clear=False
                ),
                mock.patch.object(
                    server,
                    "_release_asset",
                    return_value=(target, "tar.gz", "md-to-pdf"),
                ),
                mock.patch.object(
                    server, "_download_file", side_effect=copy_download
                ) as download,
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


class BinarySelectionTests(unittest.TestCase):
    def test_managed_release_wins_over_stale_path_binary(self) -> None:
        with (
            mock.patch.dict(os.environ, {}, clear=True),
            mock.patch.object(
                server.shutil, "which", return_value="/old/md-to-pdf"
            ) as which,
            mock.patch.object(
                server, "_install_md_to_pdf", return_value="/managed/md-to-pdf"
            ),
        ):
            self.assertEqual(server._find_md_to_pdf(), "/managed/md-to-pdf")
            which.assert_not_called()

    def test_explicit_override_does_not_download(self) -> None:
        with (
            mock.patch.dict(
                os.environ, {"MD_TO_PDF_BIN": "/custom/md-to-pdf"}, clear=True
            ),
            mock.patch.object(server, "_install_md_to_pdf") as install,
        ):
            self.assertEqual(server._find_md_to_pdf(), "/custom/md-to-pdf")
            install.assert_not_called()

    def test_disabled_auto_install_uses_path_without_download(self) -> None:
        with (
            mock.patch.dict(os.environ, {"MD_TO_PDF_AUTO_INSTALL": "0"}, clear=True),
            mock.patch.object(server.shutil, "which", return_value="/path/md-to-pdf"),
            mock.patch.object(server, "_install_md_to_pdf") as install,
        ):
            self.assertEqual(server._find_md_to_pdf(), "/path/md-to-pdf")
            install.assert_not_called()


class ToolInvocationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.input = self.root / "input.md"
        self.input.write_text("# Test\n", encoding="utf-8")

    def test_reports_automatic_install_failure_as_a_tool_error(self) -> None:
        with mock.patch.object(
            server, "_find_md_to_pdf", side_effect=RuntimeError("download failed")
        ):
            result = server._run_tool({"input": str(self.input)})

        self.assertTrue(result["isError"])
        self.assertEqual(result["content"][0]["text"], "download failed")

    def test_builds_each_optional_argument_once(self) -> None:
        completed = subprocess.CompletedProcess([], 0, "Wrote output.pdf\n", "")
        output = self.root / "output.pdf"
        output.write_bytes(b"%PDF-1.4\nfixture")
        bundle = self.root / "mermaid.js"
        bundle.write_text("window.mermaid = {};", encoding="utf-8")
        arguments = {
            "input": str(self.input),
            "output": str(output),
            "allow_html": True,
            "allow_remote_assets": True,
            "browser": "/browser",
            "mermaid_js": str(bundle),
            "keep_html": True,
            "virtual_time_budget_ms": 20000,
        }
        with (
            mock.patch.object(server, "_find_md_to_pdf", return_value="/md-to-pdf"),
            mock.patch.object(server.subprocess, "run", return_value=completed) as run,
        ):
            result = server._run_tool(arguments)

        command = run.call_args.args[0]
        self.assertEqual(command.count("--allow-html"), 1)
        self.assertEqual(command.count("--allow-remote-assets"), 1)
        self.assertEqual(command.count("--browser"), 1)
        self.assertEqual(command.count("--mermaid-js"), 1)
        self.assertEqual(command.count("--keep-html"), 1)
        self.assertEqual(command[command.index("--virtual-time-budget") + 1], "20000")
        self.assertEqual(command[-2:], ["--", str(self.input)])
        self.assertEqual(result["content"][0]["text"], str(output.resolve()))

    def test_rejects_invalid_arguments_before_installation(self) -> None:
        cases = [
            {"allow_html": "false"},
            {"allow_local_files": 1},
            {"allow_remote_assets": 1},
            {"keep_html": "true"},
            {"virtual_time_budget_ms": True},
            {"virtual_time_budget_ms": 0},
            {"virtual_time_budget_ms": 60001},
            {"virtual_time_budget_ms": 1.5},
            {"mermaid_js": "missing.js"},
            {"css": "missing.css"},
            {
                "mermaid_url": "https://example.test/mermaid.js",
                "mermaid_js": str(self.input),
            },
            {"input": "missing.md"},
            {"input": 1},
            {"extra_flag": True},
        ]
        for invalid in cases:
            with (
                self.subTest(invalid=invalid),
                mock.patch.object(server, "_find_md_to_pdf") as find,
            ):
                result = server._run_tool({"input": str(self.input), **invalid})
                self.assertTrue(result["isError"])
                find.assert_not_called()

    def test_false_flags_stay_disabled(self) -> None:
        with (
            mock.patch.object(server, "_find_md_to_pdf", return_value="/md-to-pdf"),
            mock.patch.object(
                server.subprocess,
                "run",
                return_value=subprocess.CompletedProcess([], 1, "", "test"),
            ) as run,
        ):
            server._run_tool(
                {
                    "input": str(self.input),
                    "allow_html": False,
                    "allow_local_files": False,
                    "allow_remote_assets": False,
                    "keep_html": False,
                }
            )
        self.assertNotIn("--allow-html", run.call_args.args[0])
        self.assertNotIn("--allow-local-files", run.call_args.args[0])
        self.assertNotIn("--allow-remote-assets", run.call_args.args[0])
        self.assertNotIn("--keep-html", run.call_args.args[0])

    def test_success_without_pdf_is_reported_as_error(self) -> None:
        with (
            mock.patch.object(server, "_find_md_to_pdf", return_value="/md-to-pdf"),
            mock.patch.object(
                server.subprocess,
                "run",
                return_value=subprocess.CompletedProcess(
                    [], 0, "Wrote invented.pdf", ""
                ),
            ),
        ):
            result = server._run_tool({"input": str(self.input)})
        self.assertTrue(result["isError"])
        self.assertIn("verify generated PDF", result["content"][0]["text"])


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
            REPOSITORY_ROOT / "plugins/md-to-pdf-copilot/plugin.json",
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

    def test_copilot_marketplace_uses_legacy_runtime_package(self) -> None:
        """Keep VS Code off its broken Agent Plugins 1.0 MCP runtime path."""
        marketplace = json.loads(
            (REPOSITORY_ROOT / ".github/plugin/marketplace.json").read_text(
                encoding="utf-8"
            )
        )
        package = REPOSITORY_ROOT / "plugins/md-to-pdf-copilot"
        manifest = json.loads((package / "plugin.json").read_text(encoding="utf-8"))
        mcp = json.loads((package / ".mcp.json").read_text(encoding="utf-8"))
        configuration = mcp["mcpServers"]["md-to-pdf"]

        self.assertEqual(
            marketplace["plugins"][0]["source"], "./plugins/md-to-pdf-copilot"
        )
        self.assertNotIn("$schema", manifest)
        self.assertEqual(manifest["mcpServers"], "./.mcp.json")
        self.assertEqual(configuration["command"], "python3")
        self.assertEqual(configuration["args"], ["${PLUGIN_ROOT}/mcp_server/server.py"])

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
        self.assertEqual(
            response["result"]["serverInfo"]["version"], server.SERVER_VERSION
        )

    def test_marketplace_server_matches_canonical_server(self) -> None:
        canonical = REPOSITORY_ROOT / "mcp_server/server.py"
        for package in ("md-to-pdf", "md-to-pdf-copilot"):
            packaged = REPOSITORY_ROOT / "plugins" / package / "mcp_server/server.py"
            self.assertEqual(packaged.read_bytes(), canonical.read_bytes(), package)

    def test_marketplace_skill_matches_canonical_skill(self) -> None:
        relative = "skills/convert-to-pdf/SKILL.md"
        for package in ("md-to-pdf", "md-to-pdf-copilot"):
            self.assertEqual(
                (REPOSITORY_ROOT / relative).read_bytes(),
                (REPOSITORY_ROOT / "plugins" / package / relative).read_bytes(),
                package,
            )


if __name__ == "__main__":
    unittest.main()
