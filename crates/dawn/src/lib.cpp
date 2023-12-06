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

    instance->DiscoverDefaultAdapters();

    return instance;
}

extern "C" void delete_instance(dawn::native::Instance* instance) {
    delete instance;
}

extern "C" void enumerate_adapters(
    const dawn::native::Instance* instance,
    void(*callback)(const WGPUAdapterProperties*, void*),
    void* userdata
) {
    if (callback == nullptr) return;

    auto adapters = instance->GetAdapters();

    for (auto& adapter : adapters) {
        WGPUAdapterProperties properties = {};
        adapter.GetProperties(&properties);
        callback(&properties, userdata);
    }
}

extern "C" WGPUDevice create_device(
    const dawn::native::Instance* instance,
    WGPUBackendType backendType,
    uint32_t deviceID
) {
    auto adapters = instance->GetAdapters();

    dawn::native::Adapter *selectedAdapter = nullptr;
    for (auto& adapter : adapters) {
        WGPUAdapterProperties properties = {};
        adapter.GetProperties(&properties);
        if (properties.backendType == backendType && properties.deviceID == deviceID) {
            selectedAdapter = &adapter;
            break;
        }
    }

    if (!selectedAdapter) return nullptr;

    WGPUDeviceDescriptor descriptor = {};
    return selectedAdapter->CreateDevice(&descriptor);
}
