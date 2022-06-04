#pragma once

#include <memory>

extern "C" bool validate_shader(const char* source);

extern "C" std::unique_ptr<std::string> compile_shader_to_hlsl(const char* source);
