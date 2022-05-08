# Configurations

The harness will execute the input shader against one or more configurations, and compare the results of the output buffers for each configuration to detect possible miscompilation bugs.

A configuration is defined as the combination of a WebGPU implementation (such as dawn or wgpu) and a graphics adapter. The graphics adapter is identified by its backend type (D3D12, Metal, Vulkan) and a platform-specific integer identifier for the device.

Use the `list` subcommand to get a list of available configurations on your machine.

```sh
$ harness list
ID             | Adapter Name
---------------+------------------------------
wgpu:vk:9348   | NVIDIA GeForce RTX 3070
wgpu:dx12:9348 | NVIDIA GeForce RTX 3070
wgpu:dx12:140  | Microsoft Basic Render Driver
dawn:dx12:9348 | NVIDIA GeForce RTX 3070
dawn:dx12:140  | Microsoft Basic Render Driver
dawn:vk:9348   | NVIDIA GeForce RTX 3070
```

On my machine there are three adapters available, corresponding to hardware Vulkan and D3D12 implementations as well as a D3D12 software implementation. The configuration IDs consist of the WebGPU implementation, the backend type, and the PCI ID for the adapter.

By default, the harness will attempt to find the first available adapter for each combination of WebGPU implementation and backend type. Thus, all configurations above will be selected except for the D3D12 software adapter.

To specify configurations manually, you can pass them on the command line using the `-c` option.

```sh
$ harness run test.wgsl -c wgpu:dx12:140 -c dawn:dx12:140 -c dawn:vk:9348
executing wgpu:dx12:140
outputs:
  0: [2, 0, 0, 0]

executing dawn:dx12:140
outputs:
  0: [2, 0, 0, 0]

executing dawn:vk:9348
outputs:
  0: [2, 0, 0, 0]

ok
```
