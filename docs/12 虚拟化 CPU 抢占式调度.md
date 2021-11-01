为什么使用抢占式调度？协作就像靠自觉，抢占就像排队，排队整体效率提高了，虽然牺牲了个人利益。

抢占式调度采用时钟中断实现

```rust
pub fn get_time() -> usize {
    time::read()
}

pub fn get_time_ms() -> usize {
    time::read() / (CLOCK_FREQ / MSEC_PER_SEC)
}

pub fn set_next_trigger() {
    set_timer(get_time() + CLOCK_FREQ / TICKS_PER_SEC);
}
```

