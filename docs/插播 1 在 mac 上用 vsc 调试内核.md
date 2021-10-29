## 在 mac 上用 vsc 调试内核

### 调试的原理



### 额外的硬件调试器



### 步骤

```
brew install libusb libusb-compat isl libmpc libftdi
```

| library       | description                                                  |
| ------------- | ------------------------------------------------------------ |
| isl           | Integer Set Library for the polyhedral model                 |
| libmpc        | library for the arithmetic of high precision complex numbers |
| libusb        | library for USB access                                       |
| libusb-compat | library for USB-JTAG access                                  |

- 安装 vsc 插件 metalcode-eu.darwin-riscv （这是插件 id）.
- 建立 tasks.json 内容
- 在 Makefile 添加目标，内容
- 输入 ⇧⌘B 触发任务

### gdb

```
/Users/buhe/.vscode/extensions/metalcode-eu.darwin-riscv-1.0.3/bin/riscv64-unknown-elf-gdb
```

1. gdb os elf
   1. target remote :3333
   2. load
   3. b rust_main
   4. j rust_main
   5. n 是单步，但不进入函数
   6. p 打印变量
   7. q 退出
