# Rust 生命周期教程综述

## 摘要

本文对各种已有的有关生命周期的解释进行综述，帮助读者理解Rust生命周期。首先讲解 Rust语言圣经 的内容建立初步认识，再以 Learn Rust by Example 的内容作为补充，随即讲解 Rustonomicon 中的高级知识，最终简要介绍Rust新的借用检查器。

## 目录

[toc]

## 引言

生命周期是学习Rust过程中的一个难点，但其本质是一个标记，理解以后并不困难，参考资料也十分丰富。

本文可以认为是对各种解释的汇总，读者看完大概率都能有所收获。

[GitHub](https://github.com/kingwingfly/rust_tutorial)

## 正文

关于生命周期，有以下资料可供参考：

- [Rust语言圣经](https://course.rs/advance/lifetime/advance.html)
- [Learn Rust by Example](https://doc.rust-lang.org/rust-by-example/scope/lifetime.html)
- [The Rustonomicon](https://doc.rust-lang.org/nomicon/lifetimes.html)
- [Polonius update](https://blog.rust-lang.org/inside-rust/2023/10/06/polonius-update.html) && [An alias-based formulation of the borrow checker](https://smallcultfollowing.com/babysteps/blog/2018/04/27/an-alias-based-formulation-of-the-borrow-checker/)

以下对上面各教程内容进行讲解，总有一家之言可以深得你心。

### Rust语言圣经

Rust语言圣经在GitHub上开源，是最优秀的Rust中文教程，此外还有中译的《Rust权威指南》。

此书在“认识生命周期”的章节中，回答了数个问题，先看两个：

Q：什么是生命周期标记？

A：一个`'`开头的标记，仅仅是一个记号；用来帮助编译器检查；由于“消除规则”，部分情况可以省略。

e.g: `T: 'a`, `fn longest<'a>(x: &'a str, y: &'a str) -> &'a str { ... }`

值得说明的，本文中“生命周期”指作用域的长度，类似0-70岁，16-35岁；“生命周期标记”就是给一段“生命周期”起个名字，类似“一生”“青年”。

当然，在一些书籍中，你也能看到这样的注释：

```rust
{
    let r;                // ---------+-- 'a
                          //          |
    {                     //          |
        let x = 5;        // -+-- 'b  |
        r = &x;           //  |       |
    }                     // -+       |
                          //          |
    println!("r: {}", r); //          |
}                         // ---------+
```

此图中，同一列的两个加号标记出一段生命周期，名为`'a 'b`，例如，第2行到第10行为`r`生命周期，名为`'a`。

---

Q：为什么需要生命周期标记？

A: 1. 编译器“不够聪明”，需要补充更多信息；2. 辅助程序员，降低心智负担。

在江湖传说中，早期Rust几乎处处都需要生命周期标注，只是在发展中，大佬们渐渐采纳了几条省略规则，被称为“消除规则”，这就是作为Rust初学者，几乎见不到生命周期标记的原因。而事实上，大量成功的Rust crate（Rust 库）也致力于通过优秀的设计，避免向用户暴露大量的生命周期标记，比如 Bevy（Rust热门游戏引擎），此外，axum（热门后端框架）目前也将减少生命周期标记作为优化目标之一。

函数签名中的消除规则：

- 每个引用都有各自的生命周期（事实）
- 函数参数只有一个是引用时，编译器能够推断
- 参数有`Self`的引用时，编译器采用`Self`的引用的生命周期

对于第一点不多解释。不同于VS-Code，zed等编辑器对参数中所有引用都会提示生命周期。

对于第二点，相当于选择题只有一个选项。

对于第三点，理解为`fn<'self, 'a, 'b>(&'self self, _: &'a T, _: 'b T) -> &'self T`即可。

例子：

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
```

可以看到，既没有`Self`的引用，且有多个引用，不符合任何一条规则，因此编译器无法推断，需要手动标注。

再看两个有`Self`的引用的例子：

```rust
impl Foo {
    fn foo(&self, _: &str) -> &str {
        todo!()
    }
}
```

> 函数签名中，`&self`是`self: &Self`的语法糖

再看一个推断错误的例子：

```rust
// Error
impl Foo {
    fn foo(&self, x: &str) -> &str {
        x	// <- 生命周期不匹配
    }
}
```

根据规则，函数返回的引用的生命周期取`Self`的引用的生命周期。咱“降糖”（desugar）以后：

```rust
impl Foo {
    fn foo<'a, 'b>(&'a self, x: &'b str) -> &'a str {
        x
    }
}

// lifetime may not live long enough
// consider adding the following bound: `'b: 'a`
```

编译器的提示意思是，`'b`可能活得不够长。`'b: 'a`的意思是 'b outlives 'a，即“请添加以下约束，'b比'a长”。

事实上，`'b`和`'a`作为两个不同的生命周期参数，二者的关系是不确定的。编译器有理由认为，存在`'a`长于`'b`的可能，在这种情况下，调用`foo`后，`x`的生命周期被延长了。

为了便于讲述后果，记：

- `x`指向的`str`的生命周期为`'str`
- `x`的生命周期为`'x`
- `Self`的生命周期为`'self`

基于下面三个事实：

- `x`只是`str`一个引用，当创建`x`的时候，编译器给了`x`一个生命周期，它是不大于`str`的生命周期的。我们可以缩小`x`的生命周期，但显然不能延长它（悬垂指针）。即我们有个生命周期约束：`'str: 'x`
- `foo`把`'x`延长到了`'self`
- `'self`可能 outlives `'str`

总之，有可能延长后，导致 `'x: 'str`，与前面的约束矛盾，具体表现为悬垂指针。

我们可以按照编译器提示的进行修改，当然也可以`fn foo<'a, 'b>(&'a self, x: &'b str) -> &'b str`。

---

Q：什么是 `'static`

A：一个特殊的生命周期，表示整个程序的生命周期。灵感可能来自于 `static` 关键字。

中文是 静态生命周期。

---

当然，Rust语言圣经也例举了一些实操性的例子。

第一，生命周期标记需要声明：

```rust
fn foo<'a>(x: &'a str) {...}

struct Foo<'a> { // 可见，'a也是类型的一部分
    x: &'a str
}

impl<'a>' Trait for Foo<'a> { ... } // 若 Trait 中的方法不需要 'a，可以写成
impl Trait for Foo<'_> { ... }

trait Foo<'a> {
    fn foo(&self, x: &'a T)
    	where T: Trait + 'a;
}
```

第二，生命周期的起始：

早期，Rust中生命周期是从创建到作用域结束，即`}`；后来，变为从创建到最后一次使用。详见：[NLL (Non-Lexical Lifetime)](https://course.rs/advance/lifetime/advance.html#nll-non-lexical-lifetime)

第三，异步运行时：

当我们编写多线程程序时，会发现无法将`&self`move到别的线程，导致数据共享很不方便：

```rust
use std::thread;

struct Adder {
    v: i32,
}

impl Adder {
    fn add(&mut self, x: i32) {
        let jh = thread::spawn(move || {
            self.v += x;
        });
        jh.join().unwrap();
    }
}
```

这是一段无法编译的程序：

```text
error[E0521]: borrowed data escapes outside of method
  --> src\lib.rs:9:18
   |
8  |       fn add(&mut self, x: i32) -> i32 {
   |              ---------
   |              |
   |              `self` is a reference that is only valid in the method body
   |              let's call the lifetime of this reference `'1`
9  |           let jh = thread::spawn(move || {
   |  __________________^
10 | |             self.v += x;
11 | |         });
   | |          ^
   | |          |
   | |__________`self` escapes the method body here
   |            argument requires that `'1` must outlive `'static`
...
```

可以看到，编译器希望`'self: 'static`。让我们看看`thread::spawn`的函数签名：

```rust
pub fn spawn<F, T>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
```

这里有三个问题：

Q：`F: FnOnce() -> T + Send + 'static,`是什么意思？

A：泛型 `F`满足约束 `FnOnce() -> T`和`Send`两个 trait，且 `F`的生命周期至少为`'static`。注意断句，是`F: FnOnce() -> T | + Send | + 'static,`

Q：`FnOnce() -> T`是什么？

A：闭包的特征。意思是，泛型`F`上有个`call_once`方法，只能调用一次，没有传入参数，返回值类型为`T`。

Q：为什么`F`需要`'static`的生命周期？

A：`F`会被送到别的线程执行，且可能持续到主线程结束。

为了方便阅读，我把代码再复制过来：

```rust
fn add(&mut self, x: i32) {
    let jh = thread::spawn(move || {
        self.v += x;
    });
    jh.join().unwrap();
}
```

我们知道，事实上`F`的生命周期不需要到`'static`，只需要到`join()`处。

Q：这是否过于严格了呢？

A：分两种情况：1. 若在`add`中`join`，很呆，不如直接 `self.v += x;` 2. 不`join`，那显然有可能引起竞态条件。因此是合理的。

不妨直接尝试修复：

1. 修改签名 `fn add(&'staic mut self, x: i32)`，编译通过。

这是编译器的建议。`'static mut self`的语义是“我们有一个与程序活得一样长的可变引用”。但创建一个`'static`的可变引用是十分困难的，这导致`add`几乎是不可用的。

2. 使用 `thread::scope`

```rust
impl Adder {
    fn add(&mut self, x: i32) {
        thread::scope(|s| {
            s.spawn(move || {
                self.v += x;
            });
        });
    }
}
```

`scope`具体的实现也依赖生命周期，但过于高级了，up先怂一波。

3. 慎重考虑并发bug，采用内部可变性或原子类型（这是实用的）：

```rust
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use std::thread;

struct Adder {
    v: Arc<AtomicI32>,
}

impl Adder {
    fn add(&self, x: i32) {
        let v = self.v.clone();
        let jh = thread::spawn(move || {
            v.fetch_add(x, Ordering::SeqCst);
        });
        jh.join().unwrap();
    }
}
```

---

### Learn Rust by Example

考虑到接下来的参考文献都是英文，故up会减少中文的使用，在Rust的自主学习中，知道英文的表达是十分重要的。

Initially, this tutorial differs（区分） the `lifetime` and `scope`（作用域）

- lifetime: a variable's（变量） lifetime begins when it is created and ends when it is destroyed.
- scope: the scope of the borrow is determined by where the reference is used.

---

在之前的例子中，我们看到，`thread::spawn`需要一个`'static`的闭包，但是为什么编译器会建议我们，将`&self`的生命周期改成`'static`？

答：函数（闭包）也是有自己的生命周期的。某些函数的生命周期是隐式的（消除规则），对于闭包来说，连参数也是自动捕获的。

```rust
foo<'a, 'b>(_: &'a str, _: 'b str) { ... }
// `foo` has lifetime parameters `'a` and `'b`
```

This lifetime syntax indicates that the lifetime of `foo` may not exceed（超出） that of either `'a` or `'b`.

当然，函数有生命周期是有点抽象的，你可以理解为函数作用域的长度。将`foo` inline 到调用的位置，相信你就能理解了。

---

Lifetime in struct:

```rust
struct Foo<'a> {
    v: &'a str,
}
```

Naturally, we know struct is actually just a continuous memory constructed by its fields, it is reasonable to inherit the lifetimes of its field.

---

Coercion（强制转换）:

A longer lifetime can be coerced into a shorter one so that it works inside a scope it normally wouldn't work in. This comes in the form of inferred coercion by the Rust compiler, and also in the form of declaring a lifetime difference:

```rust
fn choose_first<'a: 'b, 'b>(first: &'a i32, _: &'b i32) -> &'b i32 {
    first
}
```

---

如何创建一个 ``&'static T`？

1. `static`关键字

2. `Box::leak`

3. unsafe
   ```rust
   fn get_str_at_location(pointer: usize, length: usize) -> &'static str {
     unsafe { from_utf8_unchecked(from_raw_parts(pointer as *const u8, length)) }
   }
   ```

无界生命周期，这在前三本参考资料中均有提到：

```rust
fn f<'a, T>(x: *const T) -> &'a T {
    unsafe { &*x }
}
```

---

前面说的 “消除规则”是Rust语言圣经的翻译，在英文中，叫 `Elision`（省略）

---

### Rustonomicon

> The Rustonomicon digs into all the awful details that you need to understand when writing Unsafe Rust programs.

在 Rustonomicon 中，首先回答了一个问题：

Q：为什么我们前面没有讨论函数体（function body）中的 lifetimes？

A：不需要。Rust编译器能够很好的处理 lifetimes in local context。当跨越函数的边界时，我们就需要 lifetimes 了。

---

在 Rustonomicon 中，也列举了一些 desugar 的例子：

```rust
fn as_str<'a>(data: &'a u32) -> &'a str {
    'b: {
        let s = format!("{}", data);
        return &'a s;
    }
}
```

That basically implies that we're going to find a str somewhere in the scope the reference to the u32 originated in, or somewhere _even earlier_.

即，需要一个`str`outlives`u32`，但这个`str`只能在函数体中产生，这是不可能的（因为即使产生，也会被自动 Drop 掉）。正确的做法是返回一个 String。

> `String` 是一个结构体，其中，一个field是指向 `str` 的指针，一个是 `str` 的长度。`str` 实际是`[u8]`，编译器忽略其大小，即 Rust 中的`?Sized`。其实，`Vec`也是一样的。

还有个例子：

```rust
// Compile Error
let mut data = vec![1, 2, 3];
let x = &data[0];
data.push(4);	// == Vec::push(&mut data, 4);
println!("{}", x);
```

显然，这违反借用规则。Rustonomicon 想用这个例子说明：编译器并不懂 “代码”，它只是发现在 immutable ref 的生命周期中，出现了一个 mutable ref 而已。

---

Rustonomicon 还介绍了一些由于生命周期延长导致的问题：

我们提到过，在当前Rust中，变量的生命周期是从创建到最后一次使用，下面这个例子没有问题：

```rust
#[derive(Debug)]
struct Foo<'a>(&'a [u8]);

let mut buf = [1];
let foo = Foo(&buf);
println!("{foo:?}");
buf[0] = 2;
```

但是，若为 `Foo`实现`Drop`后：

```rust
#[derive(Debug)]
struct Foo<'a>(&'a [u8]);
impl Drop for Foo<'_> {
    fn drop(&mut self) {}
}

let mut buf = [1];
let foo = Foo(&buf);
println!("{foo:?}");
buf[0] = 2;
```

`foo`的生命周期就延长到了最后一次使用，即此时，在最后一行，同时存在`buf`的 immutable ref 和 `buf[0]=2`。

这种由于实现`Drop`导致的行为改变常常给程序员带来会心一击。

当然，由于分支的存在，一个变量可能存在多个生命周期：

```rust
let mut data = vec![1, 2, 3];
let x = &data[0];

if some_condition() {
    println!("{}", x);
    data.push(4);
} else {
    data.push(5);
}
```

这在所有可能的走向中，都不应违背借用规则。

---

以下片段意从 `HashMap`中取 key ，无则先插入默认值，最终返回`&mut V`。

```rust
// Compile Error
fn get_default<'m, K, V>(map: &'m mut HashMap<K, V>, key: K) -> &'m mut V
where
    K: Clone + Eq + Hash,
    V: Default,
{
    match map.get_mut(&key) {
        Some(value) => value,	// return 'm, so line 7's `&mut map: 'm`
        None => {
            map.insert(key.clone(), V::default());
            map.get_mut(&key).unwrap()	// return 'm, so line 11's `&mut map: 'm`
        }
    }
}
```

根据`get_default`和`HashMap::get_mut`的函数签名，`map`和返回值的生命周期都是`'m`。因此，第7行和第11行均可变引用了`map`，且生命周期为`'m`，无法编译。

---

省略规则之前已经讲过，Rustonomicon 多了一些例子：

```rust
fn print(s: &str);                                      // elided
fn print<'a>(s: &'a str);                               // expanded

fn debug(lvl: usize, s: &str);                          // elided
fn debug<'a>(lvl: usize, s: &'a str);                   // expanded

fn substr(s: &str, until: usize) -> &str;               // elided
fn substr<'a>(s: &'a str, until: usize) -> &'a str;     // expanded

