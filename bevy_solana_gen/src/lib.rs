use proc_macro::TokenStream;
use quote::quote;
use serde::Deserialize; // This needs 'features = ["derive"]' in Cargo.toml
use std::{env, fs, path::PathBuf};
use syn::{Ident, LitStr, parse_macro_input};

// --- PRIVATE HELPER STRUCTS ---
// These are NOT 'pub' so they don't violate proc-macro rules.
#[derive(Deserialize, Debug)]
struct AnchorIdl {
    #[serde(default)]
    _address: String,
    #[serde(default)]
    accounts: Vec<IdlAccount>,
    #[serde(default)]
    instructions: Vec<IdlInstruction>, // Added this line
    #[serde(default)]
    types: Vec<IdlDefinedType>,
}

#[derive(Deserialize, Debug)]
struct IdlInstruction {
    name: String,
    #[serde(default)]
    args: Vec<IdlField>, // Added this line to capture instruction arguments
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

    let mut generated_code = quote! {};

    for account in &idl.accounts {
        let name = Ident::new(&account.name, proc_macro2::Span::call_site());

        // 1. Find the type definition that matches the account name
        // Anchor 0.31 separates 'accounts' from the actual 'type' definitions
        let matching_type = idl.types.iter().find(|t| t.name == account.name);

        if let Some(idl_type_wrapper) = matching_type {
            let fields: Vec<_> = idl_type_wrapper
                .ty
                .fields
                .iter()
                .map(|f| {
                    let field_name = Ident::new(&f.name, proc_macro2::Span::call_site());

                    let field_type = match f.ty.as_str() {
                        "u64" => quote! { u64 },
                        "i64" => quote! { i64 },
                        "string" => quote! { String },
                        "bool" => quote! { bool },
                        "publicKey" => quote! { bevy_solana_core::WrappedPubkey },
                        _ => quote! { u64 },
                    };

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

    // --- Step 5: Generate Instruction Structs with Fields ---
    for instr in &idl.instructions {
        let mut chars = instr.name.chars();
        let name_str = match chars.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
        };
        let name = Ident::new(&name_str, proc_macro2::Span::call_site());

        // Map the IDL types to Rust types for each argument
        let fields: Vec<_> = instr
            .args
            .iter()
            .map(|arg| {
                let field_name = Ident::new(&arg.name, proc_macro2::Span::call_site());

                // Re-using your existing type-mapping logic
                let field_type = match arg.ty.as_str() {
                    "u64" => quote! { u64 },
                    "i64" => quote! { i64 },
                    "string" => quote! { String },
                    "bool" => quote! { bool },
                    "publicKey" => quote! { bevy_solana_core::WrappedPubkey },
                    _ => quote! { u64 }, // Default fallback
                };

                quote! { pub #field_name: #field_type }
            })
            .collect();

        let instr_gen = quote! {
            #[derive(bevy::prelude::Component, Debug, Clone, Default)]
            pub struct #name {
                #( #fields, )*
            }
        };
        generated_code.extend(instr_gen);
    }

    generated_code.into()
}
