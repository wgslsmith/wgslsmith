#include <iostream>

#include <dawn/dawn_proc.h>
#include <dawn/webgpu_cpp.h>
#include <dawn_native/DawnNative.h>

std::unique_ptr<dawn_native::Instance> create_instance()
{
    // Initialize WebGPU proc table
    dawnProcSetProcs(&dawn_native::GetProcs());

    auto instance = std::make_unique<dawn_native::Instance>();

    // This makes things slow
    // instance->EnableBackendValidation(true);
    // instance->SetBackendValidationLevel(dawn_native::BackendValidationLevel::Full);

    instance->DiscoverDefaultAdapters();

    return instance;
}

WGPUDevice create_device(const dawn_native::Instance &instance)
{
    auto adapters = instance.GetAdapters();

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

    return selectedAdapter->CreateDevice();
}