fn get_str() -> &str;                                   // ILLEGAL

fn frob(s: &str, t: &str) -> &str;                      // ILLEGAL

fn get_mut(&mut self) -> &mut T;                        // elided
fn get_mut<'a>(&'a mut self) -> &'a mut T;              // expanded

fn args<T: ToCStr>(&mut self, args: &[T]) -> &mut Command                  // elided
fn args<'a, 'b, T: ToCStr>(&'a mut self, args: &'b [T]) -> &'a mut Command // expanded

fn new(buf: &mut [u8]) -> BufWriter;                    // elided
fn new(buf: &mut [u8]) -> BufWriter<'_>;                // elided (with `rust_2018_idioms`)
fn new<'a>(buf: &'a mut [u8]) -> BufWriter<'a>          // expanded
```

---

无界生命周期 `'unbounded`

```rust
fn get_str<'a>(s: *const String) -> &'a str {
    unsafe { &*s }
}

fn main() {
    let soon_dropped = String::from("hello");
    let dangling = get_str(&soon_dropped);
    drop(soon_dropped);
    println!("Invalid str: {}", dangling); // Invalid str: gӚ_`
}
```

注意到`get_str`的输入中没有生命周期，只有输出中有生命周期。于是在编译器看来，`'a`可以不受限制地被推断为任何生命周期。

