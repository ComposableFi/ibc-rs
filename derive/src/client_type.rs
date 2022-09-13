use crate::{State};
use convert_case::{Case, Casing};

use quote::quote;
use syn::__private::TokenStream2;


impl State {
    pub fn impl_from_str_for_client_type(
        &self,
        const_idents: &[TokenStream2],
    ) -> proc_macro2::TokenStream {
        let this = &self.self_ident;
        let cases = self
            .clients
            .iter()
            .zip(const_idents)
            .map(|(client, const_ident)| {
                let variant_ident = &client.variant_ident;
                let attrs = &client.attrs;
                quote! {
                    #(#attrs)*
                    Self::#const_ident => Ok(Self::#variant_ident),
                }
            });

        quote! {
            impl core::str::FromStr for #this {
                type Err = Error;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    match s {
                        #(#cases)*
                        _ => Err(Error::unknown_client_type(s.to_string())),
                    }
                }
            }
        }
    }

    pub fn impl_client_type(&self) -> proc_macro2::TokenStream {
        let this = &self.self_ident;

        let const_idents = self
            .clients
            .iter()
            .map(|client| {
                let id = syn::Ident::new(
                    &format!(
                        "{}_STR",
                        client.variant_ident.to_string().to_case(Case::UpperSnake)
                    ),
                    client.variant_ident.span(),
                );
                quote! { #id }
            })
            .collect::<Vec<_>>();
        let consts = self
            .clients
            .iter()
            .zip(&const_idents)
            .map(|(client, const_ident)| {
                let variant_ident = &client.variant_ident;
                let attrs = &client.attrs;
                let variant_str = variant_ident.to_string();

                let const_val = syn::LitStr::new(
                    &format!(
                        "{}-{}",
                        client.discriminant.as_ref().unwrap(),
                        variant_str.to_case(Case::Kebab)
                    ),
                    variant_ident.span(),
                );
                quote! {
                    #(#attrs)*
                    const #const_ident: &'static str = #const_val;
                }
            });

        let fn_as_str_cases =
            self.clients
                .iter()
                .zip(&const_idents)
                .map(|(client, const_ident)| {
                    let variant_ident = &client.variant_ident;
                    let attrs = &client.attrs;
                    quote! {
                        #(#attrs)*
                        Self::#variant_ident => Self::#const_ident,
                    }
                });

        let impl_from_str_for_client_type = self.impl_from_str_for_client_type(&const_idents);

        quote! {
            impl #this {
                #(#consts)*

                pub fn as_str(&self) -> &'static str {
                    match self {
                        #(#fn_as_str_cases)*
                    }
                }
            }

            #impl_from_str_for_client_type
        }
    }
}
