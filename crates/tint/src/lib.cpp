#include "src/tint/api/tint.h"
#include "src/tint/api/helpers/generate_bindings.h"

#include "src/tint/lang/wgsl/reader/reader.h"
#include "src/tint/lang/wgsl/reader/program_to_ir/program_to_ir.h"

#include "src/tint/lang/hlsl/writer/writer.h"
#include "src/tint/lang/msl/writer/writer.h"

#include "src/tint/lang/wgsl/inspector/inspector.h"

#include "lib.h"

// helper`
std::string get_entry_point_name(const tint::Program& program) {
    tint::inspector::Inspector inspector(program);
    auto entry_points = inspector.GetEntryPoints();
    if (entry_points.empty()) {
        return "";
    }
    return entry_points[0].name;
}

bool validate_shader(const char* source) {
    tint::wgsl::reader::Options options;
    auto source_file = std::make_unique<tint::Source::File>("[memory]", source);
    auto program = tint::wgsl::reader::Parse(source_file.get(), options);
    return program.IsValid();
}

std::unique_ptr<std::string> compile_shader_to_hlsl(const char* source) {
    tint::wgsl::reader::Options parser_options;
    auto source_file = std::make_unique<tint::Source::File>("[memory]", source);
    auto program = tint::wgsl::reader::Parse(source_file.get(), parser_options);

    if (!program.IsValid()) {
        return nullptr;
    }

    auto ir_result = tint::wgsl::reader::ProgramToLoweredIR(program);
    if (ir_result != tint::Success) {
        return nullptr;
    }
    auto& ir = ir_result.Get();

    tint::hlsl::writer::Options gen_options = {};

    std::string ep_name = get_entry_point_name(program);
    if(ep_name.empty()) return nullptr;

    gen_options.bindings = tint::GenerateBindings(ir, ep_name, false, false);
    gen_options.entry_point_name = ep_name;

    auto result = tint::hlsl::writer::Generate(ir, gen_options);
    if (result != tint::Success) {
        return nullptr;
    }

    return std::make_unique<std::string>(std::move(result.Get().hlsl));
}

std::unique_ptr<std::string> compile_shader_to_msl(const char* source) {
    tint::wgsl::reader::Options parser_options;
    auto source_file = std::make_unique<tint::Source::File>("[memory]", source);
    auto program = tint::wgsl::reader::Parse(source_file.get(), parser_options);

    if (!program.IsValid()) {
        return nullptr;
    }

    auto ir_result = tint::wgsl::reader::ProgramToLoweredIR(program);
    if (ir_result != tint::Success) {
        return nullptr;
    }
    auto& ir = ir_result.Get();

    tint::msl::writer::Options gen_options;

    std::string ep_name = get_entry_point_name(program);
    if(ep_name.empty()) return nullptr;

    gen_options.entry_point_name = ep_name;
    gen_options.bindings = tint::GenerateBindings(ir, ep_name, true, true);

    auto result = tint::msl::writer::Generate(ir, gen_options);
    if (result != tint::Success) {
        return nullptr;
    }

    return std::make_unique<std::string>(std::move(result.Get().msl));
}
