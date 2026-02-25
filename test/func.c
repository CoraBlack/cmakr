/**
 * @file func.c
 * @brief Example C shared library for cmakr.
 *
 * Provides simple functions exported as a shared library,
 * demonstrating Rust FFI interop via cmakr.
 */

#include "func.h"
#include <stdio.h>

void hello(void) {
    printf("Hello, world!\n");
}

int add(int a, int b) {
    return a + b;
}
