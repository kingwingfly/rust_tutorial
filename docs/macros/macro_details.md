# 片段分类符

## block

```rust
macro_rules! blocks {
    ($($block:block)*) => ();
}

blocks! {
    {}
    {
        let zig;
    }
    { 2 }
}
fn main() {}
```

## expr

```rust
macro_rules! expressions {
    ($($expr:expr)*) => ();
}

expressions! {
    "literal"
    funcall()
    future.await
    break 'foo bar
}
fn main() {}
```

## ident

```rust
macro_rules! idents {
    ($($ident:ident)*) => ();
}

idents! {
    // _ <- `_` 不是标识符，而是一种模式
    foo
    async
    O_________O
    _____O_____
}
fn main() {}
```

## item

`item` 分类符只匹配 Rust 的 item 的 **定义** (definitions) ， 不会匹配指向 item 的标识符 (identifiers)。

```rust
macro_rules! items {
    ($($item:item)*) => ();
}

items! {
    struct Foo;
    enum Bar {
        Baz
    }
    impl Foo {}
    /*...*/
}
fn main() {}
```

## lifetime

```rust
macro_rules! lifetimes {
    ($($lifetime:lifetime)*) => ();
}

lifetimes! {
    'static
    'shiv
    '_
}
fn main() {}
```

## literal

```rust
macro_rules! literals {
    ($($literal:literal)*) => ();
}

literals! {
    -1
    "hello world"
    2.3
    b'b'
    true
}
fn main() {}
```

## meta

`meta` 分类符用于匹配属性 (attribute)， 准确地说是属性里面的内容。通常会在 `#[$meta:meta]` 或 `#![$meta:meta]` 模式匹配中看到这个分类符。

```rust
macro_rules! metas {
    ($($meta:meta)*) => ();
}

metas! {
    ASimplePath
    super::man
    path = "home"
    foo(bar)
}
fn main() {}
```

>   文档注释其实是具有 `#[doc="…"]` 形式的属性，`...` 实际上就是注释字符串， 这意味着可以在宏里面操作文档注释！

## pat

```rust
macro_rules! patterns {
    ($($pat:pat)*) => ();
}

patterns! {
    "literal"
    _
    0..5
    ref mut PatternsAreNice
    0 | 1 | 2 | 3 
}
fn main() {}
```

## pat_param

```rust
macro_rules! patterns {
    (pat: $pat:pat) => {
        println!("pat: {}", stringify!($pat));
    };
    (pat_param: $($pat:pat_param)|+) => {
        $( println!("pat_param: {}", stringify!($pat)); )+
    };
}
fn main() {
    patterns! {
       pat: 0 | 1 | 2 | 3
    }
    patterns! {
       pat_param: 0 | 1 | 2 | 3
    }
}

macro_rules! patterns {
    ($( $( $pat:pat_param )|+ )*) => ();
}

patterns! {
    "literal"
    _
    0..5
    ref mut PatternsAreNice
    0 | 1 | 2 | 3 
}
fn main() {}
```

## path

```rust
macro_rules! paths {
    ($($path:path)*) => ();
}

paths! {
    ASimplePath
    ::A::B::C::D
    G::<eneri>::C
    FnMut(u32) -> ()
}
fn main() {}
```

## stmt

匹配语句

## tt

标记树

## ty

```rust
macro_rules! types {
    ($($type:ty)*) => ();
}

types! {
    foo::bar
    bool
    [u8]
    impl IntoIterator<Item = u32>
}
fn main() {}
```

## vis

visible，用于匹配 `pub` 关键字

`vis` 分类符会匹配 **可能为空** 的内容

```rust
macro_rules! visibilities {
    //         ∨~~注意这个逗号，`vis` 分类符自身不会匹配到逗号
    ($($vis:vis,)*) => ();
}

visibilities! {
    , // 没有 vis 也行，因为 $vis 隐式包含 `?` 的情况
    pub,
    pub(crate),
    pub(in super),
    pub(in some_path),
}
fn main() {}
```

# 再谈宏变量和宏展开

```rust
macro_rules! dead_rule {
    ($e:expr) => { ... };
    ($i:ident +) => { ... };
}

fn main() {
    dead_rule!(x+);
}
```

编译器匹配到`x+`后，发现不是表达式，直接 panic，不会再尝试第二个分支

因为编译器这方面不行

## 片段分类符的跟随限制

|  片段跟随符   |                           跟随限制                           |
| :-----------: | :----------------------------------------------------------: |
| `stmt` `expr` |                         `=>` `,` `;`                         |
|     `pat`     |                    `=>` `,` `=` `if` `in`                    |
|  pat_params   |                  `=>` `,` `=` `|` `if` `in`                  |
|  `path` `ty`  | `=>` `, ` `=` `|` `;` `:` `>` `>>` `[` `{` `as` `where` 或者`block`型的变量 |
|     `vis`     |  `,` 除了 priv 之外的标识符 任何以类型开头的标记 `ident` 或 `ty` 或 `path` 型的元变量                                                            |

其他片段分类符所跟的内容无限制

