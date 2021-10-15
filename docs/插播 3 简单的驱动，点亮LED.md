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

给 mmu 建立映射关系

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



### 板载 led



![IMG_1402_副本](https://tva1.sinaimg.cn/large/008i3skNgy1gvcmletdejj60bs0aojsd02.jpg)

### 外部的 led

![图像](https://tva1.sinaimg.cn/large/008i3skNgy1gvg5x9jzl7j60u0140dn202.jpg)