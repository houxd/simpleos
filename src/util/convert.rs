use anyhow::{Result, anyhow};

pub fn bcd_to_dec(bcd: u8) -> Result<u8> {
    let high = (bcd >> 4) & 0x0F;
    let low = bcd & 0x0F;

    if high > 9 || low > 9 {
        return Err(anyhow!("CNV BAD BCD"));
    }

    Ok(high * 10 + low)
}

pub fn dec_to_bcd(dec: u8) -> Result<u8> {
    if dec > 99 {
        return Err(anyhow!("CNV BAD DEC"));
    }

    Ok(((dec / 10) << 4) | (dec % 10))
}
