extern crate proc_macro;
use proc_macro2::{Ident, Literal};
use quote::{quote, ToTokens, format_ident};
use syn::{parse_macro_input, DeriveInput, Data, Token, punctuated::Punctuated, parse::{Parse, ParseStream}, Fields};
use std::collections::BTreeMap;

mod kw {
    use syn::custom_keyword;
    pub use syn::token::Crate;

    // enum metadata
    custom_keyword!(Id);
    custom_keyword!(Type);
    custom_keyword!(SerializeIdOrder);
    custom_keyword!(SerializeTypeOrder);
    custom_keyword!(command_id);
    custom_keyword!(command_type);
}

enum LiteralOrIdents
{
    Literal(Literal),
    Idents(Punctuated<Ident, Token![::]>),
    Ident(Ident),
}

#[derive(Debug, Ord, PartialOrd, PartialEq, Eq)]
enum EnumParams
{
    IdIdent,
    TypeIdent,
    SerializeIdOrderIdent,
    SerializeTypeOrderIdent,
}

struct EnumParamStruct
{
    key: EnumParams,
    value: LiteralOrIdents,
}

impl Parse for EnumParamStruct {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::Id) {
            let _: kw::Id = input.parse()?;
            let _: Token![=] = input.parse()?;
            let value = input.parse()?;
            Ok(EnumParamStruct { key: EnumParams::IdIdent, value: LiteralOrIdents::Ident(value) })
        } else if lookahead.peek(kw::Type) {
            let _: kw::Type = input.parse()?;
            let _: Token![=] = input.parse()?;
            let value = input.parse()?;
            Ok(EnumParamStruct { key: EnumParams::TypeIdent, value: LiteralOrIdents::Ident(value) })
        } else if lookahead.peek(kw::SerializeIdOrder) {
            let _: kw::SerializeIdOrder = input.parse()?;
            let _: Token![=] = input.parse()?;
            let value = input.parse()?;
            Ok(EnumParamStruct { key: EnumParams::SerializeIdOrderIdent, value: LiteralOrIdents::Literal(value) })
        } else if lookahead.peek(kw::SerializeTypeOrder) {
            let _: kw::SerializeTypeOrder = input.parse()?;
            let _: Token![=] = input.parse()?;
            let value = input.parse()?;
            Ok(EnumParamStruct { key: EnumParams::SerializeTypeOrderIdent, value: LiteralOrIdents::Literal(value) })
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug, Ord, PartialOrd, PartialEq, Eq)]
enum EnumVariantParams
{
    CommandIdIdent,
    CommandTypeIdent,
}

impl Parse for LiteralOrIdents {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::Lit) {
            Ok(LiteralOrIdents::Literal(input.parse()?))
        } else {
            Ok(LiteralOrIdents::Idents(Punctuated::<_, _>::parse_separated_nonempty(input)?))
        }
    }
}

impl ToTokens for LiteralOrIdents {
    fn to_token_stream(&self) -> proc_macro2::TokenStream {
        match self {
            Self::Literal(x) => x.into_token_stream(),
            Self::Idents(x) => x.into_token_stream(),
            Self::Ident(x) => x.into_token_stream(),
        }
    }

    fn into_token_stream(self) -> proc_macro2::TokenStream
        where
            Self: Sized, {
                self.to_token_stream()
            }

    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.to_token_stream());
    }
}

struct EnumVariantParamStruct
{
    key: EnumVariantParams,
    value: LiteralOrIdents,
}

impl Parse for EnumVariantParamStruct {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::command_id) {
            let _: kw::command_id = input.parse()?;
            let _: Token![=] = input.parse()?;
            let value = input.parse()?;
            Ok(EnumVariantParamStruct { key: EnumVariantParams::CommandIdIdent, value })
        } else if lookahead.peek(kw::command_type) {
            let _: kw::command_type = input.parse()?;
            let _: Token![=] = input.parse()?;
            let value = input.parse()?;
            Ok(EnumVariantParamStruct { key: EnumVariantParams::CommandTypeIdent, value })
        } else {
            Err(lookahead.error())
        }
    }
}

