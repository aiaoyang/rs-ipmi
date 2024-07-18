use aes::cipher::{
    block_padding::NoPadding, BlockDecryptMut, BlockEncryptMut, BlockSizeUser, KeyIvInit,
};

use crate::err::Error;

pub fn hash_hmac_sha1(key: &[u8], data: &[u8]) -> [u8; 20] {
    hmac_sha1::hmac_sha1(key, data)
}

pub fn generate_iv() -> [u8; 16] {
    let value = rand::random::<u128>();
    value.to_le_bytes()
}

pub fn aes_128_cbc_encrypt(data: &mut [u8], len: &mut usize, key: &[u8; 16]) {
    let iv = generate_iv();
    type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;

    padding_data(data, len, Aes128CbcEnc::block_size());
    match Aes128CbcEnc::new(key.into(), &iv.into()).encrypt_padded_mut::<NoPadding>(data, *len) {
        Ok(v) => *len = v.len(),
        Err(e) => {
            panic!("payload: {:?}, err: {e:?}", &data[..*len]);
        }
    }
    let mut i = *len;
    while i != 0 {
        i -= 1;
        data[i + iv.len()] = data[i];
    }
    data[0..iv.len()].copy_from_slice(&iv);
    *len += iv.len();
}

fn padding_data(data: &mut [u8], len: &mut usize, block_size: usize) {
    let m = (*len + 1) % block_size;
    let pad_len = if m != 0 {
        block_size - (m)
    } else {
        return;
    };

    for i in 1..=pad_len {
        data[*len] = i as u8;
        *len += 1;
    }

    data[*len] = pad_len as u8;
    *len += 1;
}

pub fn aes_128_cbc_decrypt(encrypted_bytes: &mut [u8], key: [u8; 16]) -> Result<&[u8], Error> {
    let iv: [u8; 16] = encrypted_bytes[..16].try_into()?;
    let old_encrypted = &mut encrypted_bytes[16..];
    type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;
    let ct = Aes128CbcDec::new(&key.into(), &iv.into())
        .decrypt_padded_mut::<aes::cipher::block_padding::NoPadding>(old_encrypted)?;
    // structure of these packets is [[payload x bytes],[padding (1, 2, 3, 4, ...)], padding_length]
    let number_of_padded_bytes: usize = ct[ct.len() - 1].into();
    Ok(&ct[..(ct.len() - (number_of_padded_bytes + 1))])
}
