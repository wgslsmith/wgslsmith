#include <tint/tint.h>

extern "C" bool validate_shader(const char* source) {
    auto source_file = std::make_unique<tint::Source::File>("[memory]", source);
    auto program = std::make_unique<tint::Program>(tint::reader::wgsl::Parse(source_file.get()));
    return program->IsValid();
}
