use crate::frame_meta::FrameMeta;

#[derive(Debug, Default)]
pub struct Iap2FrameMeta { }

impl FrameMeta for Iap2FrameMeta {
    fn frame_header_len() -> usize {
        6
    }

    fn frame_match(data : &[u8]) -> bool {
        if data.len() < Self::frame_header_len() {
            return false;
        }
        match (data[0], data[1], data[4]) {
            (0xff, 0x5a, _) => true,
            (0xff, 0x55, 0xee) => true,
            _ => false,
        }
    }

    fn frame_totol_len(data: &[u8]) -> usize {
        if Self::frame_match(data) {
            match (data[0], data[1]) {
                (0xff, 0x55) => 6,
                (0xff, 0x5a) => {
                    ((((data[2] as u16) << 8) | (data[3] as u16)) as usize).into()
                },
                _ => 0,
            }
        } else {
            0
        }
    }
}