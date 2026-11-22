#![allow(clippy::redundant_pub_crate)]

mod define_message;
mod from_str_parts;
mod impl_message;
mod message_from_impl;

pub(crate) use define_message::*;
pub(crate) use from_str_parts::*;
pub(crate) use impl_message::*;
pub(crate) use message_from_impl::*;

#[cfg(test)]
mod assert_message_str;
#[cfg(test)]
#[allow(unused_imports)] // rustc is hallucinating
pub(crate) use assert_message_str::*;
