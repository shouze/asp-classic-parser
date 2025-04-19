#!/usr/bin/env bash
# ---------------------------------------------------------------------------
# asp‑classic‑parser installer — concise & readable
# ---------------------------------------------------------------------------
# Usage : ./install.sh [-v VERSION] [-d DIR] [--debug]
# ---------------------------------------------------------------------------

set -euo pipefail
IFS=$'\n\t'

# ─── Defaults ────────────────────────────────────────────────────────────────
REPO="shouze/asp-classic-parser"
API="https://api.github.com/repos/${REPO}"
VERSION="latest"
INSTALL_DIR="${HOME}/.local/bin"
DEBUG=0

# ─── Helpers ────────────────────────────────────────────────────────────────
log() { printf '\e[1m%s\e[0m\n' "$*"; }
die() { printf 'Error: %s\n' "$*" >&2; exit 1; }
has() { command -v "$1" >/dev/null 2>&1; }
json() { has jq || die "jq is required (e.g. brew install jq)"; jq -r "$@"; }

usage() {
  cat <<EOF
asp‑classic‑parser installer
Usage: $0 [options]
  -v, --version VER   Release tag (default: latest)
  -d, --dir DIR       Install dir (default: ~/.local/bin)
  --debug             Verbose output
  -h, --help          Show this help
EOF
  exit 0
}

# ─── Parse CLI ──────────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
  case $1 in
    -v|--version) VERSION="$2"; shift 2;;
    -d|--dir)     INSTALL_DIR="$2"; shift 2;;
    --debug)      DEBUG=1; shift;;
    -h|--help)    usage;;
    *)            usage;;
  esac
done

[[ ${DEBUG} -eq 1 ]] && set -x
mkdir -p "${INSTALL_DIR}"

# ─── Detect platform ────────────────────────────────────────────────────────
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)
case ${ARCH} in
  x86_64|amd64)  ARCH=x86_64 ;;
  i?86)          ARCH=i686   ;;
  arm64|aarch64) ARCH=aarch64;;
  *)             die "unsupported arch ${ARCH}";;
esac
case ${OS} in
  darwin)  PLATFORM="${ARCH}-apple-darwin"      ; EXT=tar.gz ; BIN=asp-classic-parser     ;;
  linux)   PLATFORM="${ARCH}-unknown-linux-gnu" ; EXT=tar.gz ; BIN=asp-classic-parser     ;;
  msys*|mingw*|cygwin*)
           OS=windows
           PLATFORM="${ARCH}-pc-windows-msvc"   ; EXT=zip    ; BIN=asp-classic-parser.exe ;;
  *)       die "unsupported OS ${OS}";;
esac
log "Detected platform: ${PLATFORM}"

# ─── Fetch release JSON ─────────────────────────────────────────────────────
[[ ${VERSION} != latest && ${VERSION} != v* ]] && VERSION="v${VERSION}"
if [[ ${VERSION} == latest ]]; then
  REL_JSON=$(curl -sSfL "${API}/releases/latest") || die "no releases found"
else
  REL_JSON=$(curl -sSfL "${API}/releases/tags/${VERSION}") || die "tag ${VERSION} not found"
fi
TAG=$(printf '%s' "${REL_JSON}" | json '.tag_name')
[[ -z ${TAG} || ${TAG} == null ]] && die "cannot determine tag name"
VER=${TAG#v}

# ─── Locate asset ───────────────────────────────────────────────────────────
ASSET="asp-classic-parser-${VER}-${PLATFORM}.${EXT}"
URL=$(printf '%s' "${REL_JSON}" | json --arg n "${ASSET}" '.assets[]? | select(.name==$n) | .browser_download_url')
[[ -z ${URL} || ${URL} == null ]] && die "asset ${ASSET} not found in ${TAG}"
log "Downloading ${ASSET} …"

TMP=$(mktemp -d)
trap 'rm -rf "${TMP}"' EXIT
curl -L --progress-bar "${URL}" -o "${TMP}/${ASSET}"
curl -sSL "${URL}.sha256" -o "${TMP}/${ASSET}.sha256" || true

# ─── Verify checksum ───────────────────────────────────────────────────────
if [[ -s "${TMP}/${ASSET}.sha256" ]]; then
  (cd "${TMP}" && shasum -a 256 -c "${ASSET}.sha256") || die "checksum verification failed"
else
  log "(checksum file missing, skipping verification)"
fi

# ─── Extract ───────────────────────────────────────────────────────────────
cd "${TMP}"
case ${EXT} in
  tar.gz) tar -xzf "${ASSET}";;
  zip)    unzip -q "${ASSET}";;
  *)      die "unknown archive type ${EXT}";;
esac
chmod +x "${BIN}"

# ─── Install ─────────────────────────────────────────────────────────────── ───────────────────────────────────────────────────────────────
cd "${TMP}"
case ${EXT} in
  tar.gz) tar -xzf "${ASSET}";;
  zip)    unzip -q "${ASSET}";;
  *)      die "unknown archive type ${EXT}";;
esac
chmod +x "${BIN}"

# ─── Install ───────────────────────────────────────────────────────────────
log "Installing to ${INSTALL_DIR}"
mv "${BIN}" "${INSTALL_DIR}/"

[[ ":${PATH}:" != *":${INSTALL_DIR}:"* ]] && log "Add ${INSTALL_DIR} to your PATH."

"${INSTALL_DIR}/${BIN}" --version || log "installed but version check failed"
log "asp‑classic‑parser ${TAG} installed successfully!"
