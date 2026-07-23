#!/usr/bin/env bash
# Build the ENZU Android builder image with the pinned tag from ./VERSION.
# Does NOT publish. Run from anywhere — paths are resolved relative to the repo root.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
IMAGE_NAME="enzu/rustdesk-builder"

if ! command -v docker >/dev/null 2>&1; then
    echo "ERROR: docker is not installed or not on PATH." >&2
    exit 1
fi

VERSION="$(tr -d ' \t\r\n' < "${SCRIPT_DIR}/VERSION")"
if [ -z "${VERSION}" ]; then
    echo "ERROR: docker/builder/VERSION is empty." >&2
    exit 1
fi

TAG="${IMAGE_NAME}:${VERSION}"
echo "Building image: ${TAG}"
echo "  Dockerfile : docker/builder/Dockerfile"
echo "  context    : ${REPO_ROOT}"

docker build \
    -f "${SCRIPT_DIR}/Dockerfile" \
    -t "${TAG}" \
    "${REPO_ROOT}"

echo "Done. Built ${TAG} (not published)."
