#!/usr/bin/env bash

kill -s USR1 "$WGSLREDUCE_PID"

args=(
    "$WGSLREDUCE_KIND"
    "$1"
    "$WGSLREDUCE_METADATA_PATH"
)

if [[ -n "${WGSLREDUCE_SERVER-}" ]]; then
    args+=("--server" "$WGSLREDUCE_SERVER")
fi

if [[ "$WGSLREDUCE_KIND" == "crash" ]]; then
    args+=("--regex" "$WGSLREDUCE_REGEX")

    if [[ -n "${WGSLREDUCE_CONFIG-}" ]]; then
        args+=("--config" "$WGSLREDUCE_CONFIG")
    else
        args+=(
            "--compiler" "$WGSLREDUCE_COMPILER"
            "--backend" "$WGSLREDUCE_BACKEND"
        )
    fi

    if [[ ! -n "${WGSLREDUCE_RECONDITION}" ]]; then
        args+=("--no-recondition")
    fi
fi

[WGSLSMITH] test -q "${args[@]}" >/dev/null 2>&1
