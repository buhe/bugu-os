启用了虚拟内存之后，访问 MMIO 也需要建立映射，在 memory_sets.rs 中

```rust
for pair in MMIO {
            memory_set.push(MapArea::new(
                (*pair).0.into(),
                ((*pair).0 + (*pair).1).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ), None);
        }
```



实际的 pair 在 config.rs 中，pair.0 是开始地址，pair.1 是范围

```rust
pub const MMIO: &[(usize, usize)] = &[
    // we don't need clint in S priv when running
    // we only need claim/complete for target0 after initializing
    (0x0C00_0000, 0x3000),      /* PLIC      */
    (0x0C20_0000, 0x1000),      /* PLIC      */
    (0x3800_0000, 0x1000),      /* UARTHS    */
    (0x3800_1000, 0x1000),      /* GPIOHS    */
    (0x5020_0000, 0x1000),      /* GPIO      */
    (0x5024_0000, 0x1000),      /* SPI_SLAVE */
    (0x502B_0000, 0x1000),      /* FPIOA     */
    (0x502D_0000, 0x1000),      /* TIMER0    */
    (0x502E_0000, 0x1000),      /* TIMER1    */
    (0x502F_0000, 0x1000),      /* TIMER2    */
    (0x5044_0000, 0x1000),      /* SYSCTL    */
    (0x5200_0000, 0x1000),      /* SPI0      */
    (0x5300_0000, 0x1000),      /* SPI1      */
    (0x5400_0000, 0x1000),      /* SPI2      */
];
```

### 板载 led

首先用高速 GPIO(GPIOHS) 点亮板载 led

```rust
use k210_soc::{
    fpioa::{self, io},
    gpio, gpiohs,
};

pub fn init() {
    fpioa::set_function(io::LED_B, fpioa::function::GPIOHS0);
    gpiohs::set_direction(0, gpio::direction::OUTPUT);
    gpiohs::set_pin(0, false);
    println!("0 is {}", gpiohs::get_pin(0));
}
```

1. fpio 映射实际物理引脚(io::LED_B) 到 GPIOHS0(fpioa::function::GPIOHS0) ，也就是高速 GPIO 0 号
2. 设置高速 GPIO 0 号输出
3. 设置高速 GPIO 0 号低电平



![IMG_1402_副本](https://tva1.sinaimg.cn/large/008i3skNgy1gvcmletdejj60bs0aojsd02.jpg)

### 外部的 led

我们看看怎么点亮外部的 led ，led 点亮的原理就是**正负极高低电平**

```rust
// 外部的 led

use k210_soc::{
    fpioa::{self, io},
    gpio, gpiohs,
};

pub fn init() {
    // led b 映射到 gpiohs 0
    // 9
    fpioa::set_function(io::BPSK_P, fpioa::function::GPIOHS0);
    // 10
    fpioa::set_function(io::BPSK_N, fpioa::function::GPIOHS1);
    // gpiohs 设置 0 为输出
    gpiohs::set_direction(0, gpio::direction::OUTPUT);
    gpiohs::set_direction(1, gpio::direction::OUTPUT);

    gpiohs::set_pin(0, false);
    gpiohs::set_pin(1, true);
    
    println!("0 is {}", gpiohs::get_pin(0));
    println!("1 is {}", gpiohs::get_pin(1));
}
```

1. 分别映射 9 10 两个引脚到 0 1
2. 0 1 设置为输出
3. 0 1 分别设置为低电平和高电平
4. 插入 led，注意正负极

![图像](https://tva1.sinaimg.cn/large/008i3skNgy1gvg5x9jzl7j60u0140dn202.jpg)

代码请参考：https://github.com/buhe/bugu/tree/9