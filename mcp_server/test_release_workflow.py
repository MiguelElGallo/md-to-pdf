from __future__ import annotations

import os
import re
import subprocess
import tempfile
import textwrap
import unittest
from pathlib import Path

REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
WORKFLOW_PATH = REPOSITORY_ROOT / ".github/workflows/release.yml"


def _step(workflow: str, name: str) -> str:
    marker = f"      - name: {name}\n"
    start = workflow.index(marker)
    end = workflow.find("\n      - name: ", start + len(marker))
    return workflow[start:] if end == -1 else workflow[start:end]


def _job(workflow: str, name: str) -> str:
    marker = f"  {name}:\n"
    start = workflow.index(marker)
    match = re.search(
        r"^  [a-z][a-z0-9-]*:\n",
        workflow[start + len(marker) :],
        re.MULTILINE,
    )
    if match is None:
        return workflow[start:]
    return workflow[start : start + len(marker) + match.start()]


class ReleaseWorkflowSecurityTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.workflow = WORKFLOW_PATH.read_text(encoding="utf-8")

    def test_dispatch_input_is_data_before_secret_availability_check(self) -> None:
        planner = _step(self.workflow, "Determine release mode")
        signing = _step(self.workflow, "Check macOS signing availability")

        self.assertIn("INPUT_TAG: ${{ inputs.tag }}", planner)
        self.assertIn('tag="$INPUT_TAG"', planner)
        self.assertNotIn("secrets.", planner)
        self.assertNotIn("${{ inputs.", planner[planner.index("        run: |") :])
        self.assertLess(self.workflow.index(planner), self.workflow.index(signing))

    def test_planner_accepts_supported_release_modes(self) -> None:
        cases = {
            "dry-run": ("dry-run", "false"),
            "v1.2.3": ("1.2.3", "true"),
            "v1.2.3-rc.1": ("1.2.3-rc.1", "true"),
        }
        for tag, (version, publishing) in cases.items():
            with self.subTest(tag=tag):
                result, output = self._run_planner(tag)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertIn(f"version={version}\n", output)
                self.assertIn(f"publishing={publishing}\n", output)

    def test_planner_rejects_shell_payloads_without_executing_them(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            marker = Path(directory) / "executed"
            payloads = (
                f"$(touch {marker})dry-run",
                f"`touch {marker}`dry-run",
                "v1.2.3\nmalicious=true",
                'v1.2.3"; touch executed; #',
            )
            for tag in payloads:
                with self.subTest(tag=tag):
                    result, _ = self._run_planner(tag)
                    self.assertNotEqual(result.returncode, 0)
                    self.assertFalse(marker.exists())

    @staticmethod
    def _run_planner(tag: str) -> tuple[subprocess.CompletedProcess[str], str]:
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "github-output"
            env = {
                **os.environ,
                "EVENT_NAME": "workflow_dispatch",
                "INPUT_TAG": tag,
                "INPUT_ALLOW_UNSIGNED_MACOS": "false",
                "GITHUB_SHA": "a" * 40,
                "GITHUB_OUTPUT": str(output),
            }
            result = subprocess.run(
                ["bash", "-c", ReleaseWorkflowSecurityTests._planner_script()],
                cwd=REPOSITORY_ROOT,
                env=env,
                text=True,
                capture_output=True,
                check=False,
            )
            contents = output.read_text(encoding="utf-8") if output.exists() else ""
            return result, contents

    @classmethod
    def _planner_script(cls) -> str:
        planner = _step(cls.workflow, "Determine release mode")
        run_body = planner.split("        run: |\n", maxsplit=1)[1]
        return textwrap.dedent(run_body)

    def test_every_release_action_is_pinned_to_a_commit(self) -> None:
        action_refs = re.findall(
            r"^\s*uses:\s+[^\s@]+@([^\s#]+)", self.workflow, re.MULTILINE
        )

        self.assertTrue(action_refs)
        for action_ref in action_refs:
            self.assertRegex(action_ref, r"^[0-9a-f]{40}$")

    def test_signing_isolated_from_build_tooling(self) -> None:
        build = _job(self.workflow, "build")
        macos = _job(self.workflow, "package-macos")

        self.assertNotIn("secrets.APPLE_", build)
        self.assertNotIn("cargo build", macos)
        self.assertNotIn("rust-toolchain", macos)
        self.assertNotIn("rust-cache", self.workflow)
        self.assertNotIn('"$binary" --version', macos)
        self.assertNotIn('"$binary" --help', macos)
        self.assertIn("needs:\n      - plan\n      - build\n      - package-macos", _job(self.workflow, "attest"))

    def test_all_four_final_artifact_targets_are_present(self) -> None:
        for target in (
            "x86_64-unknown-linux-gnu",
            "x86_64-pc-windows-msvc",
            "x86_64-apple-darwin",
            "aarch64-apple-darwin",
        ):
            self.assertIn(target, self.workflow)


if __name__ == "__main__":
    unittest.main()
