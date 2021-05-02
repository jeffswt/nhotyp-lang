
# nhotyp-lang

Nhotyp is a conceptual language designed for ease of implementation during my tutoring in an introductive algorithmic course at Harbin Institute of Technology, Weihai. The current repository holds the latest definition for Nhotyp, but the specification itself was initially written in Chinese.

Nhotyp is an "modern" interpretative language imitating a few features in Python and Rust, and used prefix expressions for ease of parsing. It was so designed to make the assignment easier to complete, if one chose to think the problem through, as it required few string operations and would never require the construction of an AST just to function properly.

The said repository introduces a standard implementation which would work on correct implementations, and should report common runtime errors if it was not written properly.

## Usage

Build the compiler with Rust and execute your Nhotyp code with the compiled interpreter:

```
cargo build
cargo run your_code.nh
```

You may find some samples in the `samples/` folder.

## Specifications

### 1. Comments

By standard no line shall have code and comments mixed together (i.e. a single line could be either a statement or a comment but never both), it is however up to the interpreter to decide whether this requirement should be enforced.

All comments should start with the character `#`, and all subsequence characters MUST be ignored, hidden to the statement parser.

### 2. Data Types

To simplify the implementation, only 48-bit signed integers may appear as variables or constants. There will be no characters, strings, integers of other sizes or any other data type in any form. Implementers should take care of overflow cases, which would guarantee all values within the range [-2^47, 2^47-1]. Calculation between those integers are yet to be defined in the section *Operators*.

### 3. Variables

All variable names would consist only of ASCII lowercase letters (`a-z`) or underscores (`_`). Other characters should NEVER appear as part of a variable name. All variables or constants MUST NEVER be longer than 63 characters exclusive.

```
<let-uds> ::= <"a"-"z"> | "_"
<variable-name> ::= <let-uds> <variable-name> | <let-uds>
```

