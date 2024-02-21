use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hmac::{Hmac, Mac};
use sha2::Sha256;

pub const fn u8_ms_bit(value: u8, index: u8) -> bool {
    if index > 7 {
        panic!("unsupported position")
    }
    (value << (index) >> 7) != 0
}
#[test]
fn position() {
    assert!(u8_ms_bit(0b1, 7));
    assert!(!u8_ms_bit(0b1, 6));
    assert!(!u8_ms_bit(0b10, 7));
    assert!(u8_ms_bit(0b10, 6));

    assert!(u8_ms_bit(0b1000_0000, 0));
    assert!(!u8_ms_bit(0b1000_0000, 7));
}

/**
 * @brief Ipmb misc
 */
// const IPMB_LUN_MASK: u8 = 0x03;

// pub const fn ipmb_netfn_lun_set(netfn: u8, lun: u8) -> u8 {
//     (netfn << 2) | (lun & IPMB_LUN_MASK)
// }

// pub const fn ipmb_seq_lun_set(netfn: u8, lun: u8) -> u8 {
//     (netfn << 2) | (lun & IPMB_LUN_MASK)
// }

// two's complement sum
pub fn checksum(input: &[u8]) -> u8 {
    let mut res = 0_i32;
    for val in input {
        res += *val as i32
    }
    (-res) as u8
}

fn pad_payload_bytes(data: &mut Vec<u8>) -> Vec<u8> {
    let length = &data.len();
    if length % 16 == 0 {
        data.to_vec()
    } else {
        let padding_needed = 16 - (length % 16);
        for i in 1..padding_needed {
            data.push(i.try_into().unwrap());
        }
        data.push((padding_needed - 1).try_into().unwrap());
        data.to_vec()
    }
}

pub fn hash_hmac_sha_256(key: Vec<u8>, data: Vec<u8>) -> [u8; 32] {
    type HmacSha256 = Hmac<Sha256>;
    let mut mac =
        HmacSha256::new_from_slice(key.as_slice()).expect("HMAC can take key of any size");
    mac.update(data.as_slice());
    let result = mac.finalize();
    let mut vec_bytes = [0; 32];
    for (index, i) in result.into_bytes().into_iter().enumerate() {
        vec_bytes[index] = i;
    }
    vec_bytes
}

pub fn generate_iv() -> [u8; 16] {
    let mut iv = [0; 16];
    for value in &mut iv {
        *value = rand::random::<u8>();
    }
    iv
}

pub fn aes_128_cbc_encrypt(key: [u8; 16], iv: [u8; 16], mut payload_bytes: Vec<u8>) -> Vec<u8> {
    type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
    let binding = pad_payload_bytes(&mut payload_bytes);
    let plaintext = binding.as_slice();
    // println!("encrypting this data: {:x?}", &plaintext);
    // encrypt in-place
    // buffer must be big enough for padded plaintext
    let mut buf = [0u8; 48];
    let pt_len = plaintext.len();
    buf[..pt_len].copy_from_slice(plaintext);
    let mut binding = buf;
    let ct = Aes128CbcEnc::new(&key.into(), &iv.into())
        .encrypt_padded_mut::<aes::cipher::block_padding::NoPadding>(&mut binding, pt_len)
        .unwrap();
    ct.to_vec()
}

pub fn aes_128_cbc_decrypt(key: [u8; 16], iv: [u8; 16], encrypted_bytes: Vec<u8>) -> Vec<u8> {
    let mut old_encrypted = encrypted_bytes.clone();
    type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;
    let ct = Aes128CbcDec::new(&key.into(), &iv.into())
        .decrypt_padded_mut::<aes::cipher::block_padding::NoPadding>(&mut old_encrypted)
        .unwrap()
        .to_vec();
    // structure of these packets is [[payload x bytes],[padding (1, 2, 3, 4, ...)], padding_length]
    let number_of_padded_bytes: usize = ct[ct.len() - 1].into();
    ct[..(ct.len() - (number_of_padded_bytes + 1))].to_vec()
}

pub fn append_u32_to_vec(main_vec: &mut Vec<u8>, append: u32) {
    append.to_le_bytes().map(|byte| main_vec.push(byte));
}

pub fn append_u128_to_vec(main_vec: &mut Vec<u8>, append: u128) {
    append.to_le_bytes().map(|byte| main_vec.push(byte));
}
