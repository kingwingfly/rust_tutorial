本笔记总结自[The Little Book of Rust Macros](https://github.com/veykril/tlborm/)

我对其中的案例有所简化

# 概述

## 宏的种类

### Attributes
内置宏 built-in
过程宏 proc-macro
派生宏 derive
### 函数式的宏

AKA 类函数宏

### macro_rules!

宏的变种，一般只考虑`macro_rules!`这一个宏

## 宏展开
很像 C 的 `Define`，在编译时自动替换
rust 中的宏是在 AST 之后展开，自己会加括号，不会改变运算顺序

## 卫生性
hygiene
一个概念，简单理解为：**宏不应该隐式地改变或创建变量**

比如一个宏，用户没有传入变量`a`，但它把`a`的值改变了；或者它创建了变量`b`，但始终没有move或drop，就不卫生

## 调试
`rustc +nightly -Zunpretty=expanded hello.rs`

# 思路

## 基本语法

```rust
macro_rules! $ident {
	($matcher) => {$expansion}
}
```
`expansion` 也可以叫做 `transcriber`



## matcher

匹配`()`或胡言乱语

```rust
macro_rules! four {
    () => {4};
}

macro_rules! gibberish {
    (lkflkd fn ["hi hi hi"]) => {...};
}
```

匹配`元变量`

```rust
macro_rules! vec_strs {
    (
        $($elem: expr),*
    ) => {
        ...
    }
}
```

该例中，`matcher`为 `$($elem: expr),*`

## 元变量

| name  |              example               |
| :---: | :--------------------------------: |
| block | a block surrounded by \{\}\(\)\[\] |
|expr|an expression|
| ident|an identifier (this includes keywords)|
|item|an item, like a function, struct, module, impl, etc.|
| lifetime|a lifetime (e.g. `'foo`, `'static`, ...)|
|  literal|a literal (e.g. `"Hello World!"`, `3.14`, `'🦀'`, ...)|
|meta|a meta item; the things that go inside the `#[...]` and `#![...]` attributes|
|pat|a pattern|
|path|a path (e.g. `foo`, `::std::mem::replace`, `transmute::<_, int>`, …)|
|stmt|a statement|
|tt|a single token tree|
|ty|a type|
|vis|a possible empty visibility qualifier (e.g. `pub`, `pub(in crate)`, ...)|

有一个特殊的元变量叫做 `$crate` ，它用来指代当前 crate 。

小例子：

```rust
macro_rules! times_five {
    ($e:expr) => { 5 * $e };
}

macro_rules! multiply_add {
    ($a:expr, $b:expr, $c:expr) => { $a * ($b + $c) };
}

macro_rules! discard {
    ($e:expr) => {};
}
macro_rules! repeat {
    ($e:expr) => { $e; $e; $e; };
}
```

## 反复

repetition

语法：

`$(...) sep rep`

其中：

-   `$`: a token (token意为标记)

-   `(...)`: the matcher which need repetition

-   `sep`: an optional separator token; example:`,` `;`

-   `rep`:
    -   `?`: appear zero or one time
    -   `*`:  appear any times
    -   `+`: appear once or more

Example:

```rust
#[macro_export]
macro_rules! vec_strs {
    (
        $($elem: expr) , *
        //			   ↑ ↑
        //     separator reputation
    ) => {
        {
            let mut v = Vec::new();
            $(
                v.push(format!("{}", $elem));
            )*
            v
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let v = vec_strs!(1, "hello", "world");
        assert_eq!(v, ["1", "hello", "world"]);
    }
}
```

多个变量的情况：

```rust
macro_rules! repeat_twice {
    ($($i: expr)*, $($i1:expr)*) => {
        $(
            let $i: (); let $i1:();
        )*
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test2() {
        repeat_twice!(a b c, d e f);    // This can work
        // repeat_twice!(g, h i); // This can not
    }
}
```

## macro metavariable expressions

Unstable now (2023.3.27) and we can not use it in a stable channel

```rust
#![feature(macro_metavar_expr)]
// at the very beginning of file

macro_rules! ident_counter {
    ($($i: ident)*, $($i1: ident)*) => {
        println!("{}", ${count(i)});
    };
}
```

# 实战

斐波纳切数列
$$
\begin{aligned}
&A[n] = A[n-2] + A[n-1]\\
\\
&\text {故写作：}\\
\\
&0, 1, \cdots,A[n-2]+A[n-1]\\
\end{aligned}
$$
本实例的目标是写一个能够解析表达式：$0, 1, \cdots,A[n-2]+A[n-1]$ ，并返回一个迭代器的宏

## 构建步骤

-   确定调用形式
-   确定想要生成的代码
-   改善调用形式
-   初步构建
-   替换
-   测试
-   导出

## 确定调用形式

```rust
let fib = recurrence![a[n] = 0, 1, ..., a[n-1] + a[n-2]];
for e in fib.take(10) { println!("{}", e) }
```

据此，初步构建宏：

```rust
macro_rules! recurrence {
    ( a[n] = $($inits:expr),+ , ... , $recur:expr ) => { /* ... */ };
}
```

依次匹配：

-   字面量：`a[n] =`
-   一个及以上的表达式（expr）
-   字面量：`, ... ,`
-   一个表达式（expr），本例中，此为通式

## 确定想要生成的代码

```rust
let fib = {
    use std::ops::Index;
    struct Fib {
    mem: [u64; 2],
    pos: usize,
}

impl Iterator for Fib {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let result = if self.pos < 2 {
            self.mem[self.pos]
        } else {
            let new = {
                let a = & self.mem;
                let n = self.mem.len();
                a[n-2] + a[n-1]
            };
            let mut temp = new.clone();
            for i in (0..2).rev() {
                std::mem::swap(&mut self.mem[i], &mut temp);
            }
            new
        };
        self.pos += 1;
        Some(result)
    }

    Fib { mem: [0, 1], pos: 0, }
}
```

## 改善调用形式

在 coding 的过程中发现，用户可能需要能够自定义迭代器中`Item`的类型

故修改宏的调用形式为：

```rust
let fib = recurrence![a[n]: u64 = 0, 1, ..., a[n-1] + a[n-2]];
```

据此初步构建宏：

```rust
macro_rules! recurrence {
    (
        a[n]: $type_: ty = $($inits:expr),+ , ... , $recur:expr
    ) => {};
}
```

## 初步构建

将前面几个步骤的结果组合在一起

```rust
macro_rules! recurrence {
    (
        a[n]: $type_: ty = $($inits:expr),+ , ... , $recur:expr
    ) => {{
        struct Fib {
            mem: [u64; 2],
            pos: usize,
        }

        impl Iterator for Fib {
            type Item = u64;

            fn next(&mut self) -> Option<Self::Item> {
                let result = if self.pos < 2 {
                    self.mem[self.pos]
                } else {
                    let new = {
                        let a = &self.mem;
                        let n = self.mem.len();
                        a[n - 2] + a[n - 1]
                    };
                    let mut temp = new.clone();
                    for i in (0..2).rev() {
                        std::mem::swap(&mut self.mem[i], &mut temp);
                    }
                    new
                };
                self.pos += 1;
                Some(result)
            }
        }

        Fib {
            mem: [0, 1],
            pos: 0,
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macro_test() {
        let mut fib = recurrence![a[n]: u64 = 0, 1, ..., a[n-1] + a[n-2]];
        assert_eq!(fib.next().unwrap(), 0);
        assert_eq!(fib.next().unwrap(), 1);
        assert_eq!(fib.next().unwrap(), 1);
        assert_eq!(fib.next().unwrap(), 2);
        assert_eq!(fib.next().unwrap(), 3);
        assert_eq!(fib.next().unwrap(), 5);
    }
}
```

`cargo test`

```shell
error: local ambiguity when calling macro `recurrence`: multiple parsing options: built-in NTs expr ('inits') or 1 other option.
   --> macros/src/lib.rs:148:53
    |
46  |         let mut fib = recurrence![a[n]: u64 = 0, 1, ..., a[n-1] + a[n-2]];
    |                                                     ^^^
```

这是由于在某次版本更新后， `expr` 之后只能跟随 `=>`、`,`、`;` 之一

故解决方法为用 `;...;` 代替 `,...,` ，修改以下两行：

```rust
macro_rules! recurrence {(a[n]: $type_: ty = $($inits:expr),+ ;...; $recur:expr) => {{}}
let mut fib = recurrence![a[n]: u64 = 0, 1;...; a[n-1] + a[n-2]];
```

当前完整代码：

```rust
macro_rules! recurrence {
    (
        a[n]: $type_: ty = $($inits:expr),+ ;...; $recur:expr
    ) => {{
        struct Fib {
            mem: [u64; 2],
            pos: usize,
        }

        impl Iterator for Fib {
            type Item = u64;

            fn next(&mut self) -> Option<Self::Item> {
                let result = if self.pos < 2 {
                    self.mem[self.pos]
                } else {
                    let new = {
                        let a = &self.mem;
                        let n = self.mem.len();
                        a[n - 2] + a[n - 1]
                    };
                    let mut temp = new.clone();
                    for i in (0..2).rev() {
                        std::mem::swap(&mut self.mem[i], &mut temp);
                    }
                    new
                };
                self.pos += 1;
                Some(result)
            }
        }

        Fib {
            mem: [0, 1],
            pos: 0,
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macro_test() {
        let mut fib = recurrence![a[n]: u64 = 0, 1;...; a[n-1] + a[n-2]];
        assert_eq!(fib.next().unwrap(), 0);
        assert_eq!(fib.next().unwrap(), 1);
        assert_eq!(fib.next().unwrap(), 1);
        assert_eq!(fib.next().unwrap(), 2);
        assert_eq!(fib.next().unwrap(), 3);
        assert_eq!(fib.next().unwrap(), 5);
        for _ in 0..10 {
            fib.next();
        }
        assert_eq!(fib.next().unwrap(), 987);
    }
}

```

## 替换

完成后：

```rust
// use this macro to count the initial size of the mem
macro_rules! count_exprs {
    // () => {0};
    ($e:expr) => {1};
    ($e:expr, $($e1:expr),+) => { 1 + count_exprs!($($e1),+) }
}

macro_rules! recurrence {
    (
        a[n]: $type_: ty = $($inits:expr),+ ;...; $recur:expr
    ) => {{
        const MEM_SIZE: usize = count_exprs!($($inits),+);
        struct Fib {
            mem: [$type_; MEM_SIZE],
            pos: usize,
        }

        impl Iterator for Fib {
            type Item = $type_;

            fn next(&mut self) -> Option<Self::Item> {
                let result = if self.pos < MEM_SIZE {
                    self.mem[self.pos]
                } else {
                    let new = {
                        let a = &self.mem;
                        let n = MEM_SIZE;
                        $recur
                    };
                    let mut temp = new.clone();
                    for i in (0..MEM_SIZE).rev() {
                        std::mem::swap(&mut self.mem[i], &mut temp);
                    }
                    new
                };
                self.pos += 1;
                Some(result)
            }
        }

        Fib {
            mem: [$($inits),+],
            pos: 0,
        }
    }};
}
```

`cargo test`

```shell
error[E0425]: cannot find value `a` in this scope
   --> macros/src/lib.rs:132:57
    |
132 |         let mut fib = recurrence![a[n]: u64 = 0, 1;...; a[n-1] + a[n-2]];
    |                                                         ^ not found in this scope

error[E0425]: cannot find value `n` in this scope
   --> macros/src/lib.rs:132:59
    |
132 |         let mut fib = recurrence![a[n]: u64 = 0, 1;...; a[n-1] + a[n-2]];
    |                                                           ^ not found in this scope

error[E0425]: cannot find value `a` in this scope
   --> macros/src/lib.rs:132:66
    |
132 |         let mut fib = recurrence![a[n]: u64 = 0, 1;...; a[n-1] + a[n-2]];
    |                                                                  ^ not found in this scope

error[E0425]: cannot find value `n` in this scope
   --> macros/src/lib.rs:132:68
    |
132 |         let mut fib = recurrence![a[n]: u64 = 0, 1;...; a[n-1] + a[n-2]];
    |                                                                    ^ not found in this scope
```

据说在 nightly 版本中，可以编译通过。此处不通过的原因在于，测试中的 `a` 和 `n` 与宏中的 `a` 和 `n` 具有不同的上下文，解决方法是：

```rust
macro_rules! recurrence {
    (
        $seq: ident [$ind: ident]: $type_: ty = $($inits:expr),+ ;...; $recur:expr
        // seq is short for sequence; ind is short for index
    ) => {{
        ...
                    let new = {
                        let $seq = &self.mem;
                        let $ind = MEM_SIZE;
                        $recur
                    };
        ...
}
```

这样，rustc 才能推断出 `a` 和 `n` 是 `ident`(identifier)

## 其他测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macro_test2() {
        let mut fib = recurrence!(f[i]: i32 = 1, 1, 3;...; f[i-3]+f[i-2]+f[i-1]);
        assert_eq!(fib.next().unwrap(), 1);
        assert_eq!(fib.next().unwrap(), 1);
        assert_eq!(fib.next().unwrap(), 3);
        assert_eq!(fib.next().unwrap(), 5);
        assert_eq!(fib.next().unwrap(), 9);
        for _ in 0..10 {
            fib.next();
        }
        assert_eq!(fib.next().unwrap(), 7473);
	}
}
```

## 导出宏

```rust
#[macro_export]
macro_rules! count_exprs {...

#[macro_export]
macro_rules! recurrence {
    (...) => {
        use $crate::count_exprs;
        ...
    };
}
```

## 完整代码

```rust
#[macro_export]
macro_rules! vec_strs {
    (
        $($elem: expr),*
    ) => {
        {
            let mut v = Vec::new();
            $(
                v.push(format!("{}", $elem));
            )*
            v
        }
    };
}

macro_rules! repeat_twice {
    ($($i: ident)*, $($i1: ident)*) => {
        $(
            let $i: (); let $i1:();
        )*
    };
}

struct Fib {
    mem: [u64; 2],
    pos: usize,
}

impl Iterator for Fib {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let result = if self.pos < 2 {
            self.mem[self.pos]
        } else {
            let new = {
                let a = &self.mem;
                let n = self.mem.len();
                a[n - 2] + a[n - 1]
            };
            let mut temp = new.clone();
            for i in (0..2).rev() {
                std::mem::swap(&mut self.mem[i], &mut temp);
            }
            new
        };
        self.pos += 1;
        Some(result)
    }
}

