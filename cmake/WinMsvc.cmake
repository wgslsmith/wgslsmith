# Cross toolchain configuration for using clang-cl on non-Windows hosts to
# target MSVC.
#
# Usage:
# cmake -G Ninja
#    -DCMAKE_TOOLCHAIN_FILE=/path/to/this/file
#    -DLLVM_NATIVE_TOOLCHAIN=/path/to/llvm/installation
#    -DXWIN_CACHE=/path/to/xwin-cache
#
# LLVM_NATIVE_TOOLCHAIN:
#   *Absolute path* to a folder containing the toolchain which will be used to
#   build.  At a minimum, this folder should have a bin directory with a
#   copy of clang-cl, clang, clang++, and lld-link, as well as a lib directory
#   containing clang's system resource directory.
#
# XWIN_CACHE:
#   *Absolute path* to the xwin cache.
#   Use xwin (https://github.com/Jake-Shadle/xwin) to download the MSVC and
#   Windows SDK headers and libraries.
#   Alternatively, you can setup a similar directory structure by symlinking to
#   an actual VS installation from WSL on Windows.

include_guard(GLOBAL)

# When configuring CMake with a toolchain file against a top-level CMakeLists.txt,
# it will actually run CMake many times, once for each small test program used to
# determine what features a compiler supports.  Unfortunately, none of these
# invocations share a CMakeCache.txt with the top-level invocation, meaning they
# won't see the value of any arguments the user passed via -D.  Since these are
# necessary to properly configure MSVC in both the top-level configuration as well as
# all feature-test invocations, we set environment variables with the values so that
# these environments get inherited by child invocations. We can switch to
# CMAKE_TRY_COMPILE_PLATFORM_VARIABLES once our minimum supported CMake version
# is 3.6 or greater.
function(init_user_prop prop)
  if(${prop})
    set(ENV{_${prop}} "${${prop}}")
  else()
    set(${prop} "$ENV{_${prop}}" PARENT_SCOPE)
  endif()
endfunction()

# This sets up a case-insensitive vfs overlay when running on a case-sensitive filesystem,
# since Windows includes can be really inconsistent with casing.
function(generate_winsdk_vfs_overlay winsdk_include_dir output_path)
  set(include_dirs)
  file(GLOB_RECURSE entries LIST_DIRECTORIES true "${winsdk_include_dir}/*")
  foreach(entry ${entries})
    if(IS_DIRECTORY "${entry}")
      list(APPEND include_dirs "${entry}")
    endif()
  endforeach()

  file(WRITE "${output_path}"  "version: 0\n")
  file(APPEND "${output_path}" "case-sensitive: false\n")
  file(APPEND "${output_path}" "roots:\n")

  foreach(dir ${include_dirs})
    file(GLOB headers RELATIVE "${dir}" "${dir}/*.h")
    if(NOT headers)
      continue()
    endif()

    file(APPEND "${output_path}" "  - name: \"${dir}\"\n")
    file(APPEND "${output_path}" "    type: directory\n")
    file(APPEND "${output_path}" "    contents:\n")

    foreach(header ${headers})
      file(APPEND "${output_path}" "      - name: \"${header}\"\n")
      file(APPEND "${output_path}" "        type: file\n")
      file(APPEND "${output_path}" "        external-contents: \"${dir}/${header}\"\n")
    endforeach()
  endforeach()
endfunction()

set(CMAKE_SYSTEM_NAME Windows)
set(CMAKE_SYSTEM_VERSION 10.0)
set(CMAKE_SYSTEM_PROCESSOR AMD64)

init_user_prop(LLVM_NATIVE_TOOLCHAIN)
init_user_prop(XWIN_CACHE)

set(TRIPLE_ARCH "x86_64")
set(WINSDK_ARCH "x86_64")

set(MSVC_INCLUDE "${XWIN_CACHE}/splat/crt/include")
set(MSVC_LIB "${XWIN_CACHE}/splat/crt/lib")
set(WINSDK_INCLUDE "${XWIN_CACHE}/splat/sdk/include")
set(WINSDK_LIB "${XWIN_CACHE}/splat/sdk/lib")

# Do some sanity checking to make sure we can find a native toolchain and
# that the Windows SDK / MSVC STL directories look kosher.
if(NOT EXISTS "${LLVM_NATIVE_TOOLCHAIN}/bin/clang-cl" OR
   NOT EXISTS "${LLVM_NATIVE_TOOLCHAIN}/bin/lld-link")
  message(SEND_ERROR
          "LLVM_NATIVE_TOOLCHAIN folder '${LLVM_NATIVE_TOOLCHAIN}' does not "
          "point to a valid directory containing bin/clang-cl and bin/lld-link "
          "binaries")
endif()

if(NOT EXISTS "${MSVC_INCLUDE}" OR
   NOT EXISTS "${MSVC_LIB}")
  message(SEND_ERROR
          "MSVC system headers and libraries not found in xwin cache")
endif()

