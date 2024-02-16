use bitenum::{BitEnum, BitEnumTrait};
use int_enum::IntEnum;

#[allow(non_camel_case_types)]
#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntEnum, bincode::Encode, bincode::Decode)]
pub(crate) enum StartupStateValues {
    CHANNEL_11 = 0x00000800,
    CHANNEL_12 = 0x00001000,
    CHANNEL_13 = 0x00002000,
    CHANNEL_14 = 0x00004000,
    CHANNEL_15 = 0x00008000,
    CHANNEL_16 = 0x00010000,
    CHANNEL_17 = 0x00020000,
    CHANNEL_18 = 0x00040000,
    CHANNEL_19 = 0x00080000,
    CHANNEL_20 = 0x00100000,
    CHANNEL_21 = 0x00200000,
    CHANNEL_22 = 0x00400000,
    CHANNEL_23 = 0x00800000,
    CHANNEL_24 = 0x01000000,
    CHANNEL_25 = 0x02000000,
    CHANNEL_26 = 0x04000000,
}

pub(crate) type StartupState = BitEnum<StartupStateValues>;

#[test]
fn test_macro() {
    let bitmask1 = StartupState::from_vec(vec![
        StartupStateValues::CHANNEL_17,
        StartupStateValues::CHANNEL_20,
    ]);
    println!("test1: {:?}, {:?}", bitmask1, bitmask1.to_vec());

    let data = bincode::encode_to_vec(&(14 as i32), bincode::config::standard()).unwrap();
    let bitmask2: StartupState = bincode_aligned::decode_from_reader(
        &mut std::io::BufReader::new(data.as_slice()),
        bincode::config::standard(),
        &bincode_aligned::BincodeAlignConfig::Packed,
    )
    .unwrap();
    println!("test2: {:?}, {:?}", bitmask2, bitmask2.to_vec());

    let data = bincode::encode_to_vec(&(0x00402000 as i32), bincode::config::standard()).unwrap();
    let bitmask3: StartupState = bincode_aligned::decode_from_reader(
        &mut std::io::BufReader::new(data.as_slice()),
        bincode::config::standard(),
        &bincode_aligned::BincodeAlignConfig::Packed,
    )
    .unwrap();
    println!("test3: {:?}, {:?}", bitmask3, bitmask3.to_vec());
}
