use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::Nothing,
    parse_macro_input,
    Data::{Enum, Struct},
    DataEnum, DataStruct, DeriveInput, FieldsNamed, Variant,
};

fn variants_from(
    tokens: proc_macro2::TokenStream,
) -> syn::Result<syn::punctuated::Punctuated<Variant, syn::token::Comma>> {
    let default_ast: DeriveInput = syn::parse2(tokens)?;
    match default_ast.data {
        Enum(DataEnum { variants, .. }) => Ok(variants),
        _ => panic!("only enums can provide variants"),
    }
}

fn extend_enum_with_variants_impl(
    input: proc_macro2::TokenStream,
    default_variants: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let mut input_ast: DeriveInput = syn::parse2(input).expect("parse input enum");
    let variants = variants_from(default_variants).expect("parse default variants");
    let input_variants_data = match &mut input_ast.data {
        Enum(DataEnum { variants, .. }) => variants,
        _ => panic!("only enums can accept variants"),
    };
    input_variants_data.extend(variants);
    quote! { #input_ast }
}

fn ownables_attach_impl(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    extend_enum_with_variants_impl(
        input,
        quote! {
            enum ExecuteMsg {
                Attach { attachments: Vec<AttachmentInput> },
            }
        },
    )
    .into()
}

fn ownables_close_impl(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    extend_enum_with_variants_impl(
        input,
        quote! {
            enum ExecuteMsg {
                Close {},
            }
        },
    )
    .into()
}

fn ownables_query_attachments_impl(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    extend_enum_with_variants_impl(
        input,
        quote! {
            enum QueryMsg {
                GetAttachments {},
            }
        },
    )
    .into()
}

fn ownables_query_closed_impl(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    extend_enum_with_variants_impl(
        input,
        quote! {
            enum QueryMsg {
                IsClosed {},
            }
        },
    )
    .into()
}

/// Adds `Transfer { to: Addr }` to an `ExecuteMsg` enum.
#[proc_macro_attribute]
pub fn ownables_transfer(metadata: TokenStream, input: TokenStream) -> TokenStream {
    parse_macro_input!(metadata as Nothing);
    extend_enum_with_variants_impl(
        input.into(),
        quote! {
            enum ExecuteMsg {
                Transfer {to: Addr},
            }
        },
    )
    .into()
}

/// Adds `Lock {}` to an `ExecuteMsg` enum.
#[proc_macro_attribute]
pub fn ownables_lock(metadata: TokenStream, input: TokenStream) -> TokenStream {
    parse_macro_input!(metadata as Nothing);
    extend_enum_with_variants_impl(
        input.into(),
        quote! {
            enum ExecuteMsg {
                Lock {},
            }
        },
    )
    .into()
}

/// Adds `Consume {}` to an `ExecuteMsg` enum.
#[proc_macro_attribute]
pub fn ownables_consume(metadata: TokenStream, input: TokenStream) -> TokenStream {
    parse_macro_input!(metadata as Nothing);
    extend_enum_with_variants_impl(
        input.into(),
        quote! {
            enum ExecuteMsg {
                Consume {},
            }
        },
    )
    .into()
}

/// Adds `Attach { attachments: Vec<AttachmentInput> }` to an `ExecuteMsg` enum.
#[proc_macro_attribute]
pub fn ownables_attach(metadata: TokenStream, input: TokenStream) -> TokenStream {
    parse_macro_input!(metadata as Nothing);
    ownables_attach_impl(input.into()).into()
}

/// Adds `Close {}` to an `ExecuteMsg` enum.
#[proc_macro_attribute]
pub fn ownables_close(metadata: TokenStream, input: TokenStream) -> TokenStream {
    parse_macro_input!(metadata as Nothing);
    ownables_close_impl(input.into()).into()
}

/// Adds `GetMetadata {}` to a `QueryMsg` enum.
#[proc_macro_attribute]
pub fn ownables_query_metadata(metadata: TokenStream, input: TokenStream) -> TokenStream {
    parse_macro_input!(metadata as Nothing);
    extend_enum_with_variants_impl(
        input.into(),
        quote! {
            enum QueryMsg {
                GetMetadata {},
            }
        },
    )
    .into()
}

/// Adds `GetInfo {}` to a `QueryMsg` enum.
#[proc_macro_attribute]
pub fn ownables_query_info(metadata: TokenStream, input: TokenStream) -> TokenStream {
    parse_macro_input!(metadata as Nothing);
    extend_enum_with_variants_impl(
        input.into(),
        quote! {
            enum QueryMsg {
                GetInfo {},
            }
        },
    )
    .into()
}

/// Adds `GetWidgetState {}` to a `QueryMsg` enum.
#[proc_macro_attribute]
pub fn ownables_query_widget_state(metadata: TokenStream, input: TokenStream) -> TokenStream {
    parse_macro_input!(metadata as Nothing);
    extend_enum_with_variants_impl(
        input.into(),
        quote! {
            enum QueryMsg {
                GetWidgetState {},
            }
        },
    )
    .into()
}

/// Adds `IsLocked {}` to a `QueryMsg` enum.
#[proc_macro_attribute]
pub fn ownables_query_locked(metadata: TokenStream, input: TokenStream) -> TokenStream {
    parse_macro_input!(metadata as Nothing);
    extend_enum_with_variants_impl(
        input.into(),
        quote! {
            enum QueryMsg {
                IsLocked {},
            }
        },
    )
    .into()
}

/// Adds `IsConsumed {}` to a `QueryMsg` enum.
#[proc_macro_attribute]
pub fn ownables_query_consumed(metadata: TokenStream, input: TokenStream) -> TokenStream {
    parse_macro_input!(metadata as Nothing);
    extend_enum_with_variants_impl(
        input.into(),
        quote! {
            enum QueryMsg {
                IsConsumed {},
            }
        },
    )
    .into()
}

/// Adds `GetAttachments {}` to a `QueryMsg` enum.
#[proc_macro_attribute]
pub fn ownables_query_attachments(metadata: TokenStream, input: TokenStream) -> TokenStream {
    parse_macro_input!(metadata as Nothing);
    ownables_query_attachments_impl(input.into()).into()
}

/// Adds `IsClosed {}` to a `QueryMsg` enum.
#[proc_macro_attribute]
pub fn ownables_query_closed(metadata: TokenStream, input: TokenStream) -> TokenStream {
    parse_macro_input!(metadata as Nothing);
    ownables_query_closed_impl(input.into()).into()
}

/// Adds `IsConsumerOf { issuer: Addr, consumable_type: String }` to a `QueryMsg` enum.
#[proc_macro_attribute]
pub fn ownables_query_consumer_of(metadata: TokenStream, input: TokenStream) -> TokenStream {
    parse_macro_input!(metadata as Nothing);
    extend_enum_with_variants_impl(
        input.into(),
        quote! {
            enum QueryMsg {
                IsConsumerOf {
                    issuer: Addr,
                    consumable_type: String,
                },
            }
        },
    )
    .into()
}

/// Adds default ownables fields to an `InstantiateMsg` struct:
/// InstantiateMsg {
///     pub ownable_id: String,
///     pub package: String,
///     pub nft: Option<NFT>,
///     pub ownable_type: Option<String>,
///     pub network_id: u8,
/// }
#[proc_macro_attribute]
pub fn ownables_instantiate_msg(metadata: TokenStream, input: TokenStream) -> TokenStream {
    parse_macro_input!(metadata as Nothing);

    let default_instantiate_fields: TokenStream = quote! {
        struct InstantiateMsg {
            pub ownable_id: String,
            pub package: String,
            pub nft: Option<NFT>,
            pub ownable_type: Option<String>,
            pub network_id: u32,
        }
    }
    .into();

    let default_ast: DeriveInput = parse_macro_input!(default_instantiate_fields);
    let default_fields = match default_ast.data {
        Struct(DataStruct { fields, .. }) => fields,
        _ => panic!("only structs can accept fields"),
    };

    let mut input_ast: DeriveInput = parse_macro_input!(input);
    let input_fields_data = match &mut input_ast.data {
        Struct(DataStruct { fields, .. }) => fields,
        _ => panic!("only structs can accept fields"),
    };

    if let syn::Fields::Named(FieldsNamed { named, .. }) = input_fields_data {
        named.extend(default_fields);
    }

    quote! { #input_ast }.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::ToTokens;
    use syn::{Data, Fields};

    fn parse_enum(input: proc_macro2::TokenStream) -> syn::DataEnum {
        let parsed: DeriveInput = syn::parse2(input).expect("parse enum");
        match parsed.data {
            Data::Enum(data) => data,
            _ => panic!("expected enum"),
        }
    }

    fn find_variant<'a>(data: &'a syn::DataEnum, name: &str) -> &'a Variant {
        data.variants
            .iter()
            .find(|variant| variant.ident == name)
            .unwrap_or_else(|| panic!("missing variant {name}"))
    }

    #[test]
    fn ownables_attach_injects_attachment_input_vector() {
        let data = parse_enum(ownables_attach_impl(quote! {
            enum ExecuteMsg {
                Existing {},
            }
        }));
        let variant = find_variant(&data, "Attach");

        match &variant.fields {
            Fields::Named(fields) => {
                assert_eq!(fields.named.len(), 1);
                let field = &fields.named[0];
                assert_eq!(field.ident.as_ref().expect("named field"), "attachments");
                assert_eq!(
                    field.ty.to_token_stream().to_string(),
                    "Vec < AttachmentInput >"
                );
            }
            _ => panic!("Attach should have named fields"),
        }
    }

    #[test]
    fn ownables_close_injects_close_variant() {
        let data = parse_enum(ownables_close_impl(quote! {
            enum ExecuteMsg {
                Existing {},
            }
        }));
        let variant = find_variant(&data, "Close");

        match &variant.fields {
            Fields::Named(fields) => assert!(fields.named.is_empty()),
            _ => panic!("Close should use named fields"),
        }
    }

    #[test]
    fn ownables_query_attachments_injects_get_attachments_variant() {
        let data = parse_enum(ownables_query_attachments_impl(quote! {
            enum QueryMsg {
                Existing {},
            }
        }));
        let variant = find_variant(&data, "GetAttachments");

        match &variant.fields {
            Fields::Named(fields) => assert!(fields.named.is_empty()),
            _ => panic!("GetAttachments should use named fields"),
        }
    }

    #[test]
    fn ownables_query_closed_injects_is_closed_variant() {
        let data = parse_enum(ownables_query_closed_impl(quote! {
            enum QueryMsg {
                Existing {},
            }
        }));
        let variant = find_variant(&data, "IsClosed");

        match &variant.fields {
            Fields::Named(fields) => assert!(fields.named.is_empty()),
            _ => panic!("IsClosed should use named fields"),
        }
    }
}
