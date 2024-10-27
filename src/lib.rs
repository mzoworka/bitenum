use std::marker::PhantomData;

pub trait BitEnumTrait<T>
where
    T: Sized + int_enum::IntEnum,
    <T as int_enum::IntEnum>::Int: Default,
{
    type Values: int_enum::IntEnum;
    fn to_vec(&self) -> Result<Vec<T>, int_enum::IntEnumError<T>>;
    fn from_vec(bits: Vec<T>) -> Self;
    fn from_int(bits: <T as int_enum::IntEnum>::Int) -> Result<Self, int_enum::IntEnumError<T>>
    where
        Self: Sized;
    fn get_val(&self) -> T::Int;
    fn contains(&self, bit: &T) -> bool;
}

#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
struct BitEnumInner<T>
where
    T: Sized + int_enum::IntEnum,
    <T as int_enum::IntEnum>::Int: Default,
{
    data: T::Int,
}

impl<T> bincode_aligned::BincodeAlignedEncode for BitEnumInner<T>
where
    T: Sized + int_enum::IntEnum,
    <T as int_enum::IntEnum>::Int: Default,
    <T as int_enum::IntEnum>::Int: bincode_aligned::BincodeAlignedEncode,
{
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
        align: &bincode_aligned::BincodeAlignConfig,
    ) -> Result<(), bincode::error::EncodeError> {
        bincode_aligned::BincodeAlignedEncode::encode(&self.data, encoder, align)
    }
}

impl<T> bincode_aligned::BincodeAlignedDecode for BitEnumInner<T>
where
    T: Sized + int_enum::IntEnum,
    <T as int_enum::IntEnum>::Int: Default,
    <T as int_enum::IntEnum>::Int: bincode_aligned::BincodeAlignedDecode,
{
    fn decode<D: bincode::de::Decoder>(
        decoder: &mut D,
        align: &bincode_aligned::BincodeAlignConfig,
    ) -> Result<Self, bincode::error::DecodeError>
    where
        Self: Sized,
    {
        Ok(Self {
            data: bincode_aligned::BincodeAlignedDecode::decode(decoder, align)?,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BitEnum<T>
where
    T: Sized + int_enum::IntEnum,
    <T as int_enum::IntEnum>::Int: Default,
{
    data: BitEnumInner<T>,
    phantom: PhantomData<T>,
}

impl<T> BitEnumTrait<T> for BitEnum<T>
where
    T: Sized + int_enum::IntEnum,
    <T as int_enum::IntEnum>::Int: Default,
{
    type Values = T;
    fn to_vec(&self) -> Result<Vec<Self::Values>, int_enum::IntEnumError<Self::Values>> {
        let mut v = vec![];

        let mut data = self.data.data;
        for i in (0..((std::mem::size_of::<T::Int>() * 8) - 1)).rev() {
            let test = data >> i;
            let bit = test << i;
            if bit != Default::default() {
                data = data ^ bit;
                v.push(T::from_int(bit)?)
            }
        }

        Ok(v)
    }

    fn from_vec(bits: Vec<Self::Values>) -> Self {
        let mut sum = None;
        for bit in bits {
            sum = Some(sum.unwrap_or_default() | bit.int_value());
        }
        Self {
            data: BitEnumInner {
                data: sum.unwrap_or_default(),
            },
            phantom: PhantomData {},
        }
    }

    fn get_val(&self) -> T::Int {
        self.data.data
    }

    fn from_int(bits: <T as int_enum::IntEnum>::Int) -> Result<Self, int_enum::IntEnumError<T>> {
        let mut sum = None;
        let mut data = bits;
        for i in (0..((std::mem::size_of::<T::Int>() * 8) - 1)).rev() {
            let test = data >> i;
            let bit = test << i;
            if bit != Default::default() {
                data = data ^ bit;
                sum = Some(sum.unwrap_or_default() | T::from_int(bit)?.int_value());
            }
        }
        Ok(Self {
            data: BitEnumInner {
                data: sum.unwrap_or_default(),
            },
            phantom: PhantomData {},
        })
    }

    fn contains(&self, bit: &T) -> bool {
        let data = self.data.data;
        (data & bit.int_value()) != T::Int::default()
    }
}

impl<T> bincode_aligned::BincodeAlignedEncode for BitEnum<T>
where
    T: Sized + int_enum::IntEnum,
    <T as int_enum::IntEnum>::Int: Default,
    <T as int_enum::IntEnum>::Int: bincode_aligned::BincodeAlignedEncode,
{
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
        align: &bincode_aligned::BincodeAlignConfig,
    ) -> Result<(), bincode::error::EncodeError> {
        bincode_aligned::BincodeAlignedEncode::encode(&self.data, encoder, align)
    }
}

impl<T> bincode_aligned::BincodeAlignedDecode for BitEnum<T>
where
    T: Sized + int_enum::IntEnum,
    <T as int_enum::IntEnum>::Int: Default,
    <T as int_enum::IntEnum>::Int: bincode_aligned::BincodeAlignedDecode,
{
    fn decode<D: bincode::de::Decoder>(
        decoder: &mut D,
        align: &bincode_aligned::BincodeAlignConfig,
    ) -> Result<Self, bincode::error::DecodeError>
    where
        Self: Sized,
    {
        Ok(Self {
            data: bincode_aligned::BincodeAlignedDecode::decode(decoder, align)?,
            phantom: PhantomData {},
        })
    }
}

impl<'a, T> serde::Deserialize<'a> for BitEnum<T>
where
    T: Sized + int_enum::IntEnum,
    <T as int_enum::IntEnum>::Int:
        Default + bincode_aligned::BincodeAlignedDecode + serde::Deserialize<'a>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        Ok(Self {
            data: BitEnumInner {
                data: serde::de::Deserialize::deserialize(deserializer)?,
            },
            phantom: PhantomData,
        })
    }
}

impl<T> serde::Serialize for BitEnum<T>
where
    T: Sized + int_enum::IntEnum,
    <T as int_enum::IntEnum>::Int:
        Default + bincode_aligned::BincodeAlignedDecode + serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde::ser::Serialize::serialize(&self.data.data, serializer)
    }
}
