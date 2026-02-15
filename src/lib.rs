use std::{fmt::Debug, marker::PhantomData, ops::{Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign}};

pub trait IntTrait: Debug + Clone + Copy + Default + PartialEq + PartialOrd + Eq + Ord + Shr<Output=Self> + Shl<Output=Self> + ShrAssign + ShlAssign + Add<Output=Self> + AddAssign + Sub<Output=Self> + SubAssign + Shl<usize, Output=Self> + Shr<usize, Output=Self> + BitAnd<Output=Self> + BitOr<Output=Self> + BitXor<Output=Self> + BitAndAssign + BitOrAssign + BitXorAssign + Not<Output=Self> {}

pub trait IntEnumTrait: Copy + Clone + TryFrom<Self::Int>{
    type Int: Debug + Clone + Copy + Default + PartialEq + PartialOrd + Eq + Ord + Shr<Output=Self::Int> + Shl<Output=Self::Int> + ShrAssign + ShlAssign + Add<Output=Self::Int> + AddAssign + Sub<Output=Self::Int> + SubAssign + Shl<usize, Output=Self::Int> + Shr<usize, Output=Self::Int> + BitAnd<Output=Self::Int> + BitOr<Output=Self::Int> + BitXor<Output=Self::Int> + BitAndAssign + BitOrAssign + BitXorAssign + Not<Output=Self::Int> + From<Self>;
}

pub trait BitEnumTrait<T>
where
    T: Sized + IntEnumTrait,
{
    type Values: IntEnumTrait;
    fn to_vec(&self) -> Result<Vec<T>, <T as TryFrom<T::Int>>::Error>;
    fn from_slice(bits: &[T]) -> Self;
    fn from_iter<'a, I: IntoIterator<Item=&'a T>>(bits: I) -> Self
        where T: 'a;
    fn try_from_iter<E, I: IntoIterator<Item=Result<T, E>>>(bits: I) -> Result<Self, E>
        where Self: Sized;
    fn from_int(bits: <T as IntEnumTrait>::Int) -> Result<Self, <T as TryFrom<T::Int>>::Error>
        where Self: Sized;
    fn get_val(&self) -> T::Int;
    fn contains(&self, bit: &T) -> bool;
    fn add_bit(self, bit: &T) -> Self;
    fn remove_bit(self, bit: &T) -> Self;
    fn xor_bit(self, bit: &T) -> Self;
}

#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
struct BitEnumInner<T>
where
    T: Sized + IntEnumTrait,
{
    data: T::Int,
}

impl<T> bincode_aligned::BincodeAlignedEncode for BitEnumInner<T>
where
    T: Sized + IntEnumTrait,
    <T as IntEnumTrait>::Int: bincode_aligned::BincodeAlignedEncode,
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
    T: Sized + IntEnumTrait,
    <T as IntEnumTrait>::Int: bincode_aligned::BincodeAlignedDecode,
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
    T: Sized + IntEnumTrait,
{
    data: BitEnumInner<T>,
    phantom: PhantomData<T>,
}

impl<T> BitEnumTrait<T> for BitEnum<T>
where
    T: Sized + IntEnumTrait,
{
    type Values = T;
    fn to_vec(&self) -> Result<Vec<Self::Values>, <T as TryFrom<T::Int>>::Error> {
        let mut v = vec![];

        let mut data = self.data.data;
        for i in (0..((std::mem::size_of::<T::Int>() * 8) - 1)).rev() {
            let test = data >> i;
            let bit = test << i;
            if bit != Default::default() {
                data = data ^ bit;
                v.push(T::try_from(bit)?);
            }
        }

        Ok(v)
    }

    fn from_slice(bits: &[Self::Values]) -> Self {
        let mut sum = None;
        for bit in bits {
            sum = Some(sum.unwrap_or_default() | bit.clone().into());
        }
        Self {
            data: BitEnumInner {
                data: sum.unwrap_or_default(),
            },
            phantom: PhantomData {},
        }
    }

    fn from_iter<'a, I: IntoIterator<Item=&'a T>>(bits: I) -> Self
    where T: 'a
    {
        let mut sum = None;
        for bit in bits {
            sum = Some(sum.unwrap_or_default() | bit.clone().into());
        }
        Self {
            data: BitEnumInner {
                data: sum.unwrap_or_default(),
            },
            phantom: PhantomData {},
        }
    }

    fn try_from_iter<E, I: IntoIterator<Item=Result<T, E>>>(bits: I) -> Result<Self, E>
    {
        let mut sum = None;
        for bit in bits {
            sum = Some(sum.unwrap_or_default() | bit?.clone().into());
        }
        Ok(Self {
            data: BitEnumInner {
                data: sum.unwrap_or_default(),
            },
            phantom: PhantomData {},
        })
    }

    fn get_val(&self) -> T::Int {
        self.data.data
    }

    fn from_int(bits: <T as IntEnumTrait>::Int) -> Result<Self, <T as TryFrom<T::Int>>::Error> {
        let mut sum = None;
        let mut data = bits;
        for i in (0..((std::mem::size_of::<T::Int>() * 8) - 1)).rev() {
            let test = data >> i;
            let bit = test << i;
            if bit != Default::default() {
                data = data ^ bit;
                sum = Some(sum.unwrap_or_default() | T::try_from(bit)?.into());
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
        (data & bit.clone().into()) != T::Int::default()
    }
    
    fn add_bit(self, bit: &T) -> Self {
        let mut sum = self.data.data;
        sum = sum | bit.clone().into();
        Self {
            data: BitEnumInner {
                data: sum,
            },
            phantom: PhantomData {},
        }
    }
    
    fn remove_bit(self, bit: &T) -> Self {
        let mut sum = self.data.data;
        let bit: T::Int = bit.clone().into();
        sum = sum & !bit;
        Self {
            data: BitEnumInner {
                data: sum,
            },
            phantom: PhantomData {},
        }
    }
    
    fn xor_bit(self, bit: &T) -> Self {
        let mut sum = self.data.data;
        sum = sum ^ bit.clone().into();
        Self {
            data: BitEnumInner {
                data: sum,
            },
            phantom: PhantomData {},
        }
    }
}

impl<T> bincode_aligned::BincodeAlignedEncode for BitEnum<T>
where
    T: Sized + IntEnumTrait,
    <T as IntEnumTrait>::Int: bincode_aligned::BincodeAlignedEncode,
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
    T: Sized + IntEnumTrait,
    <T as IntEnumTrait>::Int: bincode_aligned::BincodeAlignedDecode,
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
    T: Sized + IntEnumTrait,
    <T as IntEnumTrait>::Int: serde::Deserialize<'a>,
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
    T: Sized + IntEnumTrait,
    <T as IntEnumTrait>::Int: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde::ser::Serialize::serialize(&self.data.data, serializer)
    }
}

impl<T> Default for BitEnum<T>
where 
    T: IntEnumTrait,
{
    fn default() -> Self {
        Self { data: BitEnumInner{
            data: <T as IntEnumTrait>::Int::default(),
        }, phantom: PhantomData {} }
    }
}
