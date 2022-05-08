# Test case reduction

Test case reduction tools such as [c-reduce](https://embed.cs.utah.edu/creduce/) typically take an _interestingness_ test as input, which returns `0` for a useful test case or `1` if the test case should be discarded.

The harness can produce two types of errors:

- If the actual shader execution failed, this will manifest as a panic with exit code `101`.
- If the shader was successfully executed for all configurations but the outputs differ, the program will exit with code `1`.

Otherwise, the program exits normally with code `0`.

Normally when using this with a reduction tool to find miscompilations, you will want to discard the shader if the harness returns `0` or `101`, since execution failure means that the reduction process probably produced an invalid program. Only the exits with `1` are likely to be interesting.