It is advised to use 64-bit signed integers (or better yet, 128-bit ones to implement storage of variables. The usage of `snake_case` is preferred for variable names.

Function names follow the same requirements as variable names do.

As for the variable scope (data visibility), all variables are visible to their current function instances (actions like recursive calls would yield multiple instances, hence creating multiple scopes) and nowhere outside. Expressions like `if` or `while` do not create scope visibility barriers (i.e. variables within loops are visible outside, as long as they're in the same function call instance). All function names are visible everywhere and SHOULD NOT be changed whatsoever.

### 4. Expressions

For ease of expression parsing, prefix expressions (like `+ a b`) are preferred over traditional expressions (like `a + b`). Operators would always take precedence over it's parameters, so it's easier to determine the number of parameters as soon as you see the operator. A formal definition of an expression is like:

```
<constant> ::= any value from -2^47 ~ 2^47-1
<expression> ::= <constant>
               | <variable-name>
               | <operator> <expression> <expression> ... <expression>
```

Some examples include:

* Prefix expression `+ 16 233` to infix expression `16 + 233`
* Prefix expression `+ * 3 4 - 7 2` to infix expression `(3 * 4) + (7 - 2)`
* Prefix expression `* * * 1 2 3 4` to infix expression `((1 * 2) * 3) * 4`
* Prefix expression `func_three 12 13 14` to infix expression `func_three(12, 13, 14)`, whereas `func_three` is a function with 3 parameters
* Prefix expression `func_four 1 + 5 6 - 9 3 4` to infix expression `func_four(1, 5 + 6, 9 - 3, 4)`

### 5. Operators

There are a few built-in operators which functions mostly like they do in C++ / Python, but certain care have been taken to avoid confusion (especially while in division or remainder calculations).

* Addition `+`: Accepts 2 parameters, yields sum of the two. Handle overflow when needed.
* Subtraction `-`: Accepts 2 parameters, yields the first subtracted by the second. Handle overflow when needed. Example: `- 17 12 = 5`
* Multiplication `*`: Accepts 2 parameters, yields the product of the two. Proper handle of overflow is required. Example: `* 16 -3 = -48`
* Remainder `%`: Accepts 2 parameters *a* and *b*, yields the smallest non-negative integer *k* where *a = |b|p + k*, such that p is an integer. Result is always 0 when *b* is 0. Examples:
  *  `% 27 4 = 3`, because 27 = 4 * 6 + 3
  * `% -18 5 = 2`, because -18 = 5 * (-4) + 2
  * `% 36 -7 = 1`, because 36 = |-7| * 5 + 1
  * `% 7 0 = 0`, this is a defined behavior
* Division `/`: Accepts 2 parameters *a* and *b*, yields `(a - a % b) / |b|`, where a subtracted by the remainder is guaranteed to be divisible by |b|. Division by 0 yields 0 anyway. Examples:
  * `/ 9 4 = 2`, which is the same as in C
  * `/ -2 -7 = -1`, because the remainder is 5
  * `/ 0 0 = 0` and `/ 6 0 = 0`, because division by 0 yields 0 always
* Equality `==`: Compares the 2 parameters, returns 1 if equal, 0 otherwise
* Less `<`: Compares the 2 parameters, returns 1 if the latter is greater, 0 otherwise
* Greater `>`: Compares the 2 parameters, returns 1 if the former is greater, 0 otherwise
* Less than or equal `<=`: See `<` operator
* Greater than or equal `>=`: See `<` operator
* Inequality `!=`: See `<` operator
* Logic and `and`: Returns 0 if any of the 2 parameters is 0, 1 otherwise
* Logic or `or`: Returns 1 if any of the 2 parameters is not 0, 0 otherwise
* Logic exclusive or `xor`: Returns 1 if one of the 2 parameters is 0 and the other is not, 0 otherwise
* Logic not `not`: Returns 1 if the parameter is 0, 0 otherwise

### 6. Assignment Statements

With respect to Rust grammar (although all variables here are mutable and do not implement move semantics), the keyword `let` is chosen (and only) for assignment statements. For example, the following statement assigns the value 2333 to variable `waifu`:

```
let waifu = 2333
```

An assignment could contain any legal statement as its r-value. We also have a more formal definition:

```
<assignment-statement> ::= let <variable-name> = <expression>
```

It should be noted that `<variable-name` should under all circumstances be of no conflict with function names, built-in operators or built-in keywords.

### 7. Conditional Statements

There will be only an `if` expression and no `else if`, `elif` or `else` involved. It is up to Nhotyp users to keep track of the rest of the cases. An example of doubling a value twice if it's less than 10 could be written as follows:

```
if < value 10 then
    let value = * value 2
    let value = * value 2
end if
```

More generally, we have the formal definition:

```
<conditional-statement> ::= if <expression> then
                                <code-block>
                            end if
```

The exact definition for a code block is to be given beyond all statement introductions (section 11).

### 8. Loop Statements

To further simplify the implementation of Nhotyp, we eradicated the `loop`, `for`, `foreach` and other statements, leaving only `while` loops. Additionally, `continue` and `break` control statements are also removed. The user should take care of them using flags combined with conditional statements. For example, a program calculating the sum of 1 to 100 (in the silly way) can be written as:

```
let i = 1
let sum = 0
while <= i 100 do
    let sum = + sum i
end while
```

For a formal definition of the `while` loop, we have:

```
<loop-statement> ::= while <expression> do
                         <code-block>
                     end while
```

It ought to be kindly noted that, as a simple interpreter, Nhotyp implementations should never try to detect infinite loops or such (the halting problem is unsolvable at a large scale).

### 9. Functions

As a modern programming language (modern as in time), Nhotyp would have to retain a method of defining functions. Functions have certain limitations:

* No two functions may have the same function name pairwise, nor may function names conflict with variable names. In the case of such conflicts, the variable name would trigger a runtime error.
* Function names are regularized in the same way as variable names are (i.e. consists of lowercase ASCII letters and underscores, while not being longer than 63 characters).
* Functions should never receive more than 16 parameters.
* When invoked, all parameters are assigned values and appear as 
* All functions should have exactly 1 return value at the end of the function. That is, return statements should never appear at other places in the function, and the last statement of the function is a return statement.
* The `main` function is the entry to the program, and all Nhotyp programs must have exactly 1 main function. The return value could be either used as the exit code of the interpreter or not, which is up to the implementation to decide.

We will define the function more formally as:

```
<parameter> ::= <variable-name>
<parameters> ::= <parameter> | <parameter> <variable-name>
<return-statement> ::= return <expression>
<function-name> ::= <variable-name>
<function-block> ::= function <function-name> <parameters> as
                         <code-block>
                         <return-statement>
                     end function
```

Now we could write a function that takes in 3 parameters and return the largest among them:

```
function max a b c as
    let res = a
    if > b res then
        let res = b
    end if
    if > c res then
        let res = c
    end if
    return res
end function
```

### 10. Input / Output

Nhotyp defined an input function (operator also) and an output statement. The input function could be seen as an operator with no parameters (0 parameters is allowed). It reads in exactly 1 integer from `stdin`, raising any errors if the input was not a valid integer, while also ensuring the read value is within the correct range.

The output statement, on the other side, prints a list of variables, separated by spaces. Each print statement produces exactly 1 line of output regardless of the number of variables to output. Additionally:

* The print statement does not accept more than 16 variables as input.
* It also does not accept expressions or constants as input. This means that if you wish to print a constant, you will have to first assign it to a variable and then print that variable.
* Certain implementations could add prefixes to input operators or output statements as an eye candy, as long as it does not break the workflow.
* Nhotyp does not currently support printing to `stderr`.

Thus we have a formal definition of the print statement:

```
<print-statement> ::= print <parameters>
```

A sample code of I/O with reference in other programming languages is as follows:

```
# python: var = int(input()) + 15
# c:      int var;
#         scanf("%d", &var);
#         var = var + 15;
# c++:    int var;
#         cin >> var;
#         var = var + 15
let var = + scan 15

# python: print('%d %d %d %d' % (ab, cd, xy, zw))
# c:      printf("%d %d %d %d\n", ab, cd, xy, zw);
# c++:    cout << ab << ' ' << cd << ' ' << xy << ' ' << zw << endl;
print ab cd xy zw
```

### 11. Misc

As we've introduced all definitions, operators and statements, we can finally produce a formal definition of statements, code blocks and the entire program:

```
<statement> ::= <assignment-statement>
              | <conditional-statement>
              | <loop-statement>
              | <print-statement>
<code-block> ::= <statement>
               | <code-block>
                 <statement>
<program> ::= <function-block>
            | <program>
              <function-block>
           ^ contains exactly 1 `main` function
```

There's some non-trivial notes that may help you implement Nhotyp interpreters faster:

* Proficient Nhotyp users should follow the 4-space block indentation as they would in Python. Though an interpreter should function properly even without indentation. These indentation are purely for better maintainability and readability.
* Tokens are *strictly* separated with spaces for ease of parsing. That means the expression `+ a b` should never appear with the operator stuck to the adjacent variable like `+a b`.
* Comment lines or purely empty lines could appear anywhere.

A deprecated Chinese version of the specification is available at `README_zh.md`. When the two have conflicts in definition, always respect this version for clarification.

## Trivia

* When you reverse the string *Python*, you get *nohtyP*. The letter *o* and *h* were swapped only to make it look better and looks more like an actual word (but it's not).
* Ignoring the error handling part will shorten your code for at least 50%.
* The size of all documentations summed up surpasses that of the interpreter itself.
* Nhotyp actually took its inspiration from Python, Rust and Pascal.
* It should have an interactive terminal, but it's not currently on the todo list.

