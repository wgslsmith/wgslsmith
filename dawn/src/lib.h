#include <memory>

#include <dawn/webgpu.h>
#include <dawn_native/DawnNative.h>

using DawnInstance = dawn_native::Instance;
using DawnAdapter = dawn_native::Adapter;

std::unique_ptr<DawnInstance> create_instance();
WGPUDevice create_device(const DawnInstance &instance);
