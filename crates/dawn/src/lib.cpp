#include <iostream>
#include <memory>

#include <dawn/dawn_proc.h>
#include <dawn/webgpu.h>
#include <dawn/webgpu_cpp.h>
#include <dawn/native/DawnNative.h>

extern "C" dawn::native::Instance* new_instance() {
    // Initialize WebGPU proc table
    dawnProcSetProcs(&dawn::native::GetProcs());

    auto instance = new dawn::native::Instance;

    // This makes things slow
    // instance->EnableBackendValidation(true);
    // instance->SetBackendValidationLevel(dawn::native::BackendValidationLevel::Full);

    WGPURequestAdapterOptions options = {};
    instance->EnumerateAdapters(&options); 

    return instance;
}

extern "C" void instance_process_events(dawn::native::Instance* instance) {
    if (instance) {
        wgpuInstanceProcessEvents(instance->Get());
    }
}

extern "C" void delete_instance(dawn::native::Instance* instance) {
    delete instance;
}

extern "C" void enumerate_adapters(
    const dawn::native::Instance* instance,
    void(*callback)(const WGPUAdapterInfo*, void*),
    void* userdata
) {
    if (callback == nullptr) return;

    WGPURequestAdapterOptions options = {};
    auto native_adapters = instance->EnumerateAdapters(&options);

    for (auto& native_adapter : native_adapters) {
        WGPUAdapter adapterHandle = native_adapter.Get();
        WGPUAdapterInfo info = {};
        info.nextInChain = nullptr;

        wgpuAdapterGetInfo(adapterHandle, &info);

        callback(&info, userdata);
    }
}

extern "C" WGPUDevice create_device(
    const dawn::native::Instance* instance,
    WGPUBackendType backendType,
    uint32_t deviceID
) {
    WGPURequestAdapterOptions options = {};
    auto native_adapters = instance->EnumerateAdapters(&options);

    for (auto& native_adapter : native_adapters) {
        WGPUAdapter adapter_handle = native_adapter.Get();

        WGPUAdapterInfo info = {};
        wgpuAdapterGetInfo(adapter_handle, &info);

        if (info.backendType == backendType && info.deviceID == deviceID) {
            WGPUDeviceDescriptor descriptor = {};
            return wgpuAdapterCreateDevice(adapter_handle, &descriptor);
        }
    }

    return nullptr;
}
