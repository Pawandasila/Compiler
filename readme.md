# Custom Language Examples

This document provides examples of valid code for your custom compiler.

## Basic Syntax Examples

### 1. Variable Declaration and Simple Arithmetic
```
int x = 5;
int y = 10;
int sum = x + y;
sum;  // Outputs: 15
```

### 2. More Complex Arithmetic
```
int a = 5;
int b = 3;
int c = 2;
a * b + c;  // Outputs: 17
```

### 3. Multiple Calculations in One Program
```
int a = 5;
int b = 3;
int c = 2;
int result1 = a * b + c;  // 17
int result2 = a * (b + c);  // 25
result2;  // Final output will be 25
```

### 4. Integer Arithmetic
```
int x = 10;
int y = 3;
x / y;  // Integer division may truncate the result
```

### 4. What Your Compiler Doesn't Support

Your compiler doesn't support:
- C++ syntax like `#include`, `using namespace`, `main()`, etc.
- Standard library functions from other languages
- Class definitions or object-oriented features
- External imports or libraries

## Language Constraints

The language supported by your compiler is a very simple one with:
- Integer and float variable declarations
- Basic arithmetic operations
- A print function for output
- Statements must end with semicolons
- No complex control structures beyond if/else and while loops (if implemented)

## Debugging Tips

If you get errors:
1. Make sure each statement ends with a semicolon
2. Check that you're using only `int` or `float` for variable declarations
3. Verify that your expressions are valid arithmetic expressions
4. Don't use features from other languages like C++, Java, etc.
