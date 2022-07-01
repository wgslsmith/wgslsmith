#pragma once

#include <memory>
#include <string>

bool validate_shader(const char* source);

std::unique_ptr<std::string> compile_shader_to_hlsl(const char* source);

std::unique_ptr<std::string> compile_shader_to_msl(const char* source);
