use proc_macro::TokenStream;
use quote::quote;
#[proc_macro_derive(ServerConfig)]
pub fn derive_with_server_config(input: TokenStream) -> TokenStream {
    let derive_input = syn::parse_macro_input!(input as syn::DeriveInput);
    let struct_ident = derive_input.ident;
    quote! {
        impl common::WithServerConfig for #struct_ident {
            fn listening_address(&self) -> SocketAddr {
                self.listening_address
            }

            fn client_max_connections(&self) -> usize {
                self.client_max_connections
            }

            fn worker_threads(&self) -> usize {
                self.worker_threads
            }
        }
    }
        .into()
}
#[proc_macro_derive(UsernameConfig)]
pub fn derive_with_username_config(input: TokenStream) -> TokenStream {
    let derive_input = syn::parse_macro_input!(input as syn::DeriveInput);
    let struct_ident = derive_input.ident;
    quote! {
        impl common::WithUsernameConfig for #struct_ident {
            fn username(&self) -> &str {
                &self.username
            }
        }
    }
        .into()
}
#[proc_macro_derive(LogConfig)]
pub fn derive_with_log_config(input: TokenStream) -> TokenStream {
    let derive_input = syn::parse_macro_input!(input as syn::DeriveInput);
    let struct_ident = derive_input.ident;
    quote! {
        impl common::WithLogConfig for #struct_ident {
            fn log_directory(&self) -> &std::path::Path {
                &self.log_directory
            }
            fn log_name_prefix(&self) -> &str {
                &self.log_name_prefix
            }
            fn max_log_level(&self) -> &str {
                &self.max_log_level
            }
        }
    }
        .into()
}
#[proc_macro_derive(UserRepositoryConfig)]
pub fn derive_with_user_repo_config(input: TokenStream) -> TokenStream {
    let derive_input = syn::parse_macro_input!(input as syn::DeriveInput);
    let struct_ident = derive_input.ident;
    quote! {
        impl common::WithUserRepositoryConfig for #struct_ident {
            fn refresh_interval_sec(&self) -> u64 {
                self.user_repo_refresh_interval
            }
        }
    }
        .into()
}
#[proc_macro_derive(FileSystemUserRepoConfig)]
pub fn derive_with_fs_user_repo_config(input: TokenStream) -> TokenStream {
    let derive_input = syn::parse_macro_input!(input as syn::DeriveInput);
    let struct_ident = derive_input.ident;
    quote! {
        impl common::WithFileSystemUserRepoConfig for #struct_ident {
            fn user_repo_directory(&self) -> &Path {
                &self.user_repo_directory
            }
            fn public_key_file_name(&self) -> &str {
                &self.user_info_public_key_file_name
            }
            fn private_key_file_name(&self) -> &str {
                &self.user_info_private_key_file_name
            }
            fn user_info_file_name(&self) -> &str {
                &self.user_info_file_name
            }
        }
    }
        .into()
}
