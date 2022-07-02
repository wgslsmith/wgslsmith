# Reconditioner

wgslsmith uses a simple technique called reconditioning to remove certain unwanted behaviour from programs (such as out-of-bounds array accesses and infinite loops). This involves a tool called a reconditioner, which validates and transforms shaders to add safety checks around potentially dangerous operations. The advantage of using a separate tool rather than implementing this in the generator is that it enables using off-the-shelf program reduction tools such as C-Reduce and Perses.

```sh
# Recondition a shader
$ wgslsmith recondition path/to/shader.wgsl
```

The reconditioner can be used to guarantee loop termination, which is important for making sure that programs can be compiled as some compilers reject obvious infinite loops. If you only want to enforce loop terminate without any other runtime checks, pass `--enable loop-limiters` to the reconditioner.
