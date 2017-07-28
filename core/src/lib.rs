#![allow(unknown_lints)]

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate specs_derive;
extern crate specs;

#[macro_use]
extern crate derive_more;
#[allow(unused_imports)]
#[macro_use]
extern crate maplit;


#[macro_use]
pub mod util;
pub mod model;

pub mod system;
pub mod bootstrap;
pub mod collision;
