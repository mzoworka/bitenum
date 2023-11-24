pub use mzsh_commandenum_macros::CommandEnumTraitMacro;
pub use bincode;

pub trait CommandEnumTrait<Id, Type>
    where Self: Sized,
 {
    fn get_id(&self) -> Id;
    fn get_type(&self) -> Type;
    fn encode_data(&self) -> Result<Vec<u8>, String>;
    fn decode_by_id<R: bincode::de::read::Reader>(id: Id, buffer: &mut R) -> Result<Self, String>;
    fn encode_header(&self) -> Result<Vec<u8>, String>;
    fn decode_header<R: bincode::de::read::Reader>(buffer: &mut R) -> (Option<Id>, Option<Type>);
}
