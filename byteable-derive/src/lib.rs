use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, parse_macro_input};

#[proc_macro_derive(Byteable)]
pub fn derive_byteable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    match input.data {
        Data::Enum(data_enum) => derive_byteable_enum(data_enum, name),
        Data::Struct(data_struct) => derive_byteable_struct(data_struct, name),
        _ => panic!("Byteable can only be derived for enums and structs"),
    }
}

fn derive_byteable_struct(data_struct: syn::DataStruct, name: syn::Ident) -> TokenStream {
    enum FieldKind {
        Named,
        Unnamed,
        Unit,
    }

    let fields = data_struct.fields;

    let kind = match fields {
        syn::Fields::Named(_) => FieldKind::Named,
        syn::Fields::Unnamed(_) => FieldKind::Unnamed,
        syn::Fields::Unit => FieldKind::Unit,
    };

    let types = fields.iter().map(|f| f.ty.clone()).collect::<Vec<_>>();

    let (encode_section, decode_section) = match kind {
        FieldKind::Named => {
            let field_idents = fields
                .iter()
                .map(|f| f.ident.as_ref().unwrap().clone())
                .collect::<Vec<_>>();

            let encode_section = quote! {
                #(
                    self.#field_idents.encode(writer).await?;
                )*
            };

            let decode_section = quote! {
                {
                    #(
                        #field_idents: <#types as crate::helpers::Byteable>::decode(reader).await?,
                    )*
                }
            };

            (encode_section, decode_section)
        }
        FieldKind::Unnamed => {
            let enumeration = (0..types.len()).collect::<Vec<_>>();
            let encode_section = quote! {
                #(
                    self.#enumeration.encode(writer).await?;
                )*
            };
            let decode_section = quote! {
                (
                    #(
                        <#types as crate::helpers::Byteable>::decode(reader).await?,
                    )*
                )
            };

            (encode_section, decode_section)
        }

        FieldKind::Unit => (quote! {}, quote! {}),
    };

    let expanded = quote! {
        impl crate::helpers::Byteable for #name {
            async fn encode<W: tokio::io::AsyncWrite + Unpin + Send>(
                &self,
                writer: &mut W
            ) -> Result<(), crate::errors::EncodeError> {
                #encode_section
                Ok(())
            }

            async fn decode<R: tokio::io::AsyncRead + Unpin + Send>(
                reader: &mut R
            ) -> Result<Self, crate::errors::DecodeError> {
                Ok(#name #decode_section)
            }
        }
    };

    expanded.into()
}

fn derive_byteable_enum(data_enum: syn::DataEnum, name: syn::Ident) -> TokenStream {
    let mut decode_match_arms = Vec::new();
    let mut next_auto_discriminant = 0u8;

    for variant in data_enum.variants {
        let ident = &variant.ident;
        let value = if let Some((_, expr)) = &variant.discriminant {
            // Evaluate the discriminant as a literal integer
            let lit_value = match expr {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Int(lit),
                    ..
                }) => lit.base10_parse::<u8>().unwrap(),
                _ => panic!("Unsupported discriminant expression, only u8 is supported"),
            };
            next_auto_discriminant = lit_value + 1;
            quote! { #lit_value }
        } else {
            let v = next_auto_discriminant;
            next_auto_discriminant += 1;
            quote! { #v }
        };

        decode_match_arms.push(quote! {
            #value => Ok(#name::#ident),
        });
    }

    let expanded = quote! {
        impl crate::helpers::Byteable for #name {
            async fn encode<W: tokio::io::AsyncWrite + Unpin + Send>(
                &self,
                writer: &mut W
            ) -> Result<(), crate::errors::EncodeError> {
                writer.write_u8((self.clone() as u8)).await?;
                Ok(())
            }

            async fn decode<R: tokio::io::AsyncRead + Unpin + Send>(
                reader: &mut R
            ) -> Result<Self, crate::errors::DecodeError> {
                let variant = reader.read_u8().await?;

                match variant {
                    #(#decode_match_arms)*
                    other => Err(DecodeError::InvalidEnumVariant {
                        variant_value: other.to_string(),
                        enum_name: stringify!(#name),
                    }),
                }
            }
        }
    };

    expanded.into()
}