#[macro_export]
macro_rules! count_exprs {
    // () => {0}; This line could never be reached.
    ($e:expr) => {1};
    ($e:expr, $($e1:expr),+) => { 1 + count_exprs!($($e1),+) }
}

#[macro_export]
macro_rules! recurrence {
    (
        $seq: ident [$ind: ident]: $type_: ty = $($inits:expr),+ ;...; $recur:expr
        // seq is short for sequence; ind is short for index
    ) => {{
        use $crate::count_exprs;

        const MEM_SIZE: usize = count_exprs!($($inits),+);

        struct Fib {
            mem: [$type_; MEM_SIZE],
            pos: usize,
        }

        impl Iterator for Fib {
            type Item = $type_;

            fn next(&mut self) -> Option<Self::Item> {
                let result = if self.pos < MEM_SIZE {
                    self.mem[self.pos]
                } else {
                    let new = {
                        let $seq = &self.mem;
                        let $ind = MEM_SIZE;
                        $recur
                    };
                    let mut temp = new.clone();
                    for i in (0..MEM_SIZE).rev() {
                        std::mem::swap(&mut self.mem[i], &mut temp);
                    }
                    new
                };
                self.pos += 1;
                Some(result)
            }
        }

        Fib {
            mem: [$($inits),+],
            pos: 0,
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let v = vec_strs!(1, "hello", "world");
        assert_eq!(v, ["1", "hello", "world"]);
    }

    #[test]
    fn test2() {
        repeat_twice!(_a _b _c, _d _e _f); // This can work

        // repeat_twice!(g, h i);    // This can not
    }

    #[test]
    fn fib_test() {
        let mut fib = Fib {
            mem: [0, 1],
            pos: 0,
        };
        assert_eq!(fib.next().unwrap(), 0);
        assert_eq!(fib.next().unwrap(), 1);
        assert_eq!(fib.next().unwrap(), 1);
        assert_eq!(fib.next().unwrap(), 2);
        assert_eq!(fib.next().unwrap(), 3);
        assert_eq!(fib.next().unwrap(), 5);
    }

    #[test]
    fn macro_test() {
        let mut fib = recurrence![a[n]: u64 = 0, 1;...; a[n-1] + a[n-2]];
        assert_eq!(fib.next().unwrap(), 0);
        assert_eq!(fib.next().unwrap(), 1);
        assert_eq!(fib.next().unwrap(), 1);
        assert_eq!(fib.next().unwrap(), 2);
        assert_eq!(fib.next().unwrap(), 3);
        assert_eq!(fib.next().unwrap(), 5);
        for _ in 0..10 {
            fib.next();
        }
        assert_eq!(fib.next().unwrap(), 987);
    }

    #[test]
    fn macro_test2() {
        let mut fib = recurrence!(f[i]: i32 = 1, 1, 3;...; f[i-3]+f[i-2]+f[i-1]);
        assert_eq!(fib.next().unwrap(), 1);
        assert_eq!(fib.next().unwrap(), 1);
        assert_eq!(fib.next().unwrap(), 3);
        assert_eq!(fib.next().unwrap(), 5);
        assert_eq!(fib.next().unwrap(), 9);
        for _ in 0..10 {
            fib.next();
        }
        assert_eq!(fib.next().unwrap(), 7473);
    }
}

```
