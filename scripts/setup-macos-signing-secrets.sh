#!/usr/bin/env bash
#
# Local, one-time setup for macOS signing + notarization GitHub secrets.
#
# What this does, safely and without copy/paste of secret values:
#   1. Exports your Developer ID identity from the login keychain into a
#      temporary, password-protected .p12 (password is random, never shown).
#   2. Base64-encodes the .p12 and pipes it straight into a GitHub secret.
#   3. Prompts you locally (hidden input) for the Apple ID email and the
#      app-specific password, and sets them as GitHub secrets.
#   4. Sets the non-secret identity/team values.
#   5. Cleans up all temporary files.
#
# Secret values never touch the clipboard, chat, or command history output.
#
# Requirements: gh (authenticated), security, base64, openssl.
#
# Usage:
#   ./scripts/setup-macos-signing-secrets.sh
#
set -euo pipefail

REPO="MiguelElGallo/md-to-pdf"
IDENTITY_NAME="Developer ID Application: Miguel Peredo (4QLP52E475)"
TEAM_ID="4QLP52E475"
LOGIN_KEYCHAIN="$HOME/Library/Keychains/login.keychain-db"

echo "Setting up macOS signing secrets for $REPO"
echo

# --- Preflight -------------------------------------------------------------

for tool in gh security base64 openssl; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "error: required tool '$tool' not found" >&2
    exit 1
  fi
done

if ! gh auth status >/dev/null 2>&1; then
  echo "error: gh is not authenticated. Run 'gh auth login' first." >&2
  exit 1
fi

if ! security find-identity -v -p codesigning | grep -q "$IDENTITY_NAME"; then
  echo "error: signing identity not found in keychain:" >&2
  echo "       $IDENTITY_NAME" >&2
  exit 1
fi

# --- Temp workspace with guaranteed cleanup --------------------------------

workdir="$(mktemp -d)"
cleanup() {
  rm -rf "$workdir"
}
trap cleanup EXIT

p12_path="$workdir/developer-id.p12"

# Random export password. Generated locally, never displayed, used only to
# wrap the .p12 and stored in GitHub as APPLE_CERTIFICATE_PASSWORD.
p12_password="$(openssl rand -base64 24)"

echo "Exporting Developer ID identity to a temporary .p12 ..."
echo "macOS may prompt you to allow keychain access. Choose 'Always Allow' or 'Allow'."
security export \
  -k "$LOGIN_KEYCHAIN" \
  -t identities \
  -f pkcs12 \
  -P "$p12_password" \
  -o "$p12_path"

echo "Uploading APPLE_CERTIFICATE_P12_BASE64 ..."
base64 -i "$p12_path" | gh secret set APPLE_CERTIFICATE_P12_BASE64 --repo "$REPO"

echo "Uploading APPLE_CERTIFICATE_PASSWORD ..."
printf '%s' "$p12_password" | gh secret set APPLE_CERTIFICATE_PASSWORD --repo "$REPO"

echo "Uploading APPLE_DEVELOPER_IDENTITY ..."
gh secret set APPLE_DEVELOPER_IDENTITY --repo "$REPO" --body "$IDENTITY_NAME"

echo "Uploading APPLE_TEAM_ID ..."
gh secret set APPLE_TEAM_ID --repo "$REPO" --body "$TEAM_ID"

# --- Interactive, hidden prompts for Apple credentials ---------------------

echo
read -r -p "Apple ID email (for notarization): " apple_id
if [[ -z "$apple_id" ]]; then
  echo "error: Apple ID email cannot be empty" >&2
  exit 1
fi
printf '%s' "$apple_id" | gh secret set APPLE_ID --repo "$REPO"

echo
echo "Create an app-specific password at https://appleid.apple.com/account/manage"
read -r -s -p "App-specific password (input hidden): " app_password
echo
if [[ -z "$app_password" ]]; then
  echo "error: app-specific password cannot be empty" >&2
  exit 1
fi
printf '%s' "$app_password" | gh secret set APPLE_APP_SPECIFIC_PASSWORD --repo "$REPO"

# Clear sensitive shell variables.
unset p12_password app_password apple_id

echo
echo "Done. Configured secrets:"
gh secret list --repo "$REPO" | awk '{print $1}' | grep -E 'APPLE_' | sort

echo
echo "Next: run a signing/notarization dry run with"
echo "  gh workflow run Release --repo $REPO --ref main -f tag=dry-run"
