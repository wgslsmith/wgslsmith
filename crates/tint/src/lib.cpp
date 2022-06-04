#include <tint/tint.h>

#include "lib.h"

extern "C" bool validate_shader(const char* source) {
    auto source_file = std::make_unique<tint::Source::File>("[memory]", source);
    auto program = std::make_unique<tint::Program>(tint::reader::wgsl::Parse(source_file.get()));
    return program->IsValid();
}

extern "C" std::unique_ptr<std::string> compile_shader_to_hlsl(const char* source) {
    auto source_file = std::make_unique<tint::Source::File>("[memory]", source);
    auto program = std::make_unique<tint::Program>(tint::reader::wgsl::Parse(source_file.get()));

    if (!program->IsValid()) {
        return nullptr;
    }

    tint::transform::Manager transform_manager;
    tint::transform::DataMap transform_inputs;

    transform_inputs.Add<tint::transform::Renamer::Config>(
        tint::transform::Renamer::Target::kHlslKeywords,
        /* preserve_unicode */ false
    );
    transform_manager.Add<tint::transform::Renamer>();

    auto transformed = transform_manager.Run(program.get(), std::move(transform_inputs));
    if (!transformed.program.IsValid()) {
        return nullptr;
    }

    *program = std::move(transformed.program);

    tint::writer::hlsl::Options gen_options = {};
    auto result = tint::writer::hlsl::Generate(program.get(), gen_options);
    if (!result.success) {
        return nullptr;
    }

    return std::make_unique<std::string>(std::move(result.hlsl));
}
