//! # Get-Session Procedural Macro
//!
//! This macro is built specifically for Nova. It takes a directory path, and
//! expands to match a variable `target` against all the names of the child
//! modules of said path (assuming the path is a module folder).
//!
//! For example, the invocation `get_session::generate_match!("base/")` in the
//! directory structure...
//!
//! ```none
//! base/
//!     module_1.rs
//!     module_2/
//!         mod.rs
//!     module_3.rs
//!     ...
//!     module_n/
//!         mod.rs
//! ```
//!
//! ...expands to the folloing `match` statement:
//!
//!
//! ```none
//! match target {
//!     "module_1" => crate::games::module_1::Session::initialize,
//!     "module_2" => crate::games::module_2::Session::initialize,
//!     ...
//!     "module_n" => crate::games::module_n::Session::initialize,
//!     _ => panic!("Could not find game module!")
//! }
//! ```
//!
//! This helps automate core code generation every time a new game is added.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/16/2023 (maxfierro@berkeley.edu)

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use std::env;
use std::fs;

/// # Get-Session Procedural Macro
///
/// This macro is built specifically for Nova. It takes a directory path, and
/// expands to match a variable `target` against all the names of the child
/// modules of said path (assuming the path is a module folder).
///
/// For example, the invocation `get_session::generate_match!("base/")` in the
/// directory structure...
///
/// ```none
/// base/
///     module_1.rs
///     module_2/
///         mod.rs
///     module_3.rs
///     ...
///     module_n/
///         mod.rs
/// ```
///
/// ...expands to the folloing `match` statement:
///
///
/// ```none
/// match target {
///     "module_1" => crate::games::module_1::Session::initialize,
///     "module_2" => crate::games::module_2::Session::initialize,
///     ...
///     "module_n" => crate::games::module_n::Session::initialize,
///     _ => panic!("Could not find game module!")
/// }
/// ```
///
/// This helps automate core code generation every time a new game is added.
#[proc_macro]
pub fn generate_match(__input: TokenStream) -> TokenStream {
    // Get the absolute path of the directory where the macro was called from
    let __manifest_dir =
        env::var("CARGO_MANIFEST_DIR").expect("Failed to get Cargo manifest directory");
    let mut __macro_call_path =
        fs::canonicalize(__manifest_dir).expect("Failed to canonicalize Cargo manifest directory");
    __macro_call_path.push(__input.to_string().trim_matches('"'));

    // Collect the module names by iterating over the entries in the full base directory
    let __match_arms: Vec<proc_macro2::TokenStream> = fs::read_dir(__macro_call_path)
        .expect("Failed to read directory")
        .filter_map(|entry| {
            if let Ok(entry) = entry {
                if let Some(entry_name) = entry.file_name().to_str() {
                    if entry_name.ends_with(".rs") && entry_name != "mod.rs" {
                        return Some(entry_name[..entry_name.len() - 3].to_owned());
                    } else if entry_name != "mod.rs" {
                        return Some(entry_name.to_owned());
                    }
                }
            }
            None
        })
        .map(|s| {
            let solution: proc_macro2::TokenStream = s.parse().unwrap();
            quote! {
                #s => crate::games::#solution::Session::initialize,
            }
        })
        .collect();

    // Generate the match statement
    let __output_internal = quote! {
        match target {
            #(#__match_arms)*
            _ => panic!("Could not find game module!")
        }
    };

    // Return the generated code as TokenStream
    __output_internal.into()
}
