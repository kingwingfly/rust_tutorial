```rust
macro_rules! join {
    (@ {
        // One `_` for each branch in the `join!` macro. This is not used once
        // normalization is complete.
        ( $($count:tt)* )

        // The expression `0+1+1+ ... +1` equal to the number of branches.
        ( $($total:tt)* )

        // Normalized join! branches
        $( ( $($skip:tt)* ) $e:expr, )*

    }) => { ... };
    (@ { ( $($s:tt)* ) ( $($n:tt)* ) $($t:tt)* } $e:expr, $($r:tt)* ) => { ... };
    ( $($e:expr),+ $(,)?) => { ... };
    () => { ... };
}
```

以上是 Rust 异步运行时 `tokio` 的 `join!` 实现。

`join!` 宏的实现是一个递归宏，它的目的是将多个异步任务组合成一个异步任务。

一般的，若一个函数式宏具有某些不希望用户直接调用的 matcher，约定俗成地，可以使用 `@` 作为 matcher 的前缀。例如，`join!` 宏的递归入口。

[howtocodeit](https://www.howtocodeit.com/articles/writing-production-rust-macros-with-macro-rules)，这篇文章中例举并分析了tokio::join!的实现，值得一读。
