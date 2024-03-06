// We're not using the standard library
#![no_std]

/// Frame metadata module
pub mod frame_meta;
/// Iap2 frame metadata module
pub mod iap2_frame_meta;
use core::marker::PhantomData;

use frame_meta::FrameMeta;

/// An error type indicating that there are no frames available
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Empty;

/// An error type indicating that the storage is full
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Full;

/// A struct for storing and handling network frame data
#[derive(Debug)]
pub struct FramePicker<const N: usize, M: FrameMeta> {
    storage: [u8; N],
    read_at: usize,
    /// The number of bytes dropped
    pub dropped : usize,
    _marker : PhantomData<M>,
}

impl<const N: usize, M: FrameMeta> FramePicker<N, M> {
    /// Creates a new FramePicker
    pub fn new() -> Self {
        Self {
            storage: [0; N],
            read_at: 0,
            dropped: 0,
            _marker: PhantomData,
        }
    }

    /// Feeds data into the FramePicker
    /// Returns a Full error if the storage is full
    pub fn feed_data(&mut self, data: &[u8]) -> Result<usize, Full> {
        if self.read_at + data.len() > N {
            return Err(Full);
        }

        let len = data.len();
        self.storage[self.read_at..self.read_at + len].copy_from_slice(&data[..len]);
        self.read_at += len;
        self.align_buffer_with_header();
        Ok(len)
    }

    /// Aligns the buffer with the header
    fn align_buffer_with_header(&mut self) {
        let mut search_at: usize = 0;
        loop {
            if self.read_at - search_at < M::frame_header_len() {
                break;
            }
            if M::frame_match(&self.storage[search_at..self.read_at]) {
                break;
            } else {
                search_at += 1;
            }
        }
        if search_at > 0 {
            self.dropped += search_at;
            self.storage.copy_within(search_at..self.read_at, 0);
            self.read_at -= search_at;
        }
    }

    /// Checks if the FramePicker contains a frame
    pub fn contain_frame(&self) -> bool {
        if self.read_at < M::frame_header_len() {
            return false;
        }
        M::frame_match(self.storage[..self.read_at].as_ref())
    }

    /// Checks if the frame is complete
    pub fn frame_complete(&self) -> bool {
        if self.read_at < M::frame_header_len() {
            return false;
        }
        let total_len = M::frame_totol_len(self.storage[..self.read_at].as_ref());
        total_len > 0 && total_len <= self.read_at
    }

    /// Acquires a frame
    pub fn acquire_frame(&mut self) -> Result<&[u8], Empty> {
        if self.frame_complete() {
            let total_len = M::frame_totol_len(self.storage[..self.read_at].as_ref());
            let data = &self.storage[..total_len];
            Ok(data)
        } else {
            Err(Empty)
        }
    }

    /// Releases a frame
    pub fn release_frame(&mut self) -> Result<(), Empty> {
        if self.frame_complete() {
            let total_len = M::frame_totol_len(self.storage[..self.read_at].as_ref());
            self.storage.copy_within(total_len..self.read_at, 0);
            self.read_at -= total_len;
            self.align_buffer_with_header();
            Ok(())
        } else {
            Err(Empty)
        }
    }

