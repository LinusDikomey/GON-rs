#![allow(unused_variables)] // quote doesn't seem to 'use' variables properly

use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, DeriveInput, Generics, GenericParam, parse_quote, Data, Fields, spanned::Spanned};




#[proc_macro_derive(FromGon)]
pub fn derive_from_gon(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let from_body = from_gon(&input.data);

    let expanded = quote! {
        impl #impl_generics gon_rs::from::FromGon for #name #ty_generics #where_clause {
            fn from_gon(gon: &gon_rs::Gon) -> std::result::Result<Self, gon_rs::from::FromGonError> {
                #from_body
            }
        }
    };
    proc_macro::TokenStream::from(expanded)
}

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(gon_rs::from::FromGon));
        }
    }
    generics
}

fn from_gon(data: &Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Named(fields) => {
                    let recurse = fields.named.iter().map(|f| {
                        let name = &f.ident;
                        let name_str = name.as_ref().unwrap().to_string();
                        quote_spanned! {f.span()=>
                            #name: gon_rs::from::FromGon::from_gon(map.get(#name_str).ok_or(gon_rs::from::FromGonError::Missing(&&#name_str))?)?,
                        }
                    });
                    quote! {
                        match gon {
                            gon_rs::Gon::Array(_) | gon_rs::Gon::Value(_) => std::result::Result::Err(gon_rs::from::FromGonError::ExpectedObject),
                            gon_rs::Gon::Object(map) => std::result::Result::Ok(Self {
                                #( #recurse )*
                            })
                        }
                    }
                }
                Fields::Unnamed(fields) => {
                    let count = fields.unnamed.len();
                    let recurse = fields.unnamed.iter().enumerate().map(|(i, f)| {
                        //let index = Index::from(i);
                        quote_spanned! {f.span()=>
                            gon_rs::from::FromGon::from_gon(&arr[#i])
                        }
                    });
                    quote! {
                        match gon {
                            gon_rs::Gon::Object(_) | gon_rs::Gon::Value(_) => std::result::Result::Err(gon_rs::from::FromGonError::ExpectedArray),
                            gon_rs::Gon::Array(arr) => {
                                if arr.len() != #count {
                                    return std::result::Result::Err(gon_rs::from::FromGonError::InvalidLength { expected: #count, found: arr.len() });
                                }
                                Self(#( #recurse )*)
                            }
                        }
                    }
                }
                Fields::Unit => {
                    quote! {
                        match gon {
                            gon_rs::Gon::Array(_) | gon_rs::Gon::Value(_) => std::result::Result::Err(gon_rs::from::FromGonError::ExpectedObject),
                            gon_rs::Gon::Object(_) => std::result::Result::Ok(Self)
                        }
                    }
                }
            }
        }
        Data::Enum(data_enum) => {
            let recurse = data_enum.variants.iter().map(|v| {
                assert!(matches!(v.fields, Fields::Unit), "No enum fields supported for now.");
                
                let ident = &v.ident;
                let str_val = ident.to_string();

                quote! { #str_val => std::result::Result::Ok(Self::#ident), }
            });

            quote! {
                match gon {
                    gon_rs::Gon::Object(_) | gon_rs::Gon::Array(_) => std::result::Result::Err(gon_rs::from::FromGonError::ExpectedValue),
                    gon_rs::Gon::Value(val) => match val.as_str() {
                        #( #recurse )*
                        _ =>  std::result::Result::Err(gon_rs::from::FromGonError::UnexpectedValue(val.to_owned()))
                    }
                }
            }
        }
        Data::Union(_) => panic!("No union support for #[derive(FromGon)]"),
    }
}