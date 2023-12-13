#include <tint/tint.h>

#include "src/tint/lang/wgsl/helpers/flatten_bindings.h"
#include "lib.h"

bool validate_shader(const char* source) {
    auto source_file = std::make_unique<tint::Source::File>("[memory]", source);
    auto program = std::make_unique<tint::Program>(tint::wgsl::reader::Parse(source_file.get()));
    return program->IsValid();
}

std::unique_ptr<std::string> compile_shader_to_hlsl(const char* source) {
    auto source_file = std::make_unique<tint::Source::File>("[memory]", source);
    auto program = std::make_unique<tint::Program>(tint::wgsl::reader::Parse(source_file.get()));

    if (!program->IsValid()) {
        return nullptr;
    }

    tint::ast::transform::Manager transform_manager;
    tint::ast::transform::DataMap transform_inputs;
    tint::ast::transform::DataMap transform_outputs;

    transform_inputs.Add<tint::ast::transform::Renamer::Config>(
        tint::ast::transform::Renamer::Target::kHlslKeywords,
        /* preserve_unicode */ false
    );
    transform_manager.Add<tint::ast::transform::Renamer>();

    auto transformed = transform_manager.Run(*(program.get()),
                                            std::move(transform_inputs),
                                            transform_outputs);
    
    if (!transformed.IsValid()) {
        return nullptr;
    }

    *program = std::move(transformed);

    tint::hlsl::writer::Options gen_options = {};
    auto result = tint::hlsl::writer::Generate(*(program.get()), gen_options);
    if (!result) {
        return nullptr;
    }

    return std::make_unique<std::string>(std::move(result->hlsl));
}

std::unique_ptr<std::string> compile_shader_to_msl(const char* source) {
    auto source_file = std::make_unique<tint::Source::File>("[memory]", source);
    auto program = std::make_unique<tint::Program>(tint::wgsl::reader::Parse(source_file.get()));

    if (!program->IsValid()) {
        return nullptr;
    }

    tint::ast::transform::Manager transform_manager;
    tint::ast::transform::DataMap transform_inputs;
    tint::ast::transform::DataMap transform_outputs;

    transform_inputs.Add<tint::ast::transform::Renamer::Config>(
        tint::ast::transform::Renamer::Target::kMslKeywords,
        /* preserve_unicode */ false
    );
    transform_manager.Add<tint::ast::transform::Renamer>();

    //auto transformed = transform_manager.Run(program.get(), std::move(transform_inputs));
    auto transformed = transform_manager.Run(*(program.get()),
                                        std::move(transform_inputs),
                                        transform_outputs);

    if (!transformed.IsValid()) {
        return nullptr;
    }

    *program = std::move(transformed);

    const tint::Program* input_program = program.get();
    auto flattened = tint::wgsl::FlattenBindings(*(program.get()));
    if (flattened) {
        input_program = &*flattened;
    }

    tint::msl::writer::Options gen_options;
    auto result = tint::msl::writer::Generate(*(input_program), gen_options);
    if (!result) {
        return nullptr;
    }

    return std::make_unique<std::string>(std::move(result->msl));
}
