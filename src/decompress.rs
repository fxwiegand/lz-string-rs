use crate::constants::URI_KEY;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct DecompressContext<'a> {
    val: u32,
    compressed_data: &'a [u32],
    position: usize,
    index: usize,
    reset_val: usize,
}

impl<'a> DecompressContext<'a> {
    #[inline]
    pub fn new(compressed_data: &'a [u32], reset_val: usize) -> Self {
        // compressed_data.push(0 as char); // Js version seems to rely on being able to load a nonexistent byte, so just pad it here...? Maybe a bug in my impl?
        DecompressContext {
            val: compressed_data[0],
            compressed_data,
            position: reset_val,
            index: 1,
            reset_val,
        }
    }

    #[inline]
    pub fn read_bit(&mut self) -> Option<bool> {
        let res = self.val & (self.position as u32);
        self.position >>= 1;

        if self.position == 0 {
            self.position = self.reset_val;
            self.val = *self.compressed_data.get(self.index)?;
            self.index += 1;
        }

        Some(res != 0)
    }

    #[inline]
    pub fn read_bits(&mut self, n: usize) -> Option<u32> {
        let mut res = 0;
        let max_power = 2_u32.pow(n as u32);
        let mut power = 1;
        while power != max_power {
            res |= u32::from(self.read_bit()?) * power;
            power <<= 1;
        }

        Some(res)
    }
}

#[inline]
pub fn decompress_str(compressed: &[u32]) -> Option<String> {
    decompress(&compressed, 16)
}

#[inline]
pub fn decompress_uri(compressed: &[u32]) -> Option<String> {
    // let compressed = compressed.replace(" ", "+"); //Is this even necessary?
    let compressed: Option<Vec<u32>> = compressed
        .iter()
        .map(|c| {
            URI_KEY
                .bytes()
                .position(|k| u8::try_from(*c) == Ok(k))
                .map(|n| u32::try_from(n).ok())
        })
        .flatten()
        .collect();
    decompress(&compressed?, 6)
}

/// # Panics
/// Panics if `bits_per_char` is greater than the number of bits in a `u32`.
#[inline]
pub fn decompress(compressed: &[u32], bits_per_char: usize) -> Option<String> {
    assert!(bits_per_char <= std::mem::size_of::<u32>() * 8);

    if compressed.is_empty() {
        return Some(String::new());
    }

    let reset_val_pow = u32::try_from(bits_per_char).ok()? - 1;
    let reset_val = 2_usize.pow(reset_val_pow);
    let mut ctx = DecompressContext::new(compressed, reset_val);
    let mut dictionary: Vec<String> = Vec::new();
    for i in 0_u8..3_u8 {
        dictionary.push(char::from(i).to_string());
    }

    let next = ctx.read_bits(2)?;
    let first_entry = match next {
        0 | 1 => {
            let bits_to_read = (next * 8) + 8;
            let bits = ctx.read_bits(bits_to_read as usize)?;
            std::char::from_u32(bits)?
        }
        2 => return Some(String::new()),
        _ => return None,
    };
    dictionary.insert(3, first_entry.to_string());

    let mut w = first_entry.to_string();
    let mut result = first_entry.to_string();
    let mut num_bits = 3;
    let mut enlarge_in = 4;
    let mut dict_size = 4;
    let mut entry;
    loop {
        let mut cc = ctx.read_bits(num_bits)? as usize;
        match cc {
            0 | 1 => {
                let bits_to_read = (cc * 8) + 8;
                // if cc == 0 {
                // if (errorCount++ > 10000) return "Error"; // TODO: Error logic
                // }

                let bits = ctx.read_bits(bits_to_read as usize)?;
                let c = std::char::from_u32(bits)?;
                dictionary.insert(dict_size, c.to_string());
                dict_size += 1;
                cc = dict_size - 1;
                enlarge_in -= 1;
            }
            2 => {
                return Some(result);
            }
            _ => {}
        }

        if enlarge_in == 0 {
            enlarge_in = 2_u32.pow(num_bits as u32);
            num_bits += 1;
        }

        if let Some(entry_value) = dictionary.get(cc as usize) {
            entry = entry_value.clone();
        } else if cc == dict_size {
            entry = w.clone();
            entry.push(w.chars().next()?);
        } else {
            return None;
        }

        result += &entry;

        // Add w+entry[0] to the dictionary.
        let mut to_be_inserted = w.clone();
        to_be_inserted.push(entry.chars().next()?);
        dictionary.insert(dict_size, to_be_inserted);
        dict_size += 1;
        enlarge_in -= 1;

        w = entry;

        if enlarge_in == 0 {
            enlarge_in = 2_u32.pow(num_bits as u32);
            num_bits += 1;
        }
    }
}
