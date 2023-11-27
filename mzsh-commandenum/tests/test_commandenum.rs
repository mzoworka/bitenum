#![feature(derive_clone_copy, structural_match, fmt_helpers_for_derive, derive_eq)]

use std::marker::PhantomData;

use bitflags::bitflags;
use int_enum::IntEnum;
use mzsh_commandenum::{CommandEnumTrait, CommandEnumTraitMacro};
use num::{FromPrimitive, ToPrimitive};


#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnumTestType {
    POLL,
    SREQ,
    AREQ,
    SRSP,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntEnum, bincode::Encode, bincode::Decode)]
pub enum EnumTestId {
    A = 1,
    B = 2,
    C = 3,
    D = 4,
    E = 5,
    F = 6,
    G = 7,


    Z = 88,
}

#[derive(Debug, Clone)]
struct NullTerminatedBytes {
    inner: Vec<u8>,
}

impl<const X: usize> From<&[u8; X]> for NullTerminatedBytes
{
    fn from(value: &[u8; X]) -> Self {
        NullTerminatedBytes { inner: value.to_vec() }
    }
}

impl bincode::Encode for NullTerminatedBytes {
    fn encode<E: bincode::enc::Encoder>(&self, encoder: &mut E) -> Result<(), bincode::error::EncodeError> {
        bincode::enc::write::Writer::write(&mut encoder.writer(), self.inner.as_slice())
    }
}

impl bincode::Decode for NullTerminatedBytes {
    fn decode<D: bincode::de::Decoder>(decoder: &mut D) -> Result<Self, bincode::error::DecodeError> {
        let mut byte = [0u8; 1];
        let mut result = vec![];
        while let Ok(_) = bincode::de::read::Reader::read(&mut decoder.reader(), &mut byte) {
            result.push(byte[0]);
            if byte[0] == 0 {
                break;
            }
        }
        Ok(NullTerminatedBytes { inner: result })
    }
}


#[derive(Debug, Clone)]
struct CustomBytes {
    inner: Vec<u8>,
}

impl<const X: usize> From<&[u8; X]> for CustomBytes
{
    fn from(value: &[u8; X]) -> Self {
        CustomBytes { inner: value.to_vec() }
    }
}

impl bincode::Encode for CustomBytes {
    fn encode<E: bincode::enc::Encoder>(&self, encoder: &mut E) -> Result<(), bincode::error::EncodeError> {
        bincode::encode_into_writer(self.inner.len() as u16, encoder.writer(), bincode::config::standard())?;
        bincode::enc::write::Writer::write(&mut encoder.writer(), self.inner.as_slice())
    }
}

impl bincode::Decode for CustomBytes {
    fn decode<D: bincode::de::Decoder>(decoder: &mut D) -> Result<Self, bincode::error::DecodeError> {
        let size: u16 = bincode::decode_from_reader(decoder.reader(), bincode::config::standard())?;
        let mut byte = [0u8; 1];
        let mut result = vec![];
        for _ in 0..size {
            bincode::de::read::Reader::read(&mut decoder.reader(), &mut byte)?;
            result.push(byte[0]);
        }
        Ok(CustomBytes { inner: result })
    }
}

#[derive(Debug, Clone)]
struct LVList<LengthType, ValueType>
where LengthType: Sized + Clone + bincode::Encode + bincode::Decode + FromPrimitive + ToPrimitive + Default, ValueType: Sized + Clone + bincode::Encode + bincode::Decode
{
    inner: Vec<ValueType>,
    phantom: PhantomData<LengthType>,
}

impl<const X: usize, LengthType, ValueType> From<&[ValueType; X]> for LVList<LengthType, ValueType>
where LengthType: Sized + Clone + bincode::Encode + bincode::Decode + FromPrimitive + ToPrimitive + Default, ValueType: Sized + Clone + bincode::Encode + bincode::Decode
{
    fn from(value: &[ValueType; X]) -> Self {
        LVList { inner: value.to_vec(), phantom: PhantomData }
    }
}

impl<LengthType, ValueType> bincode::Encode for LVList<LengthType, ValueType> 
where LengthType: Sized + Clone + bincode::Encode + bincode::Decode + FromPrimitive + ToPrimitive + Default, ValueType: Sized + Clone + bincode::Encode + bincode::Decode
{
    fn encode<E: bincode::enc::Encoder>(&self, encoder: &mut E) -> Result<(), bincode::error::EncodeError> {
        let typed_size: LengthType = LengthType::from_usize(self.inner.len()).unwrap_or_default();
        bincode::encode_into_writer(typed_size, encoder.writer(), bincode::config::standard())?;
        bincode::encode_into_writer(&self.inner, encoder.writer(), bincode::config::standard())
    }
}

