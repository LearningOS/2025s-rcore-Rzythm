# 实现功能

使用`SyscallCounter`结构来记录`syscall`次数

为什么不用数组：因为在每个进程中开512的数组会爆栈

根据`TaskManager`中的`TaskManagerInner`中的`TaskControlBlock`中的`SyscallCounter`一步一步嵌套实现针对五个`syscall`系统调用最简化的计数

# 知识

术语 XLEN 来指代整数寄存器的位宽（32 或 64）

## `csr` (Control and Status Registers)

### `CSRRW`(原子读/写CSR)

> 以原子指令操作方式交换CSR与整数寄存器中的值。

1. 读取CSR的旧值
2. 将其扩展至XLEN位宽后写入整数寄存器rd
3. 再将rs1中的初始值写入CSR

### `CSRRS`(原子性读取并设置CSR中的位)

> 读取CSR的值，将其零扩展至XLEN位宽后写入整数寄存器rd

- 整数寄存器rs1的初始值被视为一个位掩码，用于指定CSR中需要设置的位。
- 若 rs1 中某位为高电平且 CSR 对应位可写，则该 CSR 位将被置位。


### `CSRRC`(原子性读取并清除CSR中的位)

> 读取CSR的值，将其零扩展至XLEN位宽后写入整数寄存器rd

- 整数寄存器 rs1 的初始值被视为一个位掩码，用于指定 CSR 中需要清除的位。
- 若 rs1 中某位为高电平且 CSR 对应位可写，则该 CSR 位将被清零。



# 简答题

## 1. 正确进入 U 态后，程序的特征还应有：使用 S 态特权指令，访问 S 态寄存器后会报错。 请同学们可以自行测试这些内容（运行 [三个 bad 测例 (ch2b_bad_*.rs)](https://github.com/LearningOS/rCore-Tutorial-Test-2025S/tree/master/src/bin) ）， 描述程序出错行为，同时注意注明你使用的 sbi 及其版本。

- **SBI版本信息**：
	- RustSBI version 0.3.0-alpha.2，适配 RISC-V SBI v1.0.0
	- 平台实现为：RustSBI-QEMU Version 0.2.0-alpha.2

测试输出结果
```rust
[kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x804003a4, kernel killed it.
[kernel] IllegalInstruction in application, kernel killed it.
[kernel] IllegalInstruction in application, kernel killed it.
```

- 程序尝试访问空指针0x0，触发 PageFault
- 程序在 U 态中尝试访问 S 态寄存器 `sstatus`，或执行 `sret` 指令，触发 IllegalInstruction。
- 内核捕获异常并正常终止该应用程序，验证了 U/S 态权限隔离机制正常。


## 深入理解 [trap.S](https://github.com/LearningOS/rCore-Tutorial-Code-2025S/blob/ch3/os/src/trap/trap.S) 中两个函数 `__alltraps` 和 `__restore` 的作用，并回答如下问题:

### 1. L40：刚进入 `__restore` 时，`sp` 代表了什么值。请指出 `__restore` 的两种使用情景。

sp是**当前内核栈顶**

两种使用情况：
1. 用户态返回
2. 内核态手动调度用户任务


### 2.L43-L48：这几行汇编代码特殊处理了哪些寄存器？这些寄存器的的值对于进入用户态有何意义？请分别解释

```asm
ld t0, 32*8(sp)      # 恢复 sstatus
ld t1, 33*8(sp)      # 恢复 sepc
ld t2, 2*8(sp)       # 恢复 sscratch（也就是用户态 sp）
csrw sstatus, t0     # 设置 U 态程序所需的状态（比如 SPP/U mode）
csrw sepc, t1        # 设置 U 态返回地址
csrw sscratch, t2    # 把用户栈 sp 恢复备用
```


### 3.L50-L56：为何跳过了 `x2` 和 `x4`？

```asm
ld x1, 1*8(sp)       # 恢复 ra
ld x3, 3*8(sp)       # 恢复 gp
```

x2/sp 在最后通过`csrrw sp, sscratch, sp`恢复，不需要提前恢复
x4/tp 一般用户程序不会用，或由运行时库初始化，不强制保存和恢复


### 4.L60：该指令之后，`sp` 和 `sscratch` 中的值分别有什么意义？

```asm
csrrw sp, sscratch, sp   #交换sp和sscratch的值
```

交换前：
- `sp`是内核栈
- `sscratch`是用户栈

执行后：
- `sp`被恢复成了用户栈
- `sscratch`存放内核栈指针




### 5.`__restore`：中发生状态切换在哪一条指令？为何该指令执行之后会进入用户态？


`sret`指令

#### **原因：**

- sret 根据 `sstatus.SPP` 位决定跳转到哪个特权级（SPP 为 0 表示跳到 U 态）。
- 会跳转到 `sepc` 指定的地址继续执行。
- 所以 `sstatus, sepc` 都必须提前设置好。


### 6.L13：该指令之后，`sp` 和 `sscratch` 中的值分别有什么意义？

这是 U 态发生 trap 后，S 态第一条指令。

**执行后含义：**

- sp 从原来的用户栈指针，换成了内核栈（为保存上下文准备空间）
- `sscratch` 暂存了用户态的 sp，之后用于恢复


### 7.从 U 态进入 S 态是哪一条指令发生的？

```asm
ecall       # 执行系统调用
非法指令    # 例如 `csrr sstatus` in U mode
页面异常    # 访问非法地址
```

这些会自动触发从 U 态切换到 S 态，并进入 \__alltraps。


