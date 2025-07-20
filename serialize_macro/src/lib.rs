use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Data, Fields};

#[proc_macro_derive(SerializeNumberStruct)]
pub fn serialise_number_struct(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;

    let serialize_fields = match &ast.data {
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Named(fields) => {
                    let field_serializations = fields.named.iter().map(|field| {
                        if let syn::Type::Path(type_path) = &field.ty {
                            if let Some(ident) = type_path.path.get_ident() {
                                let ty_str = ident.to_string();
                                match ty_str.as_str() {
                                    "i8" | "u8" | "i16" | "u16" | "i32" | "u32" | "i64" | "u64" | "isize" | "usize" => {},
                                    _ => panic!("Unsupported field type: {}", ty_str),
                                }
                            } else {
                                panic!("Complex types not supported");
                            }
                        } else {
                            panic!("Non-path types not supported");
                        }
                        let field_name = &field.ident;
                        quote! {
                            result.extend_from_slice(&self.#field_name.to_be_bytes());
                        }
                    });
                    quote! {
                        #(#field_serializations)*
                    }
                }
                _ => panic!("Only named fields are supported"),
            }
        }
        _ => panic!("Only structs are supported"),
    };

    let generated = quote! {
        impl Serialize for #name {
            fn serialize(&self) -> Vec<u8> {
                let mut result = Vec::new();
                #serialize_fields
                result
            }
        }
    };
    generated.into()
}

#[proc_macro_derive(DeserializeNumberStruct)]
pub fn deserialise_number_struct(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;

    let (deserialize_fields, field_assignments, total_size) = match &ast.data {
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Named(fields) => {
                    let mut offset: usize = 0;
                    let mut field_deserializations = Vec::new();
                    let mut field_assignments = Vec::new();
                    
                    for field in &fields.named {
                        let field_name = &field.ident;
                        let ty = &field.ty;
                        let field_ty_str = if let syn::Type::Path(type_path) = ty {
                            type_path.path.get_ident().map(|i| i.to_string()).unwrap_or_else(|| panic!("Unsupported type"))
                        } else {
                            panic!("Non-path types not supported")
                        };
                        let field_size: usize = match field_ty_str.as_str() {
                            "i8" | "u8" => 1,
                            "i16" | "u16" => 2,
                            "i32" | "u32" => 4,
                            "i64" | "u64" | "isize" | "usize" => 8,
                            _ => panic!("Unsupported field type: {}", field_ty_str),
                        };
                        let start_offset = offset;
                        let end_offset = offset + field_size;
                        
                        field_deserializations.push(quote! {
                            let #field_name: #ty = {
                                let bytes: [u8; #field_size] = base[#start_offset..#end_offset]
                                    .try_into()
                                    .map_err(|_| Error)?;
                                #ty::from_be_bytes(bytes)
                            };
                        });
                        
                        field_assignments.push(quote! {
                            #field_name
                        });
                        
                        offset += field_size;
                    }
                    
                    (field_deserializations, field_assignments, offset)
                }
                _ => panic!("Only named fields are supported"),
            }
        }
        _ => panic!("Only structs are supported"),
    };

    let generated = quote! {
        impl Deserialize for #name {
            fn deserialize(base: &[u8]) -> Result<Self, Error> {
                if base.len() < #total_size {
                    return Err(Error);
                }
                
                #(#deserialize_fields)*
                
                Ok(#name {
                    #(#field_assignments,)*
                })
            }
        }
    };
    generated.into()
}