一般认为，大于`'static`的生命周期是没有意义的。但在Rust中，你不能创建 `&'static &'a str`，因为不能`'static`地引用``&'a str`，后者的生命周期不够长。有了 `'unbounded`之后，也许可以玩些花活，但一般把它视为`'static`也无伤大雅。

---

Higher-Rank Trait Bound （HRTB）

有时，我们需要写这样的代码（也许是通过管道把任务发送给Worker执行）：

```rust
struct Closure<F> {
    data: (u8, u16),
    func: F,
}

impl<F> Closure<F>
    where F: Fn(&'? (u8, u16)) -> &'? u8, // Here absolutely needs lifetime
{
    fn call(&self) -> &u8 {
        (self.func)(&self.data)
    }
}
```

第7行的函数签名不符合消除规则，需要生命周期标注，但我们甚至没有可用生命周期参数。

用心感受一下，`Fn`是函数或闭包的特征，而函数或闭包自然是哪里需要就能用在哪里，那么，

答案是：

```rust
where for<'a> F: Fn(&'a (u8, u16)) -> &'a u8,
```

意为：编译器姐姐，对于所有可能的`'a`，都检查一下。

明确：借用检查器对生命周期是全知的，自然可以进行这样的检查。

另，这样写也可以

```rust
where F: for<'a> Fn(&'a (u8, u16)) -> &'a u8,
```

