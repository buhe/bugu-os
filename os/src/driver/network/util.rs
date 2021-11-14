use core::slice;

use super::traits::Write;

/** Write quoted string. `\` and `"` are escaped, and the string
 * is automatically surrounded with double-quotes.
 */
pub fn write_qstr<W>(w: &mut W, s: &[u8]) -> Result<(), W::Error>
where
    W: Write,
{
    w.write_all(b"\"")?;
    for ch in s {
        w.write_all(match ch {
            b'\"' => &[b'\\', b'"'],
            b'\\' => &[b'\\', b'\\'],
            _ => slice::from_ref(ch),
        })?;
    }
    w.write_all(b"\"")?;
    Ok(())
}

/** Write decimal unsigned number */
pub fn write_num_u32<W>(w: &mut W, mut val: u32) -> Result<(), W::Error>
where
    W: Write,
{
    let mut buf = [0u8; 10];
    let mut curr = buf.len();
    for byte in buf.iter_mut().rev() {
        *byte = b'0' + (val % 10) as u8;
        val = val / 10;
        curr -= 1;
        if val == 0 {
            break;
        }
    }
    w.write_all(&buf[curr..])
}
