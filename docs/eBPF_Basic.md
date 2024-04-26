# Rust && eBPF

## 参考文献

- [官方文档](https://ebpf.io/what-is-ebpf/#the-power-of-programmability)
- [Learn eBPF Tracing: Tutorial and Examples](https://www.brendangregg.com/blog/2019-01-01/learn-ebpf-tracing.html)
- [Kevin.K's tutorial](https://kbknapp.dev/ebpf-part-ii/index.html)

## 正文

定义：**eBPF** does to Linux what JavaScript VM does to Browser.

作用：With eBPF, instead of a fixed kernel, you can now write mini programs that run on events like disk I/O and run in a safe virtual machine in the kernel.



BPF 程序需要编译，由高级语言编译为 bytecode，由内核中的虚拟机 JIT 运行，这和 Java 是一样的。

其框架并未集成进 Linux，而是在 Linux Foundation 的 [iovisor](https://github.com/iovisor)项目中。



优点：高效和安全。（interest 驱动）

已有的 BPF 工具：

[![img](assets/bcc_tracing_tools.png)](http://www.brendangregg.com/Perf/bcc_tracing_tools.png)
