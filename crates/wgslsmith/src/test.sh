#!/usr/bin/env bash

args=(
    "$WGSLREDUCE_KIND"
    "$WGSLREDUCE_SHADER_NAME"
    "$WGSLREDUCE_METADATA_PATH"
)

if [[ -n "${WGSLREDUCE_SERVER-}" ]]; then
    args+=("--server" "$WGSLREDUCE_SERVER")
fi

if [[ "$WGSLREDUCE_KIND" == "crash" ]]; then
    args+=(
        "--config" "$WGSLREDUCE_CONFIG"
        "--regex" "$WGSLREDUCE_REGEX"
    )

    if [[ ! -n "${WGSLREDUCE_RECONDITION}" ]]; then
        args+=("--no-recondition")
    fi
fi

[WGSLSMITH] test "${args[@]}"
