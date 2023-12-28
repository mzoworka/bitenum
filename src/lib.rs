use std::marker::PhantomData;
use bincode_aligned::{BincodeAlignedFromBincode, BincodeAlignedEncode};

pub trait BitEnumTrait<T>
where T: Sized + int_enum::IntEnum,
<T as int_enum::IntEnum>::Int: Default,
{
    type Values: int_enum::IntEnum;
    fn to_vec(&self) -> Result<Vec<T>, int_enum::IntEnumError<T>>;
    fn from_vec(bits: Vec<T>) -> Self;
    fn get_val(&self) -> T::Int;
}

#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
struct BitEnumInner<T> 
where T: Sized + int_enum::IntEnum ,
<T as int_enum::IntEnum>::Int: Default,
{
    data: T::Int,
}

impl<T> bincode_aligned::BincodeAlignedEncode for BitEnumInner<T>
where T: Sized + int_enum::IntEnum,
<T as int_enum::IntEnum>::Int: Default,
<T as int_enum::IntEnum>::Int: bincode_aligned::BincodeAlignedEncode,
{
    fn encode<E: bincode::enc::Encoder>(&self, encoder: &mut E, align: &bincode_aligned::BincodeAlignConfig) -> Result<(), bincode::error::EncodeError> {
        bincode_aligned::BincodeAlignedEncode::encode(&self.data, encoder, align)
    }
}

impl<T> bincode_aligned::BincodeAlignedDecode for BitEnumInner<T>
where T: Sized + int_enum::IntEnum,
<T as int_enum::IntEnum>::Int: Default,
<T as int_enum::IntEnum>::Int: bincode_aligned::BincodeAlignedDecode,
{
    fn decode<D: bincode::de::Decoder>(decoder: &mut D, align: &bincode_aligned::BincodeAlignConfig) -> Result<Self, bincode::error::DecodeError> where Self: Sized {
        Ok(Self { data: bincode_aligned::BincodeAlignedDecode::decode(decoder, align)? })
    }
}

/*
impl<T> bincode::Encode for BitEnumInner<T>
where T: Sized + int_enum::IntEnum,
<T as int_enum::IntEnum>::Int: Default,
<T as int_enum::IntEnum>::Int: bincode::Encode,
{
    fn encode<E: bincode::enc::Encoder>(&self, encoder: &mut E) -> Result<(), bincode::error::EncodeError> {
        bincode::Encode::encode(&self.data, encoder)
    }
}

impl<T> bincode::Decode for BitEnumInner<T>
where T: Sized + int_enum::IntEnum,
<T as int_enum::IntEnum>::Int: Default,
<T as int_enum::IntEnum>::Int: bincode::Decode,
{
    fn decode<D: bincode::de::Decoder>(decoder: &mut D) -> Result<Self, bincode::error::DecodeError> {
        Ok(
            Self { data: bincode::Decode::decode(decoder)? }
        )
    }
}

impl<'de, T> bincode::BorrowDecode<'de> for BitEnumInner<T>
where T: Sized + int_enum::IntEnum ,
<T as int_enum::IntEnum>::Int: Default,
<T as int_enum::IntEnum>::Int: bincode::Decode,
{  
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de>>(
        decoder: &mut D,
    ) -> core::result::Result<Self, bincode::error::DecodeError> {
        bincode::Decode::decode(decoder)
    }
}
*/

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BitEnum<T>
where T: Sized + int_enum::IntEnum,
<T as int_enum::IntEnum>::Int: Default,
{
    data: BitEnumInner<T>,
    phantom: PhantomData<T>,
}

impl<T> BitEnumTrait<T> for BitEnum<T>
where T: Sized + int_enum::IntEnum,
<T as int_enum::IntEnum>::Int: Default,
{
    type Values = T;
    fn to_vec(&self) -> Result<Vec<Self::Values>, int_enum::IntEnumError<Self::Values>> {
        let mut v = vec![];

        let mut data = self.data.data;        
        for i in 0..(std::mem::size_of::<T::Int>()*8) {
            let test = data >> i;
            if test << i != data {
                let old_data = data;
                data = test << i;
                v.push(T::from_int(old_data & !data)?)
            }
        }       
        
        Ok(v)
    }

    fn from_vec(bits: Vec<Self::Values>) -> Self {
        let mut sum = None;
        for bit in bits {
            sum = Some(sum.unwrap_or_default() | bit.int_value());
        }
        Self {data: BitEnumInner { data: sum.unwrap_or_default() }, phantom: PhantomData{}}
    }

    fn get_val(&self) -> T::Int {
        self.data.data
    }
}


impl<T> bincode_aligned::BincodeAlignedEncode for BitEnum<T>
where T: Sized + int_enum::IntEnum,
<T as int_enum::IntEnum>::Int: Default,
<T as int_enum::IntEnum>::Int: bincode_aligned::BincodeAlignedEncode,
{
    fn encode<E: bincode::enc::Encoder>(&self, encoder: &mut E, align: &bincode_aligned::BincodeAlignConfig) -> Result<(), bincode::error::EncodeError> {
        bincode_aligned::BincodeAlignedEncode::encode(&self.data, encoder, align)
    }
}

impl<T> bincode_aligned::BincodeAlignedDecode for BitEnum<T>
where T: Sized + int_enum::IntEnum,
<T as int_enum::IntEnum>::Int: Default,
<T as int_enum::IntEnum>::Int: bincode_aligned::BincodeAlignedDecode,
{
    fn decode<D: bincode::de::Decoder>(decoder: &mut D, align: &bincode_aligned::BincodeAlignConfig) -> Result<Self, bincode::error::DecodeError> where Self: Sized {
        Ok(Self { data: bincode_aligned::BincodeAlignedDecode::decode(decoder, align)?, phantom:PhantomData{} })
    }
}
