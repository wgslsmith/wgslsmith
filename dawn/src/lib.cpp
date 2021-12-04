#include <iostream>

#include <dawn/dawn_proc.h>
#include <dawn/webgpu_cpp.h>
#include <dawn_native/DawnNative.h>

WGPUDevice init()
{
    dawnProcSetProcs(&dawn_native::GetProcs());

    auto instance = std::make_unique<dawn_native::Instance>();

    // This makes things slow
    // instance->EnableBackendValidation(true);
    // instance->SetBackendValidationLevel(dawn_native::BackendValidationLevel::Full);

    instance->DiscoverDefaultAdapters();
    auto adapters = instance->GetAdapters();

    dawn_native::Adapter *selectedAdapter = nullptr;
    for (auto &adapter : adapters)
    {
        wgpu::AdapterProperties properties;
        adapter.GetProperties(&properties);
        if (properties.backendType == wgpu::BackendType::Vulkan)
        {
            selectedAdapter = &adapter;
        }
    }

    return selectedAdapter->CreateDevice();
}
