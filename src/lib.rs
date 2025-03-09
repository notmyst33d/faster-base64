mod tables;

use crate::tables::*;

pub fn encode(data: &[u8]) -> String {
    let remainder = data.len() % 3;
    let out_size = if remainder == 0 {
        (data.len() / 3) * 4
    } else {
        ((data.len() / 3) * 4) + 4
    };

    let mut out = vec![0u8; out_size];

    if data.len() - 3 > 0 {
        unsafe {
            lcvec_enc(
                data,
                std::slice::from_raw_parts_mut(out.as_mut_ptr() as *mut u32, out.len() / 2),
            );
        }
    }

    let last_chunk_pos = if remainder == 0 {
        data.len() - 3
    } else {
        data.len() - remainder
    };
    let out_chunk_pos = out_size - 4;

    if remainder == 1 {
        let value = (data[last_chunk_pos] as usize) << 16;
        out[out_chunk_pos] = TABLE[value >> 18 & 0x3f] as u8;
        out[out_chunk_pos + 1] = TABLE[value >> 12 & 0x3f] as u8;
        out[out_chunk_pos + 2] = b'=';
        out[out_chunk_pos + 3] = b'=';
    } else if remainder == 2 {
        let value =
            (data[last_chunk_pos] as usize) << 16 | (data[last_chunk_pos + 1] as usize) << 8;
        out[out_chunk_pos] = TABLE[value >> 18 & 0x3f] as u8;
        out[out_chunk_pos + 1] = TABLE[value >> 12 & 0x3f] as u8;
        out[out_chunk_pos + 2] = TABLE[value >> 6 & 0x3f] as u8;
        out[out_chunk_pos + 3] = b'=';
    } else {
        let value = (data[last_chunk_pos] as usize) << 16
            | (data[last_chunk_pos + 1] as usize) << 8
            | data[last_chunk_pos + 2] as usize;
        out[out_chunk_pos] = TABLE[value >> 18 & 0x3f] as u8;
        out[out_chunk_pos + 1] = TABLE[value >> 12 & 0x3f] as u8;
        out[out_chunk_pos + 2] = TABLE[value >> 6 & 0x3f] as u8;
        out[out_chunk_pos + 3] = TABLE[value & 0x3f] as u8;
    }

    unsafe { String::from_utf8_unchecked(out) }
}

/// Large Chunk Vectorization decoder
pub unsafe fn lcvec_dec(s: &[u16], out: &mut [u8]) {
    let i_max = s.len() - 2;
    let j_max = (s.len() / 2) * 3;
    for (i, j) in (0..i_max).step_by(2).zip((0..j_max).step_by(3)) {
        let value = (*REVERSE_PAIR_TABLE.get_unchecked(*s.get_unchecked(i) as usize) as u32) << 12
            | *REVERSE_PAIR_TABLE.get_unchecked(*s.get_unchecked(i + 1) as usize) as u32;
        *out.get_unchecked_mut(j) = (value >> 16) as u8;
        *out.get_unchecked_mut(j + 1) = (value >> 8) as u8;
        *out.get_unchecked_mut(j + 2) = value as u8;
    }
}

/// Large Chunk Vectorization encoder
pub unsafe fn lcvec_enc(s: &[u8], out: &mut [u32]) {
    let i_max = s.len() - 3;
    let j_max = s.len() / 3;
    for (i, j) in (0..i_max).step_by(3).zip(0..j_max) {
        let value = (*s.get_unchecked(i) as usize) << 16
            | (*s.get_unchecked(i + 1) as usize) << 8
            | *s.get_unchecked(i + 2) as usize;
        *out.get_unchecked_mut(j) = *PAIR_TABLE.get_unchecked(value >> 12 & 0xfff) as u32
            | (*PAIR_TABLE.get_unchecked(value & 0xfff) as u32) << 16;
    }
}

/// Decodes Base64, uses `data` as a buffer for in-place decoding.
///
/// Returns a slice of `data` with decoded data.
pub fn decode(data: &mut [u8]) -> &[u8] {
    if data.len() < 4 {
        return &[];
    }

    if data.len() - 4 > 0 {
        unsafe {
            lcvec_dec(
                std::slice::from_raw_parts(data.as_ptr() as *const u16, data.len() / 2),
                data,
            );
        }
    }

    let last_chunk_pos = data.len() - 4;
    let out_chunk_pos = ((data.len() - 4) / 4) * 3;
    let value = REVERSE_TABLE[data[last_chunk_pos] as usize] << 18
        | REVERSE_TABLE[data[last_chunk_pos + 1] as usize] << 12
        | REVERSE_TABLE[data[last_chunk_pos + 2] as usize] << 6
        | REVERSE_TABLE[data[last_chunk_pos + 3] as usize];

    let last_chunk_len = if data[last_chunk_pos + 2] == b'=' {
        1
    } else if data[last_chunk_pos + 3] == b'=' {
        2
    } else {
        3
    };

    data[out_chunk_pos..out_chunk_pos + last_chunk_len]
        .clone_from_slice(&value.to_be_bytes()[1..last_chunk_len + 1]);

    &data[..out_chunk_pos + last_chunk_len]
}

#[cfg(test)]
mod tests {
    use crate as faster_base64;

    #[test]
    fn encode() {
        assert_eq!(faster_base64::encode(b"Hello"), "SGVsbG8=");
    }

    #[test]
    fn encode_chunk_equal() {
        assert_eq!(faster_base64::encode(b"Hel"), "SGVs");
    }

    #[test]
    fn encode_chunk_long() {
        assert_eq!(
            faster_base64::encode(b"HelHelHelHelHelHelHelHel"),
            "SGVsSGVsSGVsSGVsSGVsSGVsSGVsSGVs"
        );
    }

    #[test]
    fn encode_chunk_remain_1() {
        assert_eq!(faster_base64::encode(b"Hell"), "SGVsbA==");
    }

    #[test]
    fn decode() {
        assert_eq!(
            faster_base64::decode(&mut "SGVsbG8=".as_bytes().to_vec()),
            b"Hello"
        );
    }

    #[test]
    fn decode_nothing() {
        assert_eq!(faster_base64::decode(&mut "".as_bytes().to_vec()), b"");
    }

    #[test]
    fn decode_less_than_4() {
        assert_eq!(faster_base64::decode(&mut "AAA".as_bytes().to_vec()), b"");
    }

    #[test]
    fn decode_chunk_equal() {
        assert_eq!(
            faster_base64::decode(&mut "SGVs".as_bytes().to_vec()),
            b"Hel"
        );
    }

    #[test]
    fn decode_chunk_long() {
        assert_eq!(
            faster_base64::decode(&mut "SGVsSGVsSGVsSGVsSGVsSGVsSGVsSGVs".as_bytes().to_vec()),
            b"HelHelHelHelHelHelHelHel"
        );
    }

    #[test]
    fn decode_chunk_remain_1() {
        assert_eq!(
            faster_base64::decode(&mut "SGVsbA==".as_bytes().to_vec()),
            b"Hell"
        );
    }
}
