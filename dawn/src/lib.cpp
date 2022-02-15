#include <iostream>
#include <memory>

#include <dawn/dawn_proc.h>
#include <dawn/webgpu_cpp.h>
#include <dawn/native/DawnNative.h>

extern "C" dawn_native::Instance* new_instance()
{
    // Initialize WebGPU proc table
    dawnProcSetProcs(&dawn_native::GetProcs());

    auto instance = new dawn_native::Instance;

    // This makes things slow
    // instance->EnableBackendValidation(true);
    // instance->SetBackendValidationLevel(dawn_native::BackendValidationLevel::Full);

    instance->DiscoverDefaultAdapters();

    return instance;
}

extern "C" void delete_instance(dawn_native::Instance* instance) {
    delete instance;
}

extern "C" WGPUDevice create_device(const dawn_native::Instance* instance)
{
    auto adapters = instance->GetAdapters();

    dawn_native::Adapter *selectedAdapter = nullptr;
    for (auto &adapter : adapters)
    {
        wgpu::AdapterProperties properties;
        adapter.GetProperties(&properties);
        if (properties.backendType == wgpu::BackendType::D3D12)
        {
            selectedAdapter = &adapter;
        }
    }

    WGPUDeviceDescriptor descriptor = {};
    return selectedAdapter->CreateDevice(&descriptor);
}
