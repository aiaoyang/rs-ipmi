use aes::cipher::{BlockDecryptMut, BlockEncryptMut, BlockSizeUser, KeyIvInit};

use crate::err::Error;

pub fn hash_hmac_sha1(key: &[u8], data: &[u8]) -> [u8; 20] {
    hmac_sha1::hmac_sha1(key, data)
}

pub fn generate_iv() -> [u8; 16] {
    let mut iv = [0; 16];
    for value in &mut iv {
        *value = rand::random::<u8>();
    }
    iv
}

pub fn aes_128_cbc_encrypt(mut payload_bytes: Vec<u8>, key: [u8; 16]) -> Vec<u8> {
    let iv = generate_iv();
    type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
    padding_data(&mut payload_bytes, Aes128CbcEnc::block_size());
    let data_len = payload_bytes.len();
    let ct = Aes128CbcEnc::new(&key.into(), &iv.into())
        .encrypt_padded_mut::<aes::cipher::block_padding::NoPadding>(&mut payload_bytes, data_len)
        .unwrap();
    let mut ret = iv.to_vec();
    ret.extend(ct);
    ret
}

pub fn aes_128_cbc_decrypt(encrypted_bytes: &mut [u8], key: [u8; 16]) -> Result<Vec<u8>, Error> {
    let iv: [u8; 16] = encrypted_bytes[..16].try_into()?;
    let old_encrypted = &mut encrypted_bytes[16..];
    type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;
    let ct = Aes128CbcDec::new(&key.into(), &iv.into())
        .decrypt_padded_mut::<aes::cipher::block_padding::NoPadding>(old_encrypted)?;
    // structure of these packets is [[payload x bytes],[padding (1, 2, 3, 4, ...)], padding_length]
    let number_of_padded_bytes: usize = ct[ct.len() - 1].into();
    Ok(ct[..(ct.len() - (number_of_padded_bytes + 1))].to_vec())
}

fn padding_data(data: &mut Vec<u8>, block_size: usize) {
    let origin_length = data.len();
    let m = (origin_length + 1) % block_size;
    let pad_len = if m != 0 {
        block_size - (m)
    } else {
        return;
    };

    for i in 0..pad_len {
        data.push(i as u8 + 1);
    }
    data.push(pad_len as u8);
}

// Trailer's source is the session header and payload
pub fn add_tailer(data: &mut Vec<u8>, k1: [u8; 20]) {
    // Session Trailer (Table 13-8)
    // +---------------+
    // | Integrity PAD |  n bytes
    // | Pad Length    |  1 byte
    // | Next Header   |  1 byte  (0x07)
    // | AuthCode      | 12 bytes
    // +---------------+
    let origin_len = data.len();
    let m = (origin_len + 2 + 12) % 4;
    let pad_len = if m != 0 { 4 - m } else { 0 };

    for _ in 0..pad_len {
        data.push(0xff);
    }
    data.push(pad_len as u8);

    // Next Header, Reserved in IPMI v2.0. Set to 07h for RMCP+ packets defined in this specification.
    data.push(0x07);

    let auth_code = hmac_sha1::hmac_sha1(&k1, &data[..origin_len + pad_len + 2]);
    data.extend(&auth_code[..12]);
}

#[test]
fn test_add_tailer() {
    let mut data = vec![0; 32];
    let k1 = [1_u8; 20];
    add_tailer(&mut data, k1);
    println!("{:?}", &data[(data.len() - 1 - 2 - 12)..]);
}

#[test]
fn encrypt_decrypt() {
    let key = [0; 16];
    let data = vec![1; 32];
    let mut ret = aes_128_cbc_encrypt(data.clone(), key);
    let decrypted_data = aes_128_cbc_decrypt(&mut ret[..], key);
    assert_eq!(data, decrypted_data.unwrap());
}
