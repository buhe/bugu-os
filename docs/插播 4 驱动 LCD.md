### 目标

替换 println! 宏，打印到 LCD 上。

### 思路

1. LCD 驱动
2. 实现基于 LCD 的 console. console 实现 Writer 。

### LCD 驱动

我这块小 LCD 的驱动芯片是 st7789v ，比较简单。物理连接好，注意不要插反了，就可以编程了。

定义了 LCD ,LCDHL 和 LCDLL 。后面两个一个是高级接口，一个是底层接口。都来看看：

``` rust
pub struct LCD<SPI> {
    spi: SPI,
    spi_cs: u32,
    dcx_gpionum: u8,
    rst_gpionum: u8,
    dmac: DMAC,
    channel: dma_channel,
    pub width: u16,
    pub height: u16,
}

/** Low-level interface */
pub trait LCDLL {
    fn hard_init(&self);
    fn write_command(&self, cmd: command);
    /** Write bytes. These are provided as 32-bit units (ignoring the upper 24 bits) for efficient DMA */
    fn write_byte(&self, data_buf: &[u32]);
    /** Write 32-bit words. */
    fn write_word(&self, data_buf: &[u32]);
    fn fill_data(&self, data: u32, length: usize);
}

/** High-level interface */
pub trait LCDHL {
    /** Turn on and initialize the LCD display, this needs to be called before it's possible to use it. */
    fn init(&mut self);
    /** Set direction/alignment of display. It can be rotated and/or mirrored in every direction. */
    fn set_direction(&mut self, dir: direction);
    /** Clear the screen to a single RGB565 color. */
    fn clear(&self, color: u16);
    /** Draw a picture, filling the entire screen or part of it. `data` packs two RGB565 pixels
     * per u32 as 0xBBBBAAAA. */
    fn draw_picture(&self, x1: u16, y1: u16, width: u16, height: u16, data: &[u32]);
    /** Shut down and turn off the screen. */
    fn shutdown(&mut self);
}
```

LCD 用的是 SPI 总线，

