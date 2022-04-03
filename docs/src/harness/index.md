# Harness

The test harness is used to run shader programs within a WebGPU pipeline. The harness is flexible and can be used for shaders with different inputs and outputs by providing a JSON description of the shader's I/O interface.

The harness will execute the shaders against multiple WebGPU implementations/configurations and compare the outputs to detect mismatches.

Currently, the harness only supports running compute shaders.
