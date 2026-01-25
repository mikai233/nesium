#!/usr/bin/env bash
# apps/nesium_flutter/tool/zip_shaders.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SHADER_DIR="$(cd "${SCRIPT_DIR}/../assets/shaders" && pwd)"
ZIP_FILE="$(cd "${SCRIPT_DIR}/../assets" && pwd)/shaders.zip"

EXCLUDE_LIST=("bezel" "handheld" "stereoscopic-3d" "deinterlacing" "motion-interpolation" "test" "test-patterns")

echo "Zipping shaders from ${SHADER_DIR} to ${ZIP_FILE}..."

# Build exclusion pattern for find
EXCLUDE_ARGS=()
for item in "${EXCLUDE_LIST[@]}"; do
    EXCLUDE_ARGS+=("-o" "-path" "*/${item}/*" "-o" "-name" "${item}")
done

# Temporary list file
LIST_FILE=$(mktemp)

pushd "${SHADER_DIR}" > /dev/null
# Find all files, excluding .git and categories
find . -type f \
    ! -path "*/.git/*" \
    ! \( "${EXCLUDE_ARGS[@]:1}" \) \
    > "${LIST_FILE}"

# Create zip
zip -@ "${ZIP_FILE}" < "${LIST_FILE}"
popd > /dev/null

rm "${LIST_FILE}"

echo "Done! File size: $(du -h "${ZIP_FILE}" | cut -f1)"
