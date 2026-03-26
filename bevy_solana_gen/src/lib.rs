use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use serde::Deserialize; // This needs 'features = ["derive"]' in Cargo.toml
use std::{env, fs, path::PathBuf};
use syn::{Ident, LitStr, parse_macro_input};

// --- PRIVATE HELPER STRUCTS ---
// These are NOT 'pub' so they don't violate proc-macro rules.
#[derive(Deserialize, Debug)]
struct AnchorIdl {
    address: String,
    #[serde(default)]
    accounts: Vec<IdlAccount>,
    #[serde(default)]
    instructions: Vec<IdlInstruction>,
    #[serde(default)]
    types: Vec<IdlDefinedType>,
}

#[derive(Deserialize, Debug)]
struct IdlInstruction {
    name: String,
    #[serde(default)]
    discriminator: Vec<u8>,
    #[serde(default)]
    accounts: Vec<IdlInstructionAccountItem>,
    #[serde(default)]
    args: Vec<IdlField>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum IdlInstructionAccountItem {
    Account(IdlInstructionAccount),
    Group(IdlInstructionAccountsGroup),
}

#[derive(Deserialize, Debug)]
struct IdlInstructionAccountsGroup {
    #[serde(default)]
    _name: String,
    #[serde(default)]
    accounts: Vec<IdlInstructionAccountItem>,
}

#[derive(Deserialize, Debug)]
struct IdlInstructionAccount {
    name: String,
    #[serde(default)]
    writable: bool,
    #[serde(default)]
    signer: bool,
    #[allow(dead_code)]
    #[serde(default)]
    address: Option<String>,
}

#[derive(Deserialize, Debug)]
struct IdlAccount {
    name: String,
    // In 0.31, the discriminator is usually here, but the 'fields' might be in 'types'
}

#[derive(Deserialize, Debug)]
struct IdlDefinedType {
    name: String,
    #[serde(rename = "type")]
    ty: IdlType,
}

#[derive(Deserialize, Debug)]
struct IdlType {
    #[serde(rename = "kind", default)]
    _kind: String,
    #[serde(default)]
    fields: Vec<IdlField>,
}

#[derive(Deserialize, Debug)]
struct IdlField {
    name: String,
    #[serde(rename = "type")]
    ty: String,
}

fn pascal_case_ident(name: &str) -> Ident {
    let mut chars = name.chars();
    let name_str = match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    };

    Ident::new(&name_str, Span::call_site())
}

fn idl_field_type_tokens(ty: &str) -> TokenStream2 {
    match ty {
        "u64" => quote! { u64 },
        "i64" => quote! { i64 },
        "string" => quote! { String },
        "bool" => quote! { bool },
        "publicKey" => quote! { bevy_solana_core::WrappedPubkey },
        _ => quote! { u64 },
    }
}

fn idl_instruction_arg_type_tokens(ty: &str) -> TokenStream2 {
    match ty {
        "u64" => quote! { u64 },
        "i64" => quote! { i64 },
        "string" => quote! { String },
        "bool" => quote! { bool },
        "publicKey" => quote! { solana_sdk::pubkey::Pubkey },
        _ => quote! { u64 },
    }
}

