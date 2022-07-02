# Generator

The program generator is able to randomly generate WGSL programs using a range of language features.

```sh
# Generate a shader
$ wgslsmith gen
# Show help text
$ wgslsmith gen --help
```

Note that programs produced by the generator may not always compile (despite being syntactically valid and well-typed). This is because some WGSL compilers implement additional validation such as rejecting obvious infinite loops. wgslsmith uses a technique called reconditioning (see [here](../reconditioner/index.md)) to guarantee validity. You can recondition shaders by passing `--recondition` to the generator, or by invoking the reconditioner separately on the generated shader which allows more control over its behaviour.

The generator has various options to control the generation process. See the help text for a full list.

```admonish note
The options to control the sizes of functions and statement blocks are currently a rough approximation due to how the generator works. This may be fixed in future.
```

Pointers are currently supported as an opt-in feature (since the reconditioner may reject some shaders with invalid pointer operations). To enable them, use the `--enable-pointers` flag. If reconditioning (with `--recondition`), you can also pass `--skip-pointer-checks` to stop it from erroring if the program contains possible invalid pointer operations.