if(NOT EXISTS "${WINSDK_INCLUDE}" OR
   NOT EXISTS "${WINSDK_LIB}")
  message(SEND_ERROR
          "Windows SDK headers and libraries not found in xwin cache")
endif()

if(NOT EXISTS "${WINSDK_INCLUDE}/um/Windows.h")
  message(SEND_ERROR "Cannot find Windows.h")
endif()
if(NOT EXISTS "${WINSDK_INCLUDE}/um/WINDOWS.H")
  set(case_sensitive_filesystem TRUE)
endif()

set(CMAKE_C_COMPILER "${LLVM_NATIVE_TOOLCHAIN}/bin/clang-cl" CACHE FILEPATH "")
set(CMAKE_CXX_COMPILER "${LLVM_NATIVE_TOOLCHAIN}/bin/clang-cl" CACHE FILEPATH "")
set(CMAKE_LINKER "${LLVM_NATIVE_TOOLCHAIN}/bin/lld-link" CACHE FILEPATH "")
set(CMAKE_AR "${LLVM_NATIVE_TOOLCHAIN}/bin/llvm-lib" CACHE FILEPATH "")

# Even though we're cross-compiling, we need some native tools (e.g. llvm-tblgen), and those
# native tools have to be built before we can start doing the cross-build.  LLVM supports
# a CROSS_TOOLCHAIN_FLAGS_NATIVE argument which consists of a list of flags to pass to CMake
# when configuring the NATIVE portion of the cross-build.  By default we construct this so
# that it points to the tools in the same location as the native clang-cl that we're using.
list(APPEND _CTF_NATIVE_DEFAULT "-DCMAKE_ASM_COMPILER=${LLVM_NATIVE_TOOLCHAIN}/bin/clang")
list(APPEND _CTF_NATIVE_DEFAULT "-DCMAKE_C_COMPILER=${LLVM_NATIVE_TOOLCHAIN}/bin/clang")
list(APPEND _CTF_NATIVE_DEFAULT "-DCMAKE_CXX_COMPILER=${LLVM_NATIVE_TOOLCHAIN}/bin/clang++")

# These flags are used during build time. So if CFLAGS/CXXFLAGS/LDFLAGS is set
# for the target, makes sure these are unset during build time.
set(CROSS_TOOLCHAIN_FLAGS_NATIVE "${_CTF_NATIVE_DEFAULT}" CACHE STRING "")

set(COMPILE_FLAGS
    -D_CRT_SECURE_NO_WARNINGS
    --target=${TRIPLE_ARCH}-windows-msvc
    -fms-compatibility-version=19.20
    -imsvc "${MSVC_INCLUDE}"
    -imsvc "${WINSDK_INCLUDE}/ucrt"
    -imsvc "${WINSDK_INCLUDE}/shared"
    -imsvc "${WINSDK_INCLUDE}/um"
    -imsvc "${WINSDK_INCLUDE}/winrt")

if(case_sensitive_filesystem)
  # Ensure all sub-configures use the top-level VFS overlay instead of generating their own.
  init_user_prop(winsdk_vfs_overlay_path)
  if(NOT winsdk_vfs_overlay_path)
    set(winsdk_vfs_overlay_path "${CMAKE_BINARY_DIR}/winsdk_vfs_overlay.yaml")
    generate_winsdk_vfs_overlay("${WINSDK_INCLUDE}" "${winsdk_vfs_overlay_path}")
    init_user_prop(winsdk_vfs_overlay_path)
  endif()
  list(APPEND COMPILE_FLAGS
       -Xclang -ivfsoverlay -Xclang "${winsdk_vfs_overlay_path}")
endif()

string(REPLACE ";" " " COMPILE_FLAGS "${COMPILE_FLAGS}")
string(APPEND CMAKE_C_FLAGS_INIT " ${COMPILE_FLAGS}")
string(APPEND CMAKE_CXX_FLAGS_INIT " ${COMPILE_FLAGS}")

set(LINK_FLAGS
    # Prevent CMake from attempting to invoke mt.exe. It only recognizes the slashed form and not the dashed form.
    /manifest:no

    -libpath:"${MSVC_LIB}/${WINSDK_ARCH}"
    -libpath:"${WINSDK_LIB}/ucrt/${WINSDK_ARCH}"
    -libpath:"${WINSDK_LIB}/um/${WINSDK_ARCH}")

string(REPLACE ";" " " LINK_FLAGS "${LINK_FLAGS}")
string(APPEND CMAKE_EXE_LINKER_FLAGS_INIT " ${LINK_FLAGS}")
string(APPEND CMAKE_MODULE_LINKER_FLAGS_INIT " ${LINK_FLAGS}")
string(APPEND CMAKE_SHARED_LINKER_FLAGS_INIT " ${LINK_FLAGS}")

# CMake populates these with a bunch of unnecessary libraries, which requires
# extra case-correcting symlinks and what not. Instead, let projects explicitly
# control which libraries they require.
set(CMAKE_C_STANDARD_LIBRARIES "" CACHE STRING "" FORCE)
set(CMAKE_CXX_STANDARD_LIBRARIES "" CACHE STRING "" FORCE)
