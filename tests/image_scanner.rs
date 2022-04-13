use image::GenericImageView;
use qrcode_generator::QrCodeEcc;
use std::cmp;
use zbar_rust::{ZBarConfig, ZBarImageScanner, ZBarSymbolType};

#[inline(always)]
fn decryption_finished(encrypted: &Vec<u8>, counter: u32) -> u32 {
    encrypted.len() as u32 * 8 - counter
}

fn decrypt(encrypted: &Vec<u8>, counter: &mut u32, mut code: u32) -> usize {
    let mut tmp = 0_u32;
    let mut ret = 0_usize;
    loop {
        let cnt = *counter % 8;
        let v8 = 8 - cnt;
        let mval = cmp::min(code, cmp::min(8 - tmp, v8));

        let val1 = (ret << mval) as u8;
        let val2 = encrypted[(*counter / 8) as usize] << cnt;
        let val3 = (val2 & 0xFF) >> cnt;
        let val4 = val3 & 0xFF;
        let val5 = (v8 - mval) as u8;

        ret = (val1 | val4 >> val5) as usize;

        code -= mval;
        *counter += mval;
        tmp += mval;
        if tmp != 8 && code != 0 {
            continue;
        }
        break;
    }
    ret
}

pub fn decode_string(encrypted: &Vec<u8>) -> String {
    let mut ret = Vec::new();
    let mut counter = 0_u32;
    const MAGIC: &str = " !\"#$%&\'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_";

    while decryption_finished(encrypted, counter) >= 6 {
        ret.push(MAGIC.as_bytes()[decrypt(encrypted, &mut counter, 6)]);
    }

    let s = match std::str::from_utf8(&ret) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };
    s.to_string()
}

pub fn decode() {
    const INPUT_PATH: &str = "testdata/sample3.png";
    let img = image::open(INPUT_PATH).unwrap();
    let (width, height) = img.dimensions();
    let luma_img = img.to_luma8();
    let luma_img_data: Vec<u8> = luma_img.to_vec();
    let mut scanner = ZBarImageScanner::new();
    let mut results = scanner.scan_y800(&luma_img_data, width, height).unwrap();
    if results.len() == 1 {
        let binary_data = results.remove(0).data;

        // TODO: Parsear corretamente os campos e retornar o json
        // ou apenas retornar a string decriptada para o python
        let data_length = ((binary_data[12..14][0] as u16) << 8) | binary_data[12..14][1] as u16;
        let document_data = &binary_data[14..data_length as usize];
        println!("{}", decode_string(&document_data.to_vec()));
    }
}

#[test]
fn image_create_destroy() {
    let _scanner = ZBarImageScanner::new();
}

#[test]
fn set_config() {
    let mut scanner = ZBarImageScanner::new();
    scanner.set_config(ZBarSymbolType::ZBarNone, ZBarConfig::ZBarCfgEnable, 0).unwrap();
    scanner.set_config(ZBarSymbolType::ZBarQRCode, ZBarConfig::ZBarCfgEnable, 1).unwrap();
}

#[test]
fn decode_qrcode() {
    let mut scanner = ZBarImageScanner::new();
    let url = "https://magiclen.org";
    let size = 512;
    let data = qrcode_generator::to_image_from_str(url, QrCodeEcc::Low, size).unwrap();
    let mut result = scanner.scan_y800(&data, size as u32, size as u32).unwrap();

    assert_eq!(1, result.len());
    assert_eq!(ZBarSymbolType::ZBarQRCode, result[0].symbol_type);
    assert_eq!(url, unsafe { String::from_utf8_unchecked(result.remove(0).data) });

    decode();
}
