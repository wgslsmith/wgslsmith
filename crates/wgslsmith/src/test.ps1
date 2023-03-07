#!powershell.exe

$test_args = @("$env:WGSLREDUCE_KIND", "$env:WGSLREDUCE_SHADER_NAME", "$env:WGSLREDUCE_METADATA_PATH")

if (Test-Path env:WGSLREDUCE_SERVER) {
    $test_args += @("--server", "$env:WGSLREDUCE_SERVER")
}

if ("$env:WGSLREDUCE_KIND" -eq "crash") {
    $test_args += @("--regex", "$env:WGSLREDUCE_REGEX")

    if (Test-Path env:WGSLREDUCE_CONFIG) {
        $test_args += @("--config", "$env:WGSLREDUCE_CONFIG")
    }
    else {
        $test_args += @("--compiler", "$env:WGSLREDUCE_COMPILER", "--backend", "$env:WGSLREDUCE_BACKEND")
    }

    if (! (Test-Path env:WGSLREDUCE_RECONDITION)) {
        $test_args += @("--no-recondition")
    }
}

& [WGSLSMITH] test ($test_args | ForEach-Object { "`"$_`"" })

exit $LASTEXITCODE
