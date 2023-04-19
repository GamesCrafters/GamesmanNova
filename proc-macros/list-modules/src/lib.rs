//! # List-Modules Procedural Macro
//!
//! This macro creates a constant string slice list of all the module names
//! which are children of an indicated crate module folder. Paths are specified
//! relative to the cargo manifest directory.
//!
//! For example, calling this macro from `mod.rs` in the following file tree
//! with `list_modules::here!("parent/");`...
//!
//! ```none
//! parent/
//!     mod.rs
//!     child_1.rs
//!     child_2/
//!         mod.rs
//!         internal.rs
//!         other_internal/
//!             ...
//!         ...
//!     child_3.rs
//!     child_4.rs
//!     ...
//!     child_N/
//!         mod.rs
//! ```
//!
//! ...will result in the following list expansion:
//!
//! ```none
//! pub const LIST: [&str; N] = [
//!     "child_1",
//!     "child_2",
//!     "child_3",
//!     ...
//!     "child_N",
//! ];
//! ```
//!
//! Note that this is the only guaranteed behavior.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/16/2023 (maxfierro@berkeley.edu)

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use std::env;
use std::fs;

/// # List-Modules Procedural Macro
///
/// This macro creates a constant string slice list of all the module names
/// which are children of an indicated crate module folder. Paths are specified
/// relative to the cargo manifest directory.
///
/// For example, calling this macro from `mod.rs` in the following file tree
/// with `list_modules::here!("parent/");`...
///
/// ```none
/// parent/
///     mod.rs
///     child_1.rs
///     child_2/
///         mod.rs
///         internal.rs
///         other_internal/
///             ...
///         ...
///     child_3.rs
///     child_4.rs
///     ...
///     child_N/
///         mod.rs
/// ```
///
/// ...will result in the following list expansion:
///
/// ```none
/// pub const LIST: [&str; N] = [
///     "child_1",
///     "child_2",
///     "child_3",
///     ...
///     "child_N",
/// ];
/// ```
///
/// Note that this is the only guaranteed behavior.
#[proc_macro]
pub fn here(__input: TokenStream) -> TokenStream {
    // Get the absolute path of the directory where the macro was called from
    let __manifest_dir =
        env::var("CARGO_MANIFEST_DIR").expect("Failed to get Cargo manifest directory");
    let mut __macro_call_path =
        fs::canonicalize(__manifest_dir).expect("Failed to canonicalize Cargo manifest directory");
    __macro_call_path.push(__input.to_string().trim_matches('"'));

    // Collect the module names by iterating over the entries in the full base directory
    let __internal_module_names: Vec<String> = fs::read_dir(__macro_call_path)
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
        .collect();

    // Generate array of module names
    let __internal_module_array = quote! {
        [
            #(#__internal_module_names),*
        ]
    };

    // Get number of iterms in array to later insert them
    let __number_of_modules_in_array = __internal_module_names.len();

    // Generate the static list of string slices with the custom list name
    let __internal_macro_output: proc_macro2::TokenStream = quote! {
        pub const LIST: [&str; #__number_of_modules_in_array] = #__internal_module_array;
    };

    __internal_macro_output.into()
}
