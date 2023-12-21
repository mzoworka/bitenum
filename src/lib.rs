use std::marker::PhantomData;
use bincode_aligned::{BincodeAlignedFromBincode, BincodeAlignedEncode};

pub trait BitEnumTrait<T>
where T: Sized + int_enum::IntEnum + bincode::Encode,
<T as int_enum::IntEnum>::Int: Default,
<T as int_enum::IntEnum>::Int: bincode::Encode,
<T as int_enum::IntEnum>::Int: bincode::Decode,
{
    type Values: int_enum::IntEnum;
    fn to_vec(&self) -> Result<Vec<T>, int_enum::IntEnumError<T>>;
    fn from_vec(bits: Vec<T>) -> Self;
    fn get_val(&self) -> T::Int;
}

#[derive(Default, Clone, Copy, Debug, Eq, PartialEq, bincode::Encode)]
struct BitEnumInner<T> 
where T: Sized + int_enum::IntEnum + bincode::Encode,
<T as int_enum::IntEnum>::Int: Default,
<T as int_enum::IntEnum>::Int: bincode::Encode,
<T as int_enum::IntEnum>::Int: bincode::Decode,
{
    data: T::Int,
}

impl<T> bincode::Decode for BitEnumInner<T>
where T: Sized + int_enum::IntEnum + bincode::Encode,
<T as int_enum::IntEnum>::Int: Default,
<T as int_enum::IntEnum>::Int: bincode::Encode,
<T as int_enum::IntEnum>::Int: bincode::Decode,
{
    fn decode<D: bincode::de::Decoder>(decoder: &mut D) -> Result<Self, bincode::error::DecodeError> {
        Ok(
            Self { data: bincode::Decode::decode(decoder)? }
        )
    }
}

impl<'de, T> bincode::BorrowDecode<'de> for BitEnumInner<T>
where T: Sized + int_enum::IntEnum + bincode::Encode,
<T as int_enum::IntEnum>::Int: Default,
<T as int_enum::IntEnum>::Int: bincode::Encode,
<T as int_enum::IntEnum>::Int: bincode::Decode,
{  
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de>>(
        decoder: &mut D,
    ) -> core::result::Result<Self, bincode::error::DecodeError> {
        bincode::Decode::decode(decoder)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, bincode::Encode, bincode::Decode, BincodeAlignedFromBincode)]
pub struct BitEnum<T>
where T: Sized + int_enum::IntEnum + bincode::Encode,
<T as int_enum::IntEnum>::Int: Default,
<T as int_enum::IntEnum>::Int: bincode::Encode,
<T as int_enum::IntEnum>::Int: bincode::Decode,
{
    data: BitEnumInner<T>,
    phantom: PhantomData<T>,
}

impl<T> BitEnumTrait<T> for BitEnum<T>
where T: Sized + int_enum::IntEnum + bincode::Encode,
<T as int_enum::IntEnum>::Int: Default,
<T as int_enum::IntEnum>::Int: bincode::Encode,
<T as int_enum::IntEnum>::Int: bincode::Decode,
<T as int_enum::IntEnum>::Int: bincode::BorrowDecode<'static>
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

