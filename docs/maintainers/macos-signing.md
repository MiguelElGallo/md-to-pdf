---
icon: lucide/badge-check
---

# macOS signing setup

This guide shows maintainers how to configure Developer ID signing and Apple notarization for release workflows.

Do not paste certificate passwords, app-specific passwords, private keys, or `.p12` contents into issues, pull requests, chat, or documentation.

## Automated setup (recommended)

Run the local helper script instead of exporting and encoding by hand:

```sh
./scripts/setup-macos-signing-secrets.sh
```

The script:

- Exports your Developer ID identity from the login keychain into a temporary, randomly password-protected `.p12`.
- Base64-encodes the `.p12` and pipes it directly into the `APPLE_CERTIFICATE_P12_BASE64` secret.
- Stores the random export password as `APPLE_CERTIFICATE_PASSWORD`.
- Sets `APPLE_DEVELOPER_IDENTITY` and `APPLE_TEAM_ID`.
- Prompts locally (hidden input) for the Apple ID email and app-specific password.
- Deletes all temporary files on exit.

Secret values are never printed, copied to the clipboard, or written to shell history. The manual steps below are kept as a fallback.

## Required Apple account setup

You need:

- Apple Developer Program membership.
- A `Developer ID Application` certificate.
- The certificate and private key exported from Keychain Access as a password-protected `.p12`.
- An Apple app-specific password for `notarytool`.
- Your Apple Developer Team ID.

The expected local signing identity looks like this:

```text
Developer ID Application: Miguel Peredo (4QLP52E475)
```

Verify locally:

```sh
security find-identity -v -p codesigning | grep "Developer ID Application"
```

## Export the certificate

In Keychain Access:

1. Open the `login` keychain.
2. Select `My Certificates`.
3. Find `Developer ID Application: Miguel Peredo (4QLP52E475)`.
4. Expand it and confirm a private key is present.
5. Export the certificate and private key as a `.p12` file.
6. Use a strong export password.

Encode the `.p12` for GitHub Actions:

```sh
base64 -i DeveloperIDApplication.p12 | pbcopy
```

## Configure GitHub secrets

Add these repository secrets in GitHub:

| Secret | Value |
| --- | --- |
| `APPLE_CERTIFICATE_P12_BASE64` | Base64 output of the exported `.p12`. |
| `APPLE_CERTIFICATE_PASSWORD` | Password used when exporting the `.p12`. |
| `APPLE_DEVELOPER_IDENTITY` | `Developer ID Application: Miguel Peredo (4QLP52E475)` |
| `APPLE_ID` | Apple ID email used for notarization. |
| `APPLE_TEAM_ID` | Apple Developer Team ID, for example `4QLP52E475`. |
| `APPLE_APP_SPECIFIC_PASSWORD` | App-specific password created at `appleid.apple.com`. |

The release workflow treats macOS signing as enabled only when all six secrets exist.

## Verify with a dry run

After adding secrets, run the `Release` workflow manually with:

```text
tag=dry-run
```

The dry run should:

- Build macOS `.zip` artifacts.
- Import the Developer ID certificate into a temporary keychain.
- Sign macOS binaries with hardened runtime and a secure timestamp.
- Submit macOS zip archives to Apple's notary service.
- Verify the extracted signed binary with `codesign` and `spctl`.
- Produce checksums and artifact attestations.

Do not publish a signed macOS release until the dry run passes.

## Unsigned releases

Tag pushes are blocked when Apple signing secrets are missing. If maintainers intentionally need to publish unsigned macOS artifacts, manually dispatch the `Release` workflow with:

```text
allow_unsigned_macos=true
```

Only use that option for an explicitly unsigned release, and make sure the release notes say the macOS artifacts are unsigned and not notarized.
