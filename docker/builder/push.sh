#!/usr/bin/env bash
# OPTIONAL: retag and push the builder image to a registry you control.
# Publishing is a deliberate manual action — this is never run by CI.
# Usage: docker/builder/push.sh <registry-prefix>
#   e.g. docker/builder/push.sh registry.gitlab.com/pos9014819/rust-desk
# Requires that you have already run `docker login` yourself. This script never
# reads, stores, or logs credentials/tokens.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
IMAGE_NAME="enzu/rustdesk-builder"

if [ "$#" -ne 1 ] || [ -z "${1:-}" ]; then
    echo "Usage: $0 <registry-prefix>" >&2
    exit 1
fi
REGISTRY_PREFIX="${1%/}"

if ! command -v docker >/dev/null 2>&1; then
    echo "ERROR: docker is not installed or not on PATH." >&2
    exit 1
fi

VERSION="$(tr -d ' \t\r\n' < "${SCRIPT_DIR}/VERSION")"
SRC_TAG="${IMAGE_NAME}:${VERSION}"
DEST_TAG="${REGISTRY_PREFIX}/${IMAGE_NAME}:${VERSION}"

echo "Retagging ${SRC_TAG} -> ${DEST_TAG}"
docker tag "${SRC_TAG}" "${DEST_TAG}"
echo "Pushing ${DEST_TAG} (ensure you have run 'docker login' already)"
docker push "${DEST_TAG}"
echo "Done."