#[proc_macro_derive(CommandEnumTraitMacro, attributes(command_enum_trait))]
pub fn derive_trait(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = input.generics;
    
    let mut attrs = input.attrs.iter()
        .filter(|x| x.path().is_ident("command_enum_trait"));
    let parsed: Result<Vec<EnumParamStruct>, syn::Error> = attrs.try_fold(Vec::new(), |mut vec, attr| {
        vec.extend(attr.parse_args_with(Punctuated::<_, Token![,]>::parse_terminated)?);
        Ok(vec)
    });
    let parsed_map = match parsed {
        Err(e) => return e.to_compile_error().into(),
        Ok(parsed) => parsed.into_iter().map(|x| (x.key, x.value)).collect::<BTreeMap<_, _>>(),
    };
    let id_ident = match parsed_map.get(&EnumParams::IdIdent) {
        Some(v) => v,
        None => return syn::Error::new(name.span(), "Missing command enum param: Id".to_string()).to_compile_error().into(),
    };
    let type_ident = match parsed_map.get(&EnumParams::TypeIdent) {
        Some(v) => v,
        None => return syn::Error::new(name.span(), "Missing command enum param: Type".to_string()).to_compile_error().into(),
    };
    let serialize_id_order: Option<u8> = match parsed_map.get(&EnumParams::SerializeIdOrderIdent) {
        Some(x) => match x { 
            LiteralOrIdents::Literal(x) => match x.to_string().parse() {
                Ok(x) => Some(x),
                Err(_) => return syn::Error::new(name.span(), "Expected u8 value for: SerializeIdOrder".to_string()).to_compile_error().into(),
            },
            _ => return syn::Error::new(name.span(), "Expected Literal value for: SerializeIdOrder".to_string()).to_compile_error().into(), 
        },
        None => None,
    };
    let serialize_type_order: Option<u8> = match parsed_map.get(&EnumParams::SerializeTypeOrderIdent) {
        Some(x) => match x { 
            LiteralOrIdents::Literal(x) => match x.to_string().parse() {
                Ok(x) => Some(x),
                Err(_) => return syn::Error::new(name.span(), "Expected u8 value for: SerializeTypeOrder".to_string()).to_compile_error().into(),
            },
            _ => return syn::Error::new(name.span(), "Expected Literal value for: SerializeTypeOrder".to_string()).to_compile_error().into(), 
        },
        None => None,
    };

    if serialize_id_order.is_some() && serialize_id_order == serialize_type_order {
        return syn::Error::new(name.span(), "SerializeIdOrder and SerializeTypeOrder cannot be equal".to_string()).to_compile_error().into()
    }
    
    let variants = match &input.data {
        Data::Enum(v) => &v.variants,
        _ => return syn::Error::new(name.span(), "Not an enum".to_string()).to_compile_error().into(),
    };
    let mut match_arms_to_string = Vec::new();
    let mut match_arms_get_id = Vec::new();
    let mut match_arms_get_type = Vec::new();
    let mut match_arms_get_data = Vec::new();
    let mut match_arms_from_bin = Vec::new();
    let mut match_arms_get_header1 = Vec::new();
    let mut match_arms_get_header2 = Vec::new();
    for variant in variants {
        let ident = &variant.ident;
        let params_dummy = match &variant.fields {
            Fields::Unit => quote! {},
            Fields::Unnamed(..) => quote! { (..) },
            Fields::Named(..) => quote! { {..} },
        };

        let params = match &variant.fields {
            Fields::Unit => (quote! {}, quote! {}),
            Fields::Unnamed(fields) => {
                let mut field_idents = Vec::new();
                for i in 0..fields.unnamed.len() {
                    field_idents.push(format_ident!("x{}", i));
                }
                (quote! { (#(#field_idents,)*) }, quote! { (#(#field_idents,)*) })
            },
            Fields::Named(fields) => {
                let mut field_idents = Vec::new();
                for field in &fields.named {
                    field_idents.push(field.ident.as_ref().unwrap());
                }
                (quote! { {#(#field_idents,)*} }, quote! { (#(#field_idents,)*) })
            },
        };

        let mut variant_attrs = variant.attrs.iter()
            .filter(|x| x.path().is_ident("command_enum_trait"));
        let parsed: Result<Vec<EnumVariantParamStruct>, syn::Error> = variant_attrs.try_fold(Vec::new(), |mut vec, attr| {
            vec.extend(attr.parse_args_with(Punctuated::<_, Token![,]>::parse_terminated)?);
            Ok(vec)
        });
        let parsed_map = match parsed {
            Err(e) => return e.to_compile_error().into(),
            Ok(parsed) => parsed.into_iter().map(|x| (x.key, x.value)).collect::<BTreeMap<_, _>>(),
        };
        let variant_id_ident = match parsed_map.get(&EnumVariantParams::CommandIdIdent) {
            Some(v) => v,
            None => return syn::Error::new(name.span(), "Missing variant enum param: command_id".to_string()).to_compile_error().into(),
        };
        let variant_type_ident = match parsed_map.get(&EnumVariantParams::CommandTypeIdent) {
            Some(v) => v,
            None => return syn::Error::new(name.span(), "Missing variant enum param: command_type".to_string()).to_compile_error().into(),
        };

        match_arms_to_string.push(quote! { #name::#ident #params_dummy => stringify!(#ident), });
        match_arms_get_id.push(quote! { #name::#ident #params_dummy => #variant_id_ident, });
        match_arms_get_type.push(quote! { #name::#ident #params_dummy => #variant_type_ident, });

        match &variant.fields {
            Fields::Unnamed(fields) => {
                let deserializers = ::core::iter::repeat(quote!(mzsh_commandenum::bincode::decode_from_reader(&mut *buffer, mzsh_commandenum::bincode::config::standard()).map_err(|e| e.to_string())?)).take(fields.unnamed.len());
                match_arms_from_bin.push(quote! { #variant_id_ident => Ok(#name::#ident(#(#deserializers),*)), })
            },
            Fields::Named(fields) => {
                let mut field_idents = Vec::new();
                for field in &fields.named {
                    field_idents.push(field.ident.as_ref().unwrap());
                }
                match_arms_from_bin.push(quote! { #variant_id_ident => Ok(#name::#ident{#(#field_idents: mzsh_commandenum::bincode::decode_from_reader(&mut *buffer, mzsh_commandenum::bincode::config::standard()).map_err(|e| e.to_string())?,)*}), })
            },
            Fields::Unit => match_arms_from_bin.push(quote! { #variant_id_ident => Ok(#name::#ident), }),
        }
        
        if !params.0.is_empty() {
            let left_params = params.0;
            let right_params = params.1;
            match_arms_get_data.push(quote! { #name::#ident #left_params => {mzsh_commandenum::bincode::encode_to_vec(& #right_params, mzsh_commandenum::bincode::config::standard()).map_err(|e| e.to_string())}, });
        }
        if let Some(order_id) = serialize_id_order {
            match serialize_type_order {
                Some(order_type) => match order_id < order_type {
                    true => match_arms_get_header1.push(quote!{ #name::#ident #params_dummy => {mzsh_commandenum::bincode::encode_to_vec(& self.get_id(), mzsh_commandenum::bincode::config::standard()).map_err(|e| e.to_string())}, }),
                    false => match_arms_get_header2.push(quote!{ #name::#ident #params_dummy => {mzsh_commandenum::bincode::encode_to_vec(& self.get_id(), mzsh_commandenum::bincode::config::standard()).map_err(|e| e.to_string())}, }),
                },
                None => match_arms_get_header1.push(quote!{ #name::#ident #params_dummy => {mzsh_commandenum::bincode::encode_to_vec(& self.get_id(), mzsh_commandenum::bincode::config::standard()).map_err(|e| e.to_string())}, }),
            }
        }
        if let Some(order_type) = serialize_type_order {
            match serialize_id_order {
                Some(order_id) => match order_id < order_type {                  
                    true => match_arms_get_header2.push(quote!{ #name::#ident #params_dummy => {mzsh_commandenum::bincode::encode_to_vec(& self.get_type(), mzsh_commandenum::bincode::config::standard()).map_err(|e| e.to_string())}, }),
                    false => match_arms_get_header1.push(quote!{ #name::#ident #params_dummy => {mzsh_commandenum::bincode::encode_to_vec(& self.get_type(), mzsh_commandenum::bincode::config::standard()).map_err(|e| e.to_string())}, }),
                },
                None => match_arms_get_header1.push(quote!{ #name::#ident #params_dummy => {mzsh_commandenum::bincode::encode_to_vec(& self.get_type(), mzsh_commandenum::bincode::config::standard()).map_err(|e| e.to_string())}, }),
            }
        }
    } 
    let mut stream_decode_header1 = quote!{};
    let mut stream_decode_header2 = quote!{};

    if let Some(order_id) = serialize_id_order {
        match serialize_type_order {
            Some(order_type) => match order_id < order_type {
                true => stream_decode_header1 = quote!{ id_value = mzsh_commandenum::bincode::decode_from_reader(&mut *buffer, mzsh_commandenum::bincode::config::standard()).ok(); },
                false => stream_decode_header2 = quote!{ id_value = mzsh_commandenum::bincode::decode_from_reader(&mut *buffer, mzsh_commandenum::bincode::config::standard()).ok(); },
            },
            None => stream_decode_header1 = quote!{ id_value = mzsh_commandenum::bincode::decode_from_reader(&mut *buffer, mzsh_commandenum::bincode::config::standard()).ok(); },
        }
    }
    if let Some(order_type) = serialize_type_order {
        match serialize_id_order {
            Some(order_id) => match order_id < order_type {
                true => stream_decode_header2 = quote!{ type_value = mzsh_commandenum::bincode::decode_from_reader(&mut *buffer, mzsh_commandenum::bincode::config::standard()).ok(); },
                false => stream_decode_header1 = quote!{ type_value = mzsh_commandenum::bincode::decode_from_reader(&mut *buffer, mzsh_commandenum::bincode::config::standard()).ok(); },
            },
            None => stream_decode_header1 = quote!{ type_value = mzsh_commandenum::bincode::decode_from_reader(&mut *buffer, mzsh_commandenum::bincode::config::standard()).ok(); },
        }
    }


    let expanded = quote! {
        
        impl #generics core::fmt::Display for #name #generics {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", match self {
                    #(#match_arms_to_string)*
                })
            }
        }

        impl #generics CommandEnumTrait<#id_ident, #type_ident> for #name #generics {
            fn get_id(&self) -> #id_ident
            {
                match self {
                    #(#match_arms_get_id)*
                }
            }

            fn get_type(&self) -> #type_ident
            {
                match self {
                    #(#match_arms_get_type)*
                }
            }

            fn encode_header(&self) -> Result<Vec<u8>, String>
            {
                let header1: Result<Vec<u8>, String> = match self {
                    #(#match_arms_get_header1)*
                    _ => Ok(vec![]),
                };
                let header2: Result<Vec<u8>, String> = match self {
                    #(#match_arms_get_header2)*
                    _ => Ok(vec![]),
                };
                let iter1: std::vec::IntoIter<u8> = header1?.into_iter();
                let iter2: std::vec::IntoIter<u8> = header2?.into_iter();
                Ok(iter1.chain(iter2).collect::<Vec<u8>>())
            }

            fn decode_header<R: bincode::de::read::Reader>(buffer: &mut R) -> (Option<#id_ident>, Option<#type_ident>)
            {
                let mut id_value = None;
                let mut type_value = None;
                #stream_decode_header1;
                #stream_decode_header2;
                (id_value, type_value)
            }

            fn encode_data(&self) -> Result<Vec<u8>, String>
            {
                match self {
                    #(#match_arms_get_data)*
                    _ => Ok(vec![]),
                }
            }

            fn decode_by_id<R: bincode::de::read::Reader>(id: #id_ident, buffer: &mut R) -> Result<Self, String>
            {
                match id {
                    #(#match_arms_from_bin)*
                    _ => Err(format!("Missing enum variant for id `{:?}`", id)),
                }
            }
        }


        impl #generics bincode::Encode for #name #generics
        {
            fn encode<E: bincode::enc::Encoder>(&self, encoder: &mut E) -> Result<(), bincode::error::EncodeError> {
                bincode::enc::write::Writer::write(
                    encoder.writer(), 
                    self.encode_header().map_err(|e| bincode::error::EncodeError::OtherString(e.to_string()))?.as_slice()
                )?;
                bincode::enc::write::Writer::write(
                    encoder.writer(), 
                    self.encode_data().map_err(|e| bincode::error::EncodeError::OtherString(e.to_string()))?.as_slice()
                )
            }
        }

        impl #generics bincode::Decode for #name #generics
        {
            fn decode<D: bincode::de::Decoder>(decoder: &mut D) -> Result<Self, bincode::error::DecodeError> {
                let (header_id, _) = Self::decode_header(decoder.reader());
                if let Some(id) = header_id {
                    Self::decode_by_id(id, decoder.reader()).map_err(|e| bincode::error::DecodeError::OtherString(e.to_string()))
                }
                else  {
                    Err(bincode::error::DecodeError::OtherString(format!("Enum {} doesnt encode id, decoding is not possible", stringify!(#name))))
                }
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}