impl<LengthType, ValueType> bincode::Decode for LVList<LengthType, ValueType> 
where LengthType: Sized + Clone + bincode::Encode + bincode::Decode + FromPrimitive + ToPrimitive + Default, ValueType: Sized + Clone + bincode::Encode + bincode::Decode
{
    fn decode<D: bincode::de::Decoder>(decoder: &mut D) -> Result<Self, bincode::error::DecodeError> {
        let size: LengthType = bincode::decode_from_reader(decoder.reader(), bincode::config::standard())?;
        let mut byte = [0u8; 1];
        let mut result = vec![];
        for _ in 0..size.to_usize().unwrap_or_default() {
            bincode::de::read::Reader::read(&mut decoder.reader(), &mut byte)?;
            result.push(byte[0]);
        }
        Ok(LVList { inner: bincode::decode_from_slice(result.as_slice(), bincode::config::standard())?.0, phantom: PhantomData })
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntEnum, bincode::Encode, bincode::Decode)]
pub enum LatencyReq {
    NoLatencyReqs = 0x00,
    FastBeacons = 0x01,
    SlowBeacons = 0x02,
}


type Bitmap8 = u8;
type NWK = u16;
type ClusterId = u16;
type ShortBytes = LVList<u8, u8>;
type LongBytes = LVList<u16, u8>;
type PanId = NWK;
type EUI64 = [u8; 8];

bitflags! {
    #[derive(Debug, Clone)]
    struct TransmitOptions: Bitmap8 {
        //Will force the message to use Wildcard ProfileID
        const WILDCARD_PROFILEID = 0x02;

        //Will force APS to callback to preprocess before calling NWK layer
        const APS_PREPROCESS = 0x04;
        const LIMIT_CONCENTRATOR = 0x08;
        const ACK_REQUEST = 0x10;

        //Suppress Route Discovery for intermediate routes (route discovery performed for
        //initiating device)
        const SUPPRESS_ROUTE_DISC_NETWORK = 0x20;
        const ENABLE_SECURITY = 0x40;
        const SKIP_ROUTING = 0x80;
    }
}

impl bincode::Encode for TransmitOptions
{
    fn encode<E: bincode::enc::Encoder>(&self, encoder: &mut E) -> Result<(), bincode::error::EncodeError> {
        let config = *(encoder.config());
        bincode::encode_into_writer(self.bits(), encoder.writer(), config)
    }
}

impl bincode::Decode for TransmitOptions
{
    fn decode<D: bincode::de::Decoder>(decoder: &mut D) -> Result<Self, bincode::error::DecodeError> {
        let config = *(decoder.config());
        Ok(TransmitOptions::from_bits_retain(bincode::decode_from_reader(decoder.reader(), config)?))
    }
}