    /// Dequeues a frame with a function
    pub fn dequeue_frame_with<F,R>(&mut self, f: F) -> Result<R, Empty>
    where
        F: FnOnce(&[u8]) -> R,
    {
        if let Ok(frame) = self.acquire_frame() {
            let result = f(frame);
            self.release_frame().unwrap();
            Ok(result)
        } else {
            Err(Empty)
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn picker() -> FramePicker::<500,iap2_frame_meta::Iap2FrameMeta> {
        FramePicker::<500,iap2_frame_meta::Iap2FrameMeta>::new()
    }

    #[test]
    fn test_feed_handshake() {
        let mut picker = picker();
        let shake = [0xff, 0x55, 0x02, 0x00, 0xee, 0x10];
        assert_eq!(picker.feed_data(&shake[..]), Ok(6));
        assert_eq!(picker.dropped, 0);
        assert_eq!(picker.read_at, 6);
        assert!(picker.contain_frame());
        assert_eq!(picker.frame_complete(), true);

        assert_eq!(picker.acquire_frame(), Ok(&shake[..]));
        assert_eq!(picker.release_frame(), Ok(()));
    }

    #[test]
    fn test_feed_normal_data() {
        let mut picker = picker();
        let normal_data = [0xff, 0x5a, 0x00, 0x0a, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a];
        assert_eq!(picker.feed_data(&normal_data[..]), Ok(10));
        assert_eq!(picker.dropped, 0);
        assert_eq!(picker.read_at, 10);
        assert!(picker.contain_frame());
        assert_eq!(picker.frame_complete(), true);

        assert_eq!(picker.acquire_frame(), Ok(&normal_data[..]));
        assert_eq!(picker.release_frame(), Ok(()));
    }

    #[test]
    fn test_feed_data_exceed_storage() {
        let mut picker = FramePicker::<10,iap2_frame_meta::Iap2FrameMeta>::new();
        let normal_data = [0xff, 0x5a, 0x00, 0x0b, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b];
        assert_eq!(picker.feed_data(&normal_data[..]), Err(Full));
        assert_eq!(picker.read_at, 0);
        assert_eq!(picker.contain_frame(), false);
        assert_eq!(picker.frame_complete(), false);
    }

    #[test]
    fn test_feed_data_with_dropped() {
        let mut picker = picker();
        let normal_data = [0x00, 0x00, 0xff, 0x5a, 0x00, 0x0a, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a];
        assert_eq!(picker.feed_data(&normal_data[..]), Ok(12));
        assert_eq!(picker.dropped, 2);
        assert_eq!(picker.read_at, 10);
        assert_eq!(picker.contain_frame(), true);
        assert_eq!(picker.frame_complete(), true);

        assert_eq!(picker.feed_data(&normal_data[..]), Ok(12));
        assert_eq!(picker.dropped, 2);
        assert_eq!(picker.read_at, 10+12);
        assert_eq!(picker.release_frame(), Ok(()));
        assert_eq!(picker.release_frame(), Ok(()));
    }

    #[test]
    fn test_short_frame() {
        let mut picker = picker();
        let normal_data = [0xff, 0x5a, 0x00, 0x0a, 0x05, 0x06, 0x07, 0x08, 0x09];
        assert_eq!(picker.feed_data(&normal_data[..]), Ok(9));
        assert_eq!(picker.dropped, 0);
        assert_eq!(picker.read_at, 9);
        assert!(picker.contain_frame());
        assert_eq!(picker.frame_complete(), false);

        assert_eq!(picker.feed_data(&[0x0a, 0xff, 0x5a]), Ok(3));
        assert_eq!(picker.frame_complete(), true);
        assert_eq!(picker.release_frame(), Ok(()));

        assert_eq!(picker.feed_data(&[0x00, 0x0a, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a]), Ok(8));
        assert_eq!(picker.frame_complete(), true);
    }

    #[test]
    fn test_header_not_complete() {
        let mut picker = picker();
        let data1 = [0x32,0x34,0x42,0x33,0x52,0x42, 0x32, 0x31, 0x93,0x32,0xff];
        let data2 = [0x5a,0x00,0x0a,0x05,0x06,0x07,0x08,0x09,0x0a];
        assert_eq!(picker.feed_data(&data1[..]), Ok(11));
        assert_eq!(picker.dropped, 6);
        assert_eq!(picker.contain_frame(), false);

        assert_eq!(picker.feed_data(&data2[..]), Ok(9));
        assert_eq!(picker.dropped, 10);
        assert_eq!(picker.contain_frame(), true);

    }

}
