# FramePicker

`FramePicker` is a Rust library for handling network frame data, designed to solve problems similar to TCP/UART packet sticking.

## Features

- Provides a `FramePicker` structure for storing and processing network frame data.
- Provides a `FrameMeta` trait for defining the metadata of network frames.

## Usage

First, define a type that implements the `FrameMeta` trait. Then, create a `FramePicker` instance and use it to process network frame data.

```rust
let mut picker = FramePicker::<500, Iap2FrameMeta>::new();
let data = [0xff, 0x5a, 0x00, 0x0a, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a];
picker.feed_data(&data).unwrap();
assert!(picker.contain_frame());
assert!(picker.frame_complete());
```

## Testing

Use the `cargo test` command to run test cases.

## Contribution

PRs and issues are welcome.

## License

[MIT](LICENSE)