pub trait FrameMeta {
    fn frame_header_len() -> usize;
    fn frame_match(data : &[u8]) -> bool;
    fn frame_totol_len(data: &[u8]) -> usize;
}