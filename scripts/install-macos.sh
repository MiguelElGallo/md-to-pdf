#!/usr/bin/env sh
set -eu

repo="MiguelElGallo/md-to-pdf"
bin_name="md-to-pdf"
install_dir="/usr/local/bin"

arch="$(uname -m)"
case "$arch" in
  arm64)
    target="aarch64-apple-darwin"
    ;;
  x86_64)
    target="x86_64-apple-darwin"
    ;;
  *)
    echo "unsupported macOS architecture: $arch" >&2
    exit 1
    ;;
esac

version="${MD_TO_PDF_VERSION:-}"
if [ -z "$version" ]; then
  latest_url="https://github.com/$repo/releases/latest"
  release_url="$(curl -fsSL -o /dev/null -w '%{url_effective}' "$latest_url")"
  case "$release_url" in
    "https://github.com/$repo/releases/tag/"*)
      version="${release_url##*/}"
      ;;
    *)
      echo "failed to determine latest release version from $release_url" >&2
      exit 1
      ;;
  esac
fi

archive="$bin_name-$version-$target.zip"
checksum="$bin_name-$version-$target.sha256"
base_url="https://github.com/$repo/releases/download/$version"
workdir="$(mktemp -d)"

cleanup() {
  rm -rf "$workdir"
}
trap cleanup EXIT INT TERM

cd "$workdir"

echo "Downloading $archive"
curl -fsSLO "$base_url/$archive"
curl -fsSLO "$base_url/$checksum"

echo "Verifying checksum"
shasum -a 256 -c "$checksum"

echo "Extracting"
unzip -q "$archive"

binary="$workdir/$bin_name-$version-$target/$bin_name"
if [ ! -f "$binary" ]; then
  echo "expected binary not found: $binary" >&2
  exit 1
fi

if [ ! -d "$install_dir" ]; then
  echo "Creating $install_dir"
  sudo mkdir -p "$install_dir"
fi

echo "Installing to $install_dir/$bin_name"
sudo install "$binary" "$install_dir/$bin_name"

# The archive is notarized, but the downloaded file may still carry quarantine.
# Remove it from the installed binary after checksum verification.
xattr -dr com.apple.quarantine "$install_dir/$bin_name" 2>/dev/null || true

"$install_dir/$bin_name" --version

echo "Installed $bin_name $version"