就像是魔法一样。

---

Subtyping and Variance

子类型和型变

都是范畴论的概念，在`typescript`以及某些离谱面试中有所涉及。Rustonomic 用它们描述所有权与借用的关系。

由此例引入：

```rust
fn debug<'a>(a: &'a str, b: &'a str) {
    println!("a = {a:?} b = {b:?}");
}

fn main() {
    let hello: &'static str = "hello";
    {
        let world = String::from("world");
        let world = &world; // 'world has a shorter lifetime than 'static
        debug(hello, world); // hello silently downgrades from `&'static str` into `&'world str`
    }
}
```

`&'static str`隐式地降级为`&'world str`，因此可以编译。

Q：为什么可以降级？

A：因为借用是顺变（covariance）的，已知`'static: 'world`，故`&'static str: &'world str`。

这里有两个疑惑：

Q1：结论里的冒号是什么意思？

A：`&'static str: &'world str`，读作：`&'static str`是`&'world str`的子类型。

作为初学者，也许有点反直觉：能使用`&'world str`的地方一定能使用`&'static str`的，反之则不一定。`&'static str`明明比`&'world str`强，为什么是个“子”呢？因为儿子比父亲强。

Q2：什么是“顺变”？

我们用 `F(x)` 表示对 `x`做一次`F`变换。若`Sub: Sup`且`F(Sub): F(Sup)`，则`F`是顺变的。

