extern crate base64;

use crustopals::problem11;
use crustopals::tools::*;

lazy_static! {
  pub static ref RANDOM_KEY: Vec<u8> = aes::generate_key();
  pub static ref APPEND_STR: String = "Um9sbGluJyBpbiBteSA1LjAKV2l0aCBteSByYWct\
dG9wIGRvd24gc28gbXkgaGFpciBjYW4gYmxvdwpUaGUgZ2lybGllcyBvbiBzdGFuZGJ5IHdhdmluZyB\
qdXN0IHRvIHNheSBoaQpEaWQgeW91IHN0b3A/IE5vLCBJIGp1c3QgZHJvdmUgYnkK".to_string();
}

pub fn aes_128_ecb_rand_key_oracle(mut message: Vec<u8>) -> Vec<u8> {
  message.extend(base64::decode(&APPEND_STR.to_string()).unwrap());
  aes::encrypt_message_ecb(&message, &RANDOM_KEY.to_vec())
}

pub fn discover_blocksize() -> usize {
  let test_byte: u8 = 65;
  let ciphertext_len1 =
    aes_128_ecb_rand_key_oracle(build_byte_vec(test_byte, 1)).len();
  let mut ciphertext_len2 =
    aes_128_ecb_rand_key_oracle(build_byte_vec(test_byte, 2)).len();
  let mut repeats = 2;
  while ciphertext_len2 <= ciphertext_len1 {
    repeats = repeats + 1;
    let pt = build_byte_vec(test_byte, repeats);
    ciphertext_len2 = aes_128_ecb_rand_key_oracle(pt).len();
  }
  ciphertext_len2 - ciphertext_len1
}

pub fn test_ecb() -> bool {
  let test_bytes =
    "two blocks of message repeating two blocks of message repeating "
      .to_string()
      .as_bytes()
      .to_vec();
  let ciphertext = aes_128_ecb_rand_key_oracle(test_bytes);
  let mode: String = problem11::aes_ecb_cbc_oracle(&ciphertext);
  "ecb".to_string() == mode
}

pub fn crack_the_aes_ecb_oracle() -> Vec<u8> {
  let block_size = discover_blocksize();
  let mut recovered_pt: Vec<u8> = vec![];
  let mut found_padding: bool = false;
  while !found_padding {
    // this is a bit hard to wrap your head around, but when the entire
    // plaintext has been recovered we'll start "recovering" padding. The first
    // byte of padding we recover is naturally 0x01 since we are recovering
    // bytes in the last position of a block (and everything before it will be
    // actual plaintext). There may be a subtle bug here if the plaintext had a
    // literal 0x01 in it.. but we can cross that bridge if we come to it.
    if let Some(&1u8) = recovered_pt.last() {
      found_padding = true;
      recovered_pt.pop();
    } else {
      let mut oracle_msg =
        build_byte_vec(65, block_size - 1 - (recovered_pt.len() % block_size));
      let ct = &aes_128_ecb_rand_key_oracle(oracle_msg.clone());
      let blk_num = recovered_pt.len() / block_size;
      let relevant_ct_block =
        &ct[(blk_num * block_size)..((blk_num + 1) * block_size)];
      oracle_msg.extend(recovered_pt.to_vec());
      recovered_pt
        .push(find_next_pt_byte(oracle_msg, relevant_ct_block).unwrap());
    }
  }
  recovered_pt
}

fn find_next_pt_byte(oracle_msg: Vec<u8>, block: &[u8]) -> Result<u8, &str> {
  let blk_num = oracle_msg.len() / block.len();
  for byte in freq_analysis::ORDERED_PRINTABLE_ASCII.iter() {
    let mut msg = oracle_msg.clone();
    msg.push(byte.clone());
    let ct = aes_128_ecb_rand_key_oracle(msg);
    if block == &ct[(blk_num * block.len())..((blk_num + 1) * block.len())] {
      return Ok(byte.clone());
    }
  }
  Err("Failed to find byte :(")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn cracks_the_oracle() {
    let recovered_pt_bytes = crack_the_aes_ecb_oracle();
    let recovered_pt = bytes_to_string(recovered_pt_bytes);

    assert_eq!(recovered_pt, "Rollin' in my 5.0\nWith my rag-top down so my hair can blow\nThe girlies on standby waving just to say hi\nDid you stop? No, I just drove by\n");
  }

  #[test]
  fn can_determine_blocksize() {
    assert_eq!(discover_blocksize(), 16);
  }

  #[test]
  fn confirms_ecb_mode() {
    assert_eq!(test_ecb(), true);
  }
}
