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
                        let field_name = &field.ident;
                        let ty = &field.ty;
                        if let syn::Type::Path(type_path) = ty {
                            if let Some(ident) = type_path.path.get_ident() {
                                let ty_str = ident.to_string();
                                match ty_str.as_str() {
                                    "String" => quote! {
                                        let bytes = self.#field_name.as_bytes();
                                        result.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
                                        result.extend_from_slice(bytes);
                                    },
                                    "i8" | "u8" | "i16" | "u16" | "i32" | "u32" | "i64" | "u64" | "isize" | "usize" => quote! {
                                        result.extend_from_slice(&self.#field_name.to_be_bytes());
                                    },
                                    _ => panic!("Unsupported field type: {}", ty_str),
                                }
                            } else {
                                panic!("Complex types not supported");
                            }
                        } else {
                            panic!("Non-path types not supported");
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

    let deserialize_fields = match &ast.data {
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Named(fields) => {
                    let mut field_deserializations = Vec::new();
                    let mut field_assignments = Vec::new();
                    let mut fixed_size: usize = 0; // For upfront check of minimum size (numbers + length prefixes)

                    let mut offset_var = quote! { let mut offset: usize = 0; };

                    for field in &fields.named {
                        let field_name = &field.ident;
                        let ty = &field.ty;
                        let field_ty_str = if let syn::Type::Path(type_path) = ty {
                            type_path.path.get_ident().map(|i| i.to_string()).unwrap_or_else(|| panic!("Unsupported type"))
                        } else {
                            panic!("Non-path types not supported")
                        };
                        
                        match field_ty_str.as_str() {
                            "String" => {
                                fixed_size += 4; // length prefix
                                field_deserializations.push(quote! {
                                    if base.len() < offset + 4 {
                                        return Err(Error);
                                    }
                                    let len_bytes: [u8; 4] = base[offset..offset + 4].try_into().map_err(|_| Error)?;
                                    let len = u32::from_be_bytes(len_bytes) as usize;
                                    offset += 4;
                                    if base.len() < offset + len {
                                        return Err(Error);
                                    }
                                    let #field_name = String::from_utf8(base[offset..offset + len].to_vec()).map_err(|_| Error)?;
                                    offset += len;
                                });
                            },
                            "i8" | "u8" => {
                                fixed_size += 1;
                                field_deserializations.push(quote! {
                                    if base.len() < offset + 1 {
                                        return Err(Error);
                                    }
                                    let bytes: [u8; 1] = base[offset..offset + 1].try_into().map_err(|_| Error)?;
                                    let #field_name: #ty = #ty::from_be_bytes(bytes);
                                    offset += 1;
                                });
                            },
                            "i16" | "u16" => {
                                fixed_size += 2;
                                field_deserializations.push(quote! {
                                    if base.len() < offset + 2 {
                                        return Err(Error);
                                    }
                                    let bytes: [u8; 2] = base[offset..offset + 2].try_into().map_err(|_| Error)?;
                                    let #field_name: #ty = #ty::from_be_bytes(bytes);
                                    offset += 2;
                                });
                            },
                            "i32" | "u32" => {
                                fixed_size += 4;
                                field_deserializations.push(quote! {
                                    if base.len() < offset + 4 {
                                        return Err(Error);
                                    }
                                    let bytes: [u8; 4] = base[offset..offset + 4].try_into().map_err(|_| Error)?;
                                    let #field_name: #ty = #ty::from_be_bytes(bytes);
                                    offset += 4;
                                });
                            },
                            "i64" | "u64" | "isize" | "usize" => {
                                fixed_size += 8;
                                field_deserializations.push(quote! {
                                    if base.len() < offset + 8 {
                                        return Err(Error);
                                    }
                                    let bytes: [u8; 8] = base[offset..offset + 8].try_into().map_err(|_| Error)?;
                                    let #field_name: #ty = #ty::from_be_bytes(bytes);
                                    offset += 8;
                                });
                            },
                            _ => panic!("Unsupported field type: {}", field_ty_str),
                        };
                        
                        field_assignments.push(quote! {
                            #field_name
                        });
                    }
                    
                    quote! {
                        #offset_var
                        #(#field_deserializations)*
                        if offset != base.len() {
                            return Err(Error);
                        }
                        Ok(#name {
                            #(#field_assignments,)*
                        })
                    }
                }
                _ => panic!("Only named fields are supported"),
            }
        }
        _ => panic!("Only structs are supported"),
    };

    let generated = quote! {
        impl Deserialize for #name {
            fn deserialize(base: &[u8]) -> Result<Self, Error> {
                #deserialize_fields
            }
        }
    };
    generated.into()
}