在本例中，`Sub='static` `Sup='world`，`F`表示取不可变引用的操作，检查发现的确是顺变的。

型变包含顺变、逆变、协变，英文分别为，covariance、contra variance、invariant

再举个例子：

```rust
fn assign<T>(input: &mut T, val: T) {
    *input = val;
}

fn main() {
    let mut hello: &'static str = "hello";
    {
        let world = String::from("world");
        assign(&mut hello, &world);
    }
    println!("{hello}"); // use after free
}
```

In `assign`, we are setting the `hello` reference to point to `world`. But then `world` goes out of scope, before the later use of `hello` in the `println!`, leading to a classic use-after-free bug.

The fact is that we cannot assume that `&mut &'static str` and `&mut &'b str` are compatible. This means that `&mut &'static str` **cannot** be a _subtype_ of `&mut &'b str`, even if `'static` is a subtype of `'b`.

用前面的话来说，假如能够将`&mut &'static str`降级为`&mut &'b str`，也就可以将可变引用指向的`&'static str`变成`&'b str`，细想一下其中的危险。所以，我们认为`&mut &'static str`不是`&mut &'b str`的子类型。

因此，`&mut`是协变（invariant）的。

总结：

- `F` is **covariant** if `F<Sub>` is a subtype of `F<Super>` (the subtype property is passed through)
- `F` is **contravariant** if `F<Super>` is a subtype of `F<Sub>` (the subtype property is "inverted")
- `F` is **invariant** otherwise (no subtyping relationship exists)

