mod tables;

use crate::tables::*;

pub fn encode(data: &[u8]) -> String {
    let len = data.len();
    let data_ptr = data.as_ptr();
    let mut sb = String::with_capacity(len);
    let mut i = 0;
    loop {
        if i > len - 1 {
            break;
        }
        let available_bytes = len - i;
        let mut v = (unsafe { (data_ptr.add(i) as *const u32).read() }).swap_bytes() as usize;
        if available_bytes == 2 {
            v &= 0xffff0000;
            sb.push_str(TABLE[v >> 26 & 0x3f]);
            sb.push_str(TABLE[v >> 20 & 0x3f]);
            sb.push_str(TABLE[v >> 14 & 0x3f]);
            sb.push_str("=");
            break;
        } else if available_bytes == 1 {
            v &= 0xff000000;
            sb.push_str(TABLE[v >> 26 & 0x3f]);
            sb.push_str(TABLE[v >> 20 & 0x3f]);
            sb.push_str("=");
            sb.push_str("=");
            break;
        } else {
            sb.push_str(TABLE[v >> 26 & 0x3f]);
            sb.push_str(TABLE[v >> 20 & 0x3f]);
            sb.push_str(TABLE[v >> 14 & 0x3f]);
            sb.push_str(TABLE[v >> 8 & 0x3f]);
            i += 3;
        }
    }
    sb
}

/// Large chunk auto-vectorization
pub unsafe fn lcvec(s: &[u16], out: &mut [u8]) {
    let max_size = s.len() - 2;
    let mut j = 0;
    for i in (0..max_size).step_by(2) {
        let value = (*REVERSE_PAIR_TABLE.get_unchecked(*s.get_unchecked(i) as usize) as u32) << 12
            | *REVERSE_PAIR_TABLE.get_unchecked(*s.get_unchecked(i + 1) as usize) as u32;
        *out.get_unchecked_mut(j) = (value >> 16) as u8;
        *out.get_unchecked_mut(j + 1) = (value >> 8) as u8;
        *out.get_unchecked_mut(j + 2) = value as u8;
        j += 3;
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
            lcvec(
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
