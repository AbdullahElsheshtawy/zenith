cmake_minimum_required(VERSION 3.23)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/$<CONFIGURATION>")
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/$<CONFIGURATION>")

option(ZENITH_ASAN OFF "Enable address sanitizer")
option(ZENITH_CLANG_TIDY OFF "Enable clang-tidy")

if(ZENITH_ASAN)
    message("Enabled address sanitizer")
    add_compile_options(-fsanitize=address)
    add_link_options(-fsanitize=address)
endif()

find_program(CCACHE_PROGRAM ccache)
if(CCACHE_PROGRAM)
    message("Found ccache")
    set(CMAKE_CXX_COMPILER_LAUNCHER ccache)
endif()

if(ZENITH_CLANG_TIDY)
    find_program(CLANG_TIDY_PROGRAM clang-tidy)
    if(CLANG_TIDY_PROGRAM)
        message("Found clang-tidy")
        set(CMAKE_CXX_CLANG_TIDY ${CLANG_TIDY_PROGRAM} -p ${CMAKE_BINARY_DIR})
        message("Enabled clang-tidy")
    else()  
        message(FATAL_ERROR "clang-tidy not found but ZENITH_CLANG_TIDY is enabled")
    endif()
endif()

project(zenith)

add_subdirectory(third_party)
add_subdirectory(src)