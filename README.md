## nhotyp-lang

Nhotyp is a conceptual language designed for ease of implementation during my tutoring in an introductive algorithmic course at Harbin Institute of Technology, Weihai. The assignment specifications were written in Chinese, though. An English version might be added if requested.

Nhotyp is an interpretative language imitating a few features in Python and Rust, and used prefix expressions for ease of parsing. It was so designed to make the assignment easier to complete, if one chose to think the problem through, as it required few string operations and would never require the construction of an AST just to function properly.

The said repository introduces a standard implementation which would work on correct implementations, and should report common runtime errors if it was not written properly.

## Usage

Build the compiler with Rust and execute your Nhotyp code with the compiled intepreter:

```
rustc -o nhotyp nhotyp.rs
./nhotyp your_code.nh
```

## Language Specs

### i. 简介

Python 是一种动态解析型语言。

然而萌新并不会使用 Python... 他不管怎么配环境都没法让解释器运行起来。所以萌新决定自己设计一种语言，叫 Nhotyp。这种语言有一些特性，就是编译器非常小，而且没有任何自带标准库。语法也非常奇怪，导致几乎没有人会用它。

**现在，萌新想请你来帮他写一个 Nhotyp 解释器以运行他写的程序。**

萌新希望能用 Nhotyp 来进行一些简单的运算工作，于是，他对 Nhotyp 语言进行了以下标准定义。

**0. 注释：**只能有单行注释，除去所有前缀空格以外，注释行应且仅应以 `#` 字符开头。换句话说，如果某行开头除了空格就是一个 `#` 字符，那么你应视其为一行注释。

**1. 数据类型：**由于萌新的大脑自带转换功能，所以他能把所有整数都转换成浮点数、字符串或者其他复合类型。以及由于萌新太厉害了导致智商溢出，他无法理解数组是个什么东西（当然也不理解指针是什么）。所以 Nhotyp 语言里所有变量都是 64 位有符号整数。

**2. 变量：**所有变量名都是由小写字母或者下划线构成的 ASCII 字符串，且保证长度不超过 63 个字符（萌新是一个过于狂热的强迫症患者，连常量名都不想用大写，坚决使用小写下划线变量名 233 年不动摇）。同时所有变量里存储的值都是如上的 64 位有符号整数。

注意：你需要**模拟**溢出保证所有值处于 $-2^{47}$ 到 $2^{47}-1$ 这个范围之间。

有关作用域的问题：Nhotyp 语言中变量的作用域为整个函数，且不存在全局变量（全局空间中仅各函数变量名可见，且 Nhotyp 中没有闭包 (Closure) 这个概念）。换言之，函数的每次调用产生一个局部变量表（当然，这个表在函数调用完成后要销毁）。

**3. 表达式：**为了代码编写的方便（笔者根本看不出这里哪里方便了），萌新要求所有表达式都是前缀表达式，即先有符号，再有参数的表达式，例如：

* 前缀表达式 `+ 16 233` 对应中缀表达式 `16 + 233`
* 前缀表达式 `+ * 3 4 - 7 2` 对应中缀表达式 `(3 * 4) + (7 - 2)`
* 前缀表达式 `* * * 1 2 3 4` 对应中缀表达式 `((1 * 2) * 3) * 4`
* 前缀表达式 `func_three 12 13 14` 对应中缀表达式 `func_three(12, 13, 14)`，其中 `func_three` 是一个有三个参数的函数（注：保证返回一个 64 位有符号整型）
* 前缀表达式 `func_four 1 + 5 6 - 9 3 4` 对应中缀表达式 `func_four(1, 5 + 6, 9 - 3, 4)`

**4. 运算符：**以下运算符可以被用于在数之间运算，其中部分运算符可能和 C++ 或 Python 中的不同：

* 加法 `+`：接受两个参数，直接对两数进行加法。当运算结果超过变量允许范围时，适当处理溢出。
  例如：`+ a b; + 233 x; + 16 25;`
* 减法 `-`：接受两个参数，将第一个参数减去第二个。如果发生下溢的时候，适当处理溢出。
  例如：`- 17 12; - 6 x; - x y;`
* 乘法 `*`：接受两个参数，将两个参数相乘。注意：直接相乘再处理溢出可能会导致结果并不正确。
  例如：`* a b; * 5 x; * 233 666;`
* 模运算 `%`：接受两个参数，计算第一个参数 $a$ 在第二个参数 $b$ 的剩余系 $[0, |b|)$ 下的值。例如：
  `% 27 4` 计算 $27 \mod 4$，结果为 $3$，因为 $27 = 4 \times 6 + 3$
  `% -18 5` 计算结果为 $2$，因为 $-18 = 5 \times (-4) + 2$
  `% 36 -7` 计算结果为 $1$，因为 $36 = (-7) \times 5 + 1$
* 除运算 `/`：接受两个参数，计算第一个参数 $a$ 对第二个参数 $b$ 的运算 $\frac{a - a \% b}{|b|}$，即将 $a$ 减去 $a$ 模 $b$ 的值以后再对 $|b|$ 除，这样保证结果是一个整数。注意，在加上 $a \% b$ 后你不需要处理暂存值的溢出问题。例如：
  `/ 9 4` 计算 $\frac{9-1}{4}$ 得到 $2$，这与 C 语言里处理整数下取整的方法是类似的
  `/ -2 -7` 计算 $\frac{-2 - 5}{|-7|}$ 得到 $-1$
* 相等比较符 `== a b`：接受两个参数 $a$ 和 $b$，比较两个参数是否相等。若相等运算结果为 $1$，否则为 $0$
* 小于比较符 `< a b`：类似相等比较符，若 $a<b$ 运算结果为 $1$，否则为 $0$
* 大于比较符 `> a b`：参照小于比较符
* 小于等于比较符 `<= a b`：参照小于比较符
* 大于等于比较符 `>= a b`：参照小于比较符
* 不等比较符 `!= a b`：参照小于比较符
* 逻辑与运算 `and a b`：若参数 $a, b$ 中有一参数值为 $0$，运算结果为 $0$，否则为 $1$。
* 逻辑或运算 `or a b`：若参数 $a, b$ 中有一参数值非 $0$，运算结果为 $1$，否则为 $0$。
* 逻辑异或运算 `xor a b`：若两参数 $a, b$ 同时为 $0$ 或同时非 $0$，运算结果为 $0$，否则为 $1$。
* 逻辑非运算 `not a`：若唯一参数 $a$ 为 $0$，运算结果为 $1$，反之为 $0$。
* 所有其他用户自定义的运算符（也作函数，具体见下方定义）

**5. 赋值表达式：**参考 Rust 语法（只不过这里的所有变量都是可变的），我们用 `let` 作为赋值表达式的提示符。例如以下表达式代表将数值 `2333` 存入变量 `waifu` 中：

```
let waifu = 2333
```

进一步地，我们对赋值表达式有着以下具体定义：

```
let <variable> = <expression>
```

其中 `variable` 需为变量名，而 `expression` 需为一个合法表达式。

**6. 条件语句：**由于萌新非常清楚他的分支逻辑在干什么，所以他不需要 else if 或者 else 来判别未考虑到的其他情况。于是我们只有一种 if 语句，定义如下：

```
if <expression> then
    # code block here
end if
```

其中 `expression` 是一个合法的表达式。当且仅当该表达式运算结果不为 $0$ 时执行该条件语句。例如，下述样例仅在变量 `head` 与 `tail` 之和小于 `size` 时，修改另外一个变量 `res = max(head + tail, res)`（C++ 语法）：

```
if < + head tail size then
    let res = max + head tail res
    # res is updated if it's too small
end if
```

**7. 循环语句：**有的时候萌新不想穷尽他的精力来编写特别多的条件语句，这时候可以用循环语句来减少大量的重复人力工作。为了尽可能地让 Nhotyp 语言看起来简洁（信了你的鬼话），萌新只用了 while 循环，而没有用 for, foreach 或者 loop 这类高大上东西。自然地，continue 和 break 这类乱七八糟东西也没有被纳入 Nhotyp 的语言标准里。例如，计算 1 到 100 的和的 Nhotyp 程序可以这样写：

```
let i = 1
let sum = 0
while <= i 100 do
    let sum = + sum i
end while
```

其中，while 循环的形式化定义如下：

```
while <expression> do
    # loop code block here
end while
```

请注意：萌新非常清楚他的程序在干什么，所以不会整出刁钻的程序跑出非常慢的循环甚至是死循环来刁难你。所以如果你的解释器超时了，那么一定是你哪里写错了。

**8. 函数：**作为一款现代编程语言（笑），Nhotyp 必须要有一个完整的函数定义方法。Nhotyp 里的函数可以接受很多个参数（但是数量是固定的），定义如下：

```
function <name> <param_1> <param_2> ... <param_n> as
    # function content here
    return <expression>
end function
```

* 函数名两两不能重复（即保证给你的程序满足这一点），也不能和任何一个变量名重复。但是函数名的命名规则和变量名是一样的（最多支持 64 个字符，只能由小写字母和下划线构成）
* 最多支持 16 个参数，保证提供的程序中不会出现多于 16 个
* 函数**一定有一个返回值**，返回语句如定义中 `return ...`，其中 `expression` 可以是任意合法表达式
* 返回语句一定在函数的末尾（即返回语句以后除了注释和空行以外就没有东西了），也不可能在函数内的分支语句或者循环语句中出现
* **程序入口为主函数，其名称一定为 `main`，且保证每个程序均提供主函数；主函数返回值可以忽略**

一个合法的表达式中调用函数的方法如下：

```
<name> <param_1> <param_2> ... <param_n>
```

现在我们可以写一个 max 函数来求三个数之间的最大值（注意，这个函数是应该可以在你的解释器上真实运行的，只是现在缺少一个主函数）：

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

**9. 输入输出：**Nhotyp 定义了一个输入函数和一个输出语句。输入函数可以被视作一个没有参数的函数，会从 stdin 读入一个整数，保证该整数处在 Nhotyp 允许的变量范围内。输出语句则会向标准输出 stdout 打印一行一些整数，取决于这一行中有多少个参数。例如下：

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

具体地，`scan` 为一个输入函数，定义略；`print` 形式定义如下：

```
print <var_1> <var_2> ... <var_n>
```

其中参数数目不得超过 16 个，且每一个**参数必须为合法变量名**（你可以认为这 $n$ 个参数都已经存在了）。

**10. 大家一起和平地玩吧！**

* 萌新是一个代码风格非常良好的程序员。他严格执行 4 空格缩进规则，**即便对 Nhotyp 语言来讲行首空格和行尾空格都可以忽略**。
* 在 Nhotyp 语言中，token 之间必须有空格存在。你可以理解为：`+233 a` 这样可能具有迷惑性的输入是不会出现的。换言之，**你可以用空格分割开所有元素**。但是注意，**元素之间可能存在不止一个空格**。
* 程序中可能会出现空行（即除了空格什么都没有），而且**没有除了空格和换行符以外的任何字符**。
* **萌新写的所有代码都是对的**，不要问为什么。

### ii. 样例程序

**1. 最大值最小值：**用户向标准输入每次输入绝对值不大于 $10^7$ 的两个数 $a, b$；当 $a = b = 0$ 时结束程序。若程序未终止，每次输出一行两个整数，分别代表这两个数之中的最小值和最大值。

你的解释器应当能正常运行该程序并给出合理的结果。

Nhotyp 程序代码见样例输入1。

**2. 斐波那契：**第一行输入一个小于 $10$ 的正整数 $T$，代表总共会有 $T$ 组数据。接下来 $T$ 行每行一个不大于 $8$ 的整数 $a_i$，程序应输出斐波那契的第 $a_i$ 项（前三项分别为 $1, 1, 2$）该样例程序利用递归实现。

注：斐波那契数列满足除前两项外有 $a_{n+2} = a_n + a_{n+1}$。

Nhotyp 程序代码见样例输入2。

### iii. 输入输出

你的程序应当从 stdin 读取程序代码并解析。程序代码以一行 $79$ 个 `#` 结束。

接下来 stdin 会提供一堆整数，这些都是应当用 `scan` 运算符（或函数）读入你的程序的内容。一般地，如果程序运行没有问题，这些输入理应被你的程序读完。

然后 `print` 语句应当向标准输出 stdout 输出一些行，每行一些整数。我们会对这些结果与标准输出进行比对，来检验你的解释器是否正确。

### iv. 样例

**样例输入1（最小值最大值）：**

```
# returns min(a, b)
function min a b as
    let res = a
    if < b res then
        let res = b
    end if
    return res
end function

# returns max(a, b)
# probable overflow when a + b is too big
function max a b as
    return - + a b min a b
end function

# main function handles input / output
function main as
    let break = 0
    while not break do
        let a = scan
        let b = scan
        let break = 1
        if != a 0 then
            break = 0
        end if
        if != b 0 then
            break = 0
        end if
        if not break then
            let c = min a b
            let d = max a b
            print c d
        end if
    end while
    return -1
end main
###############################################################################
23333 76543
1234 89
0 0
```

**样例输出1：**

```
23333 76543
89 1234
```

**样例输入2（斐波那契）：**

```
function fib i as
    let res = 1
    if >= i 3
        let res = + fib - i 2 fib - i 1
    return res
end function

function main as
    let t = scan
    while > t 0 do
        let x = scan
        let res = fib x
        print res
        let t = - t 1
    end while
    return 0
end function
###############################################################################
7
1
2
3
4
5
6
8
```

**样例输出2：**

```
1
1
2
3
5
8
21
```

### v. 数据范围

* 保证程序代码不超过 256 行。
* 不会有任何单个表达式中包含的元素数量超过 128 个。
* 函数内变量名不会和外部任何一个函数名重复。
* 所有变量名、函数名长度均不超过 63 字符，且均由小写字母和下划线构成。
* 提供的 Nhotyp 程序保证是正确的（你不需要判断语法错误、段错误等等问题）。
* 等价的 C 语言程序在一台现代台式机上以给定输入运行时间不超过 30ms。