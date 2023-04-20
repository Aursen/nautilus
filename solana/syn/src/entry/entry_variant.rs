use nautilus_idl::idl_instruction::{IdlInstruction};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type};

use crate::{entry::required_account::{RequiredAccountSubtype}, object::NautilusObject};

use super::{
    entry_enum::NautilusEntrypointEnum,
    required_account::{
        metadata_ident, mint_authority_ident, self_account_ident, RequiredAccount,
        RequiredAccountType,
    },
};

#[derive(Debug)]
pub struct NautilusEntrypointEnumVariant {
    pub discriminant: u8,
    pub variant_ident: Ident,
    pub variant_args: Vec<(Ident, Type)>,
    pub required_accounts: Vec<RequiredAccount>,
    pub call_ident: Ident,
    pub call_context: Vec<CallContext>,
}

#[derive(Debug)]
pub enum CallContext {
    Nautilus(NautilusObject),
    Arg(Ident),
}

impl NautilusEntrypointEnumVariant {
    pub fn new(
        discriminant: u8,
        variant_ident: Ident,
        variant_args: Vec<(Ident, Type)>,
        call_ident: Ident,
        call_context: Vec<CallContext>,
    ) -> Self {
        let required_accounts = RequiredAccount::condense(
            call_context
                .iter()
                .filter_map(|ctx| match ctx {
                    CallContext::Nautilus(n) => {
                        let req = n.get_required_accounts();
                        let mut accounts = vec![];
                        accounts.extend(req.0);
                        match req.1 {
                            Some(r) => accounts.extend(r),
                            None => (),
                        };
                        Some(accounts)
                    }
                    CallContext::Arg(_) => None,
                })
                .collect(),
        );
        Self {
            discriminant,
            variant_ident,
            variant_args,
            required_accounts,
            call_ident,
            call_context,
        }
    }

    fn build_match_arm_logic(&self) -> TokenStream {
        let instruction_name = self.variant_ident.to_string();
        let all_accounts = self.required_accounts.iter().map(|r| {
            let ident = match &r.account_type {
                RequiredAccountType::Account(subtype) => match &subtype {
                    RequiredAccountSubtype::SelfAccount => self_account_ident(&r.ident),
                    RequiredAccountSubtype::Metadata => metadata_ident(&r.ident),
                    RequiredAccountSubtype::MintAuthority => mint_authority_ident(&r.ident),
                },
                _ => r.ident.clone(),
            };
            quote! { let #ident = next_account_info(accounts_iter)?; }
        });
        let mut object_inits = vec![];
        let mut call_args = vec![];
        {
            self.call_context.iter().for_each(|ctx| {
                match ctx {
                    CallContext::Nautilus(obj) => match &obj.entry_config {
                        Some(config) => {
                            let arg_ident = &config.arg_ident;
                            let obj_type = &obj.ident;
                            let required_accounts_for_obj = obj.get_required_accounts();

                            let accounts_for_read = required_accounts_for_obj.0;
                            let read_call_idents = accounts_for_read.iter().map(|r| {
                                let t: TokenStream = r.into();
                                t
                            });

                            let accounts_for_create_option = required_accounts_for_obj.1;

                            match accounts_for_create_option {
                                Some(accounts_for_create) => {
                                    let create_call_idents = accounts_for_create.iter().map(|r| {
                                        let t: TokenStream = r.into();
                                        t
                                    });
                                    object_inits.push(
                                        quote! { let mut #arg_ident = Create::new(
                                            #(#create_call_idents,)*
                                            Box::new(#obj_type::new(#(#read_call_idents,)* false))
                                    ); },
                                    );
                                },
                                None => {
                                    if config.is_signer { 
                                        object_inits.push(
                                            quote! { let #arg_ident = Signer::new(Box::new(#obj_type::new(#(#read_call_idents,)* true))); },
                                        );
                                    } else if config.is_mut {
                                        object_inits.push(
                                            quote! { let #arg_ident = Mut::new(Box::new(#obj_type::new(#(#read_call_idents,)* true))); },
                                        );
                                    } else { 
                                        object_inits.push(
                                            quote! { let #arg_ident = #obj_type::new(#(#read_call_idents,)* true); },
                                        );
                                    }
                                },
                            };
                            
                            call_args.push(quote! { #arg_ident })
                        }
                        None => {
                            panic!("Error processing entrypoint: `entry_config` not set.")
                        }
                    },
                    CallContext::Arg(arg) => call_args.push(quote! { #arg }),
                };
            });
        }
        let call_ident = &self.call_ident;
        quote::quote! {
            {
                msg!("Instruction: {}", #instruction_name);
                let accounts_iter = &mut accounts.iter();
                #(#all_accounts)*
                #(#object_inits)*
                #call_ident(#(#call_args,)*)
            }
        }
    }
}

impl From<&NautilusEntrypointEnumVariant> for (TokenStream, TokenStream, IdlInstruction) {
    fn from(value: &NautilusEntrypointEnumVariant) -> Self {
        let variant_ident = &value.variant_ident;
        let enum_ident = NautilusEntrypointEnum::enum_ident();
        let (arg_names, arg_types): (Vec<Ident>, Vec<Type>) =
            value.variant_args.clone().into_iter().unzip();
        let match_arm_logic = value.build_match_arm_logic();
        (
            quote! { #variant_ident(#(#arg_types,)*), },
            quote! { #enum_ident::#variant_ident(#(#arg_names,)*) => #match_arm_logic, },
            value.into(),
        )
    }
}