常见型变与子类型：

|                 |    'a     |         T         |     U     |
| --------------- | :-------: | :---------------: | :-------: |
| `&'a T `        | covariant |     covariant     |           |
| `&'a mut T`     | covariant |     invariant     |           |
| `Box<T>`        |           |     covariant     |           |
| `Vec<T>`        |           |     covariant     |           |
| `UnsafeCell<T>` |           |     invariant     |           |
| `Cell<T>`       |           |     invariant     |           |
| `fn(T) -> U`    |           | **contra**variant | covariant |
| `*const T`      |           |     covariant     |           |
| `*mut T`        |           |     invariant     |           |

如何理解这个表：

以`fn(T) -> U`为例，当`T1: T2`时，`fn(T2) -> U: fn(T1) -> U`，为逆变；当`U1: U2`时，`fn(T) -> U1: fn(T) -> U2`，为顺变。

因为`T1: T2`，所以`fn(T1) -> U`对参数要求更严格，适用性比`fn(T2) -> U`低，所以前者是父类型，因此是逆变。

也因为这是唯一一个逆变的情形，所以用型变来理解借用检查实用性不高。

现在，我们可以再次细想前面的例子：

```rust
fn assign<T>(input: &mut T, val: T) {
    *input = val;
}

let mut hello: &'static str = "hello";
{
    let world = String::from("world");
    assign(&mut hello, &world);
}
println!("{hello}"); // use after free
```

`&mut`是 invariant 的，不能发生隐式的降级，故 `T`被推断为`&'static str`，但`&world`不符合要求，编译器报错。

---

### Polonius update && An alias-based formulation of the borrow checker

[An alias-based formulation of the borrow checker](https://smallcultfollowing.com/babysteps/blog/2018/04/27/an-alias-based-formulation-of-the-borrow-checker/)是在[Inside Rust Blog](https://blog.rust-lang.org/inside-rust/)的文章[Polonius update](https://blog.rust-lang.org/inside-rust/2023/10/06/polonius-update.html)中提到的。描述了当前Rust编译器中借用检查器的实现。

一切都是为了解决之前提到的一个问题：

```rust
// Compile Error
fn get_default<'m, K, V>(map: &'m mut HashMap<K, V>, key: K) -> &'m mut V
where
    K: Clone + Eq + Hash,
    V: Default,
{
    match map.get_mut(&key) {
        Some(value) => value,	// return 'm, so line 7's `&mut map: 'm`
        None => {
            map.insert(key.clone(), V::default());
            map.get_mut(&key).unwrap()	// return 'm, so line 11's `&mut map: 'm`
        }
    }
}
```

编译器认为在`'m`中，存在两个可变引用，于是拒绝编译。

以及这个：

```rust
struct Thing;

impl Thing {
    fn maybe_next(&mut self) -> Option<&mut Self> { None }
}

fn main() {
    let mut temp = &mut Thing;

    loop {
        match temp.maybe_next() {
            Some(v) => { temp = v; }
            None => { }
        }
    }
}
```

实际上，每次循环指向的`temp`都不相同，但编译器依然拒绝它。

感兴趣的同学可以阅读上面提到的两篇文章，新的借用检查器看起来是在顺利的开发中。

## 总结

很酷，累……

喜欢的话别忘了Star、三连、关注和分享，本人喜欢装逼。