fn idl_arg_serializer_tokens(field_name: &Ident, ty: &str) -> TokenStream2 {
    match ty {
        "u64" => quote! {
            data.extend_from_slice(&#field_name.to_le_bytes());
        },
        "i64" => quote! {
            data.extend_from_slice(&#field_name.to_le_bytes());
        },
        "string" => quote! {
            let bytes = #field_name.as_bytes();
            let len = u32::try_from(bytes.len()).expect("string argument too large for Anchor serialization");
            data.extend_from_slice(&len.to_le_bytes());
            data.extend_from_slice(bytes);
        },
        "bool" => quote! {
            data.push(u8::from(#field_name));
        },
        "publicKey" => quote! {
            data.extend_from_slice(&#field_name.to_bytes());
        },
        _ => quote! {
            compile_error!("Unsupported IDL argument type for generated instruction builder");
        },
    }
}

fn flatten_instruction_accounts<'a>(
    items: &'a [IdlInstructionAccountItem],
    flattened: &mut Vec<&'a IdlInstructionAccount>,
) {
    for item in items {
        match item {
            IdlInstructionAccountItem::Account(account) => flattened.push(account),
            IdlInstructionAccountItem::Group(group) => {
                flatten_instruction_accounts(&group.accounts, flattened);
            }
        }
    }
}

// --- THE ACTUAL MACRO ---
#[proc_macro]
pub fn generate_bevy_components(input: TokenStream) -> TokenStream {
    let path_lit = parse_macro_input!(input as LitStr);
    let path = path_lit.value();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .expect("ERROR: CARGO_MANIFEST_DIR is unavailable during macro expansion.");
    let idl_path = PathBuf::from(manifest_dir).join(path);

    // Read the IDL file from the path provided in the macro call
    let idl_content = fs::read_to_string(&idl_path)
        .expect("ERROR: Could not find the Anchor IDL file. Check your path.");

    let idl: AnchorIdl = serde_json::from_str(&idl_content)
        .expect("ERROR: Failed to parse IDL JSON. Ensure it matches Anchor 0.31 format.");
    let program_address = LitStr::new(&idl.address, Span::call_site());

    let mut generated_code = quote! {};

    for account in &idl.accounts {
        let name = Ident::new(&account.name, Span::call_site());

        // 1. Find the type definition that matches the account name
        // Anchor 0.31 separates 'accounts' from the actual 'type' definitions
        let matching_type = idl.types.iter().find(|t| t.name == account.name);

        if let Some(idl_type_wrapper) = matching_type {
            let fields: Vec<_> = idl_type_wrapper
                .ty
                .fields
                .iter()
                .map(|f| {
                    let field_name = Ident::new(&f.name, Span::call_site());
                    let field_type = idl_field_type_tokens(&f.ty);

                    quote! { pub #field_name: #field_type }
                })
                .collect(); // .collect() is vital for the quote! macro to infer the type

            let struct_gen = quote! {
                #[derive(bevy::prelude::Component, bevy::prelude::Reflect, Default, Debug, Clone)]
                #[reflect(Component)]
                pub struct #name {
                    #( #fields, )*
                }
            };
            generated_code.extend(struct_gen);
        }
    }

    // --- Generate instruction structs and builder functions ---
    for instr in &idl.instructions {
        let name = pascal_case_ident(&instr.name);
        let builder_name = Ident::new(&instr.name, Span::call_site());
        // Phase 2: Create a name for the Accounts Struct
        let accounts_struct_name = format_ident!("{}Accounts", name);

        // Existing Arg Fields
        let fields: Vec<_> = instr
            .args
            .iter()
            .map(|arg| {
                let field_name = Ident::new(&arg.name, Span::call_site());
                let field_type = idl_field_type_tokens(&arg.ty);
                quote! { pub #field_name: #field_type }
            })
            .collect();

        // Function arguments for the builder
        let function_args: Vec<_> = instr
            .args
            .iter()
            .map(|arg| {
                let arg_name = Ident::new(&arg.name, Span::call_site());
                let arg_type = idl_instruction_arg_type_tokens(&arg.ty);
                quote! { #arg_name: #arg_type }
            })
            .collect();

        let mut flattened_accounts = Vec::new();
        flatten_instruction_accounts(&instr.accounts, &mut flattened_accounts);

        // Phase 2: Generate the fields for the ACCOUNTS struct
        let account_struct_fields: Vec<_> = flattened_accounts
            .iter()
            .map(|acc| {
                let acc_name = Ident::new(&acc.name, Span::call_site());
                quote! { pub #acc_name: solana_sdk::pubkey::Pubkey }
            })
            .collect();

        // Phase 2: Generate the AccountMeta mapping inside the builder
        let account_metas: Vec<_> = flattened_accounts
            .iter()
            .map(|acc| {
                let constructor = if acc.writable {
                    quote! { solana_sdk::instruction::AccountMeta::new }
                } else {
                    quote! { solana_sdk::instruction::AccountMeta::new_readonly }
                };
                let signer = acc.signer;
                let acc_name = Ident::new(&acc.name, Span::call_site());
                // Phase 2: All AccountMeta keys come from the typed accounts struct.
                quote! { #constructor(accounts.#acc_name, #signer) }
            })
            .collect();

        // ... Keep your existing discriminator and arg_serializers logic here ...
        let discriminator = &instr.discriminator;
        let arg_serializers: Vec<_> = instr
            .args
            .iter()
            .map(|arg| {
                let arg_name = Ident::new(&arg.name, Span::call_site());
                idl_arg_serializer_tokens(&arg_name, &arg.ty)
            })
            .collect();

        let instr_gen: TokenStream2 = quote! {
            // The Component for the instruction data
            #[derive(bevy::prelude::Component, Debug, Clone, Default)]
            pub struct #name {
                #( #fields, )*
            }

            // Phase 2: The new Accounts Parameter Struct
            #[derive(Debug, Clone, Copy)]
            pub struct #accounts_struct_name {
                #( #account_struct_fields, )*
            }

            // Phase 2: Updated builder signature
            pub fn #builder_name(
                accounts: #accounts_struct_name,
                #( #function_args ),*
            ) -> solana_sdk::instruction::Instruction {
                let mut data = Vec::with_capacity(8);
                data.extend_from_slice(&[#( #discriminator ),*]);
                #( #arg_serializers )*

                solana_sdk::instruction::Instruction {
                    program_id: solana_sdk::pubkey!(#program_address),
                    accounts: vec![#( #account_metas ),*],
                    data,
                }
            }
        };
        generated_code.extend(instr_gen);
    }

    generated_code.into()
}
