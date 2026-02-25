# Example CMake project for cmakr
#
# This builds a minimal shared library (test_lib) that exports
# C functions callable from Rust via FFI. It serves as both a
# test fixture for the cmakr library and a usage example.

cmake_minimum_required(VERSION 3.20)

project(test_lib
    VERSION 0.1.0
    LANGUAGES C
    DESCRIPTION "Example shared library for cmakr"
)

# Build a shared library from func.c
add_library(${PROJECT_NAME} SHARED func.c)

# Ensure the public header is available to consumers
target_include_directories(${PROJECT_NAME} PUBLIC ${CMAKE_CURRENT_SOURCE_DIR})
