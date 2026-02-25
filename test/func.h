/**
 * @file func.h
 * @brief Public API for the test_lib shared library.
 */

#ifndef FUNC_H
#define FUNC_H

/**
 * @brief Prints "Hello, world!" to stdout.
 */
void hello(void);

/**
 * @brief Returns the sum of two integers.
 *
 * @param a First operand.
 * @param b Second operand.
 * @return The sum of a and b.
 */
int add(int a, int b);

#endif /* FUNC_H */
