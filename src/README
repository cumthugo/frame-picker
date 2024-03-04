# FramePicker

`FramePicker` 是一个用于处理网络帧数据的 Rust 库。

## 功能

- 提供了一个 `FramePicker` 结构体，用于存储和处理网络帧数据。
- 提供了一个 `FrameMeta` trait，用于定义网络帧的元数据。

## 使用

首先，定义一个实现了 `FrameMeta` trait 的类型。然后，创建一个 `FramePicker` 实例，并使用它来处理网络帧数据。

```rust
let mut picker = FramePicker::<500, Iap2FrameMeta>::new();
let data = [0xff, 0x5a, 0x00, 0x0a, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a];
picker.feed_data(&data).unwrap();
assert!(picker.contain_frame());
assert!(picker.frame_complete());
```

## 测试

使用 `cargo test` 命令来执行测试用例。

## 贡献

欢迎提交 PR 和 issue。

## 许可证

[MIT](LICENSE)