//TODO: StructIntEnum
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntEnum, bincode::Encode, bincode::Decode)]
pub enum AddrMode {
    Group = 0x01,
    NWK = 0x02,
    IEEE = 0x03,
    Broadcast = 0x0F,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum AddrModeAddress {
    Group(NWK),
    NWK(NWK),
    IEEE(EUI64),
    Broadcast(NWK),
}

impl bincode::Encode for AddrModeAddress
{
    fn encode<E: bincode::enc::Encoder>(&self, encoder: &mut E) -> Result<(), bincode::error::EncodeError> {
        let config = *(encoder.config());
        match self {
            Self::Group(x) => {
                bincode::encode_into_writer(AddrMode::Group, encoder.writer(), config)?;
                bincode::encode_into_writer(x, encoder.writer(), config)
            },
            Self::NWK(x) => {
                bincode::encode_into_writer(AddrMode::NWK, encoder.writer(), config)?;
                bincode::encode_into_writer(x, encoder.writer(), config)
            },
            Self::IEEE(x) => {
                bincode::encode_into_writer(AddrMode::IEEE, encoder.writer(), config)?;
                bincode::encode_into_writer(x, encoder.writer(), config)
            },
            Self::Broadcast(x) => {
                bincode::encode_into_writer(AddrMode::Broadcast, encoder.writer(), config)?;
                bincode::encode_into_writer(x, encoder.writer(), config)
            },
        }
    }
}

impl bincode::Decode for AddrModeAddress
{
    fn decode<D: bincode::de::Decoder>(decoder: &mut D) -> Result<Self, bincode::error::DecodeError> {
        let config = *(decoder.config());
        let mode: AddrMode = bincode::decode_from_reader(decoder.reader(), config)?;
        match mode {
            AddrMode::Group => Ok(Self::Group(bincode::decode_from_reader(decoder.reader(), config)?)),
            AddrMode::NWK => Ok(Self::NWK(bincode::decode_from_reader(decoder.reader(), config)?)),
            AddrMode::IEEE => Ok(Self::IEEE(bincode::decode_from_reader(decoder.reader(), config)?)),
            AddrMode::Broadcast => Ok(Self::Broadcast(bincode::decode_from_reader(decoder.reader(), config)?)),
        }
    }
}


#[derive(CommandEnumTraitMacro, Debug, Clone)]
#[command_enum_trait(Id = EnumTestId, Type = i8, SerializeIdOrder = 1)]
enum EReq{
    #[command_enum_trait(command_id = EnumTestId::A, command_type = 0)]
    A,
    #[command_enum_trait(command_id = EnumTestId::B, command_type = 0)]
    B(i8),
    #[command_enum_trait(command_id = EnumTestId::C, command_type = 0)]
    C((i8, String)),
    #[command_enum_trait(command_id = EnumTestId::D, command_type = 0)]
    D(i8, NullTerminatedBytes),
    #[command_enum_trait(command_id = EnumTestId::E, command_type = 0)]
    E{a: i8, b:[u8; 4]},
    #[command_enum_trait(command_id = EnumTestId::F, command_type = 0)]
    F{a: i8, b:CustomBytes},
    #[command_enum_trait(command_id = EnumTestId::G, command_type = 0)]
    G{a: i8, b:TransmitOptions},
}



#[derive(CommandEnumTraitMacro, Debug, Clone)]
#[command_enum_trait(Id = u8, Type = EnumTestType, SerializeIdOrder = 1)]
enum AFReq{
    #[command_enum_trait(command_id = 0x00, command_type = EnumTestType::SREQ)]
    Register{
        endpoint: u8, //Endpoint Id of the device
        profile_id: u16, //Application Profile ID
        device_id: u16, //Device Description ID
        device_version: u8, //Device version number
        latency_req: LatencyReq, //Specifies latency reqs
        input_clusters: LVList<u8, u16>, //Input cluster list
        output_clusters: LVList<u8, u16>, //Output cluster list
    },
    #[command_enum_trait(command_id = 0x01, command_type = EnumTestType::SREQ)]
    DataRequest{
        dst_addr: NWK, //Short address of the destination device
        dst_endpoint: u8, //Endpoint of the destination device
        src_endpoint: u8, //Endpoint of the source device
        cluster_id: ClusterId, //Cluster ID
        tsn: u8, //Transaction Sequence Number
        options: TransmitOptions, //Transmit options bitmask
        radius: u8, //Specifies the number of hops allowed delivering the message
        data: ShortBytes, //Data request
    },
    #[command_enum_trait(command_id = 0x02, command_type = EnumTestType::SREQ)]
    DataRequestExt{
        dst_addr_mode_address: AddrModeAddress, //Destination address mode and address
        dst_endpoint: u8, //Endpoint of the destination device
        dst_pan_id: PanId, //PanId of the destination device
        src_endpoint: u8, //Endpoint of the source device
        cluster_id: ClusterId, //Cluster ID
        tsn: u8, //Transaction Sequence Number
        options: TransmitOptions, //Transmit options bitmask
        radius: u8, //Specifies the number of hops allowed delivering the message
        data: LongBytes, //Data request
    },

}

#[test]
fn test_macro(){
    let demo1=EReq::A;
    println!("demo1.to_string: {:?}", demo1.to_string());
    println!("demo1.get_id: {:?}", demo1.get_id());
    println!("demo1.get_type: {:?}", demo1.get_type());
    println!("demo1.encode_data: {:?}", demo1.encode_data());

    let demo2=EReq::B(4);
    println!("demo2.to_string: {:?}", demo2.to_string());
    println!("demo2.get_id: {:?}", demo2.get_id());
    println!("demo2.get_type: {:?}", demo2.get_type());
    println!("demo2.encode_data: {:?}", demo2.encode_data());

    println!("demo3(EReq::decode_by_id(A, [])): {:?}", EReq::decode_by_id(EnumTestId::A, &mut std::io::BufReader::new(vec![].as_slice())));
    println!("demo3(EReq::decode_by_id(B, [])): {:?}", EReq::decode_by_id(EnumTestId::B, &mut std::io::BufReader::new(vec![].as_slice())));
    println!("demo3(EReq::decode_by_id(B, [4])): {:?}", EReq::decode_by_id(EnumTestId::B, &mut std::io::BufReader::new(vec![4].as_slice())));
    let mut data = bincode::encode_to_vec(&(4 as i8), bincode::config::standard()).unwrap();
    data.extend(bincode::encode_to_vec(&("testc"), bincode::config::standard()).unwrap());
    println!("demo3(EReq::decode_by_id(C, [4, 'test'])): {:?} -> {:?}", data.clone(), EReq::decode_by_id(EnumTestId::C, &mut std::io::BufReader::new(data.as_slice())));

    let mut data = bincode::encode_to_vec(&(4 as i8), bincode::config::standard()).unwrap();
    data.extend(bincode::encode_to_vec(&(b"testd\0asd"), bincode::config::standard()).unwrap());
    println!("demo3(EReq::decode_by_id(D, [4, 'test'])): {:?} -> {:?}", data.clone(), EReq::decode_by_id(EnumTestId::D, &mut std::io::BufReader::new(data.as_slice())));


    let mut data = bincode::encode_to_vec(&(4 as i8), bincode::config::standard()).unwrap();
    data.extend(bincode::encode_to_vec(&("teste"), bincode::config::standard()).unwrap());
    println!("demo3(EReq::decode_by_id(E, [4, 'test'])): {:?} -> {:?}", data.clone(), EReq::decode_by_id(EnumTestId::E, &mut std::io::BufReader::new(data.as_slice())));

    let mut data = bincode::encode_to_vec(&(4 as i8), bincode::config::standard()).unwrap();
    data.extend(bincode::encode_to_vec(&(6 as u16), bincode::config::standard()).unwrap());
    data.extend(bincode::encode_to_vec(&(b"testfasd"), bincode::config::standard()).unwrap());
    let ereqf = EReq::decode_by_id(EnumTestId::F, &mut std::io::BufReader::new(data.clone().as_slice()));
    println!("demo3(EReq::decode_by_id(F, [4, 'testfasd'])): {:?} -> {:?} -> {:?}", data, ereqf.clone(), ereqf.map(|x| x.encode_data()));

    let mut data = bincode::encode_to_vec(&(4 as i8), bincode::config::standard()).unwrap();
    data.extend(bincode::encode_to_vec(&(24 as u8), bincode::config::standard()).unwrap());
    let ereqf = EReq::decode_by_id(EnumTestId::G, &mut std::io::BufReader::new(data.clone().as_slice()));
    println!("demo3(EReq::decode_by_id(G, [4, 24])): {:?} -> {:?} -> {:?}", data, ereqf.clone(), ereqf.map(|x| x.encode_data()));


    let demo4=EReq::D(4, b"test".into());
    println!("demo4.to_string: {:?}", demo4.to_string());
    println!("demo4.get_id: {:?}", demo4.get_id());
    println!("demo4.get_type: {:?}", demo4.get_type());
    println!("demo4.encode_data: {:?}", demo4.encode_data());

    let demo5=EReq::E{a: 4, b:b"test".clone()};
    println!("demo5.to_string: {:?}", demo5.to_string());
    println!("demo5.get_id: {:?}", demo5.get_id());
    println!("demo5.get_type: {:?}", demo5.get_type());
    println!("demo5.encode_data: {:?}", demo5.encode_data());

    let demo6=EReq::G{a: 4, b: TransmitOptions::LIMIT_CONCENTRATOR | TransmitOptions::ACK_REQUEST};
    println!("demo6.to_string: {:?}", demo6.to_string());
    println!("demo6.get_id: {:?}", demo6.get_id());
    println!("demo6.get_type: {:?}", demo6.get_type());
    println!("demo6.encode_data: {:?}", demo6.encode_data());

    let demo7=EReq::G{a: 4, b: TransmitOptions::LIMIT_CONCENTRATOR | TransmitOptions::ACK_REQUEST};
    let demo7_encoded = bincode::encode_to_vec(demo7, bincode::config::standard());
    println!("demo7.encode: {:?}", &demo7_encoded);

    let demo8: Result<(EReq, usize), _> = bincode::decode_from_slice(demo7_encoded.unwrap_or(vec![]).as_slice(), bincode::config::standard());
    println!("demo8.decode: {:?}", &demo8);
    
}
