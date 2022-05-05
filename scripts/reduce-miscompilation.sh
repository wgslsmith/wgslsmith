#!/usr/bin/env bash

TMPDIR="${TMPDIR:-/tmp}"

# Recondition the shader - if this fails then the shader syntax is probably broken
reconditioned=$(cat "$WGSLREDUCE_SHADER_NAME" | "$WGSLREDUCE_BIN_PATH/reconditioner")
if [ $? -ne 0 ]; then
    exit 1
fi

# Validate with naga first since it tends to reject more often
if ! echo "$reconditioned" | "$WGSLREDUCE_BIN_PATH/preprocessor" | naga --stdin-file-path $WGSLREDUCE_SHADER_NAME; then
    exit 1
fi

# Also validate with tint
# TODO: Would be nice to avoid writing it to a file here - might need to make a simple custom cli for tint
echo "$reconditioned" > "$TMPDIR/wgslreduce_reconditioned.wgsl"
if ! tint --validate "$TMPDIR/wgslreduce_reconditioned.wgsl"; then
    exit 1
fi

# Finally execute the shader
# This will return 1 for a buffer mismatch mismatch - other errors should be ignored as uninteresting
echo "$reconditioned" | "$WGSLREDUCE_BIN_PATH/harness-client" "$WGSLREDUCE_SERVER" "$WGSLREDUCE_METADATA_PATH"
if [ $? -ne 1 ]; then
    exit 1
fi
