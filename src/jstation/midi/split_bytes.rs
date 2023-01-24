#[track_caller]
pub fn to_bool(sb: &[u8]) -> bool {
    sb[1] != 0
}

pub fn from_u8(val: u8) -> [u8; 2] {
    [val >> 7, val & 0x7f]
}

#[track_caller]
pub fn to_u8(sb: &[u8]) -> u8 {
    (sb[0] << 7) + sb[1]
}

pub fn from_u16(val: u16) -> [u8; 4] {
    let lsb = (val & 0xff) as u8;
    let msb = (val >> 8) as u8;

    [lsb >> 7, lsb & 0x7f, msb >> 7, msb & 0x7f]
}

#[track_caller]
pub fn to_u16(sb: &[u8]) -> u16 {
    (((sb[0] << 7) + sb[1]) as u16) + ((sb[2] as u16) << 15) + ((sb[3] as u16) << 8)
}

#[cfg(test)]
mod tests {
    #[test]
    fn to_u8() {
        assert_eq!(super::to_u8(&[0, 0]), 0);
        assert_eq!(super::to_u8(&[0, 1]), 1);
        assert_eq!(super::to_u8(&[0, 8]), 8);
        assert_eq!(super::to_u8(&[1, 8]), 0x88);
    }

    #[test]
    fn from_u8() {
        assert_eq!(super::from_u8(0), [0, 0]);
        assert_eq!(super::from_u8(1), [0, 1]);
        assert_eq!(super::from_u8(8), [0, 8]);
        assert_eq!(super::from_u8(0x88), [1, 8]);
    }
}
