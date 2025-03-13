#![no_std]
#![no_main]

use kernel::prelude::*;
use kernel::module;

module! {
    type: HelloRustModule,
    name: "hello_rust",
    author: "Your Name",
    description: "A simple Rust kernel module",
    license: "GPL",
}

struct HelloRustModule;

impl kernel::Module for HelloRustModule {
    fn init() -> Result<Self> {
        pr_info!("Hello, Rust kernel module loaded!\n");
        Ok(HelloRustModule)
    }
}

impl Drop for HelloRustModule {
    fn drop(&mut self) {
        pr_info!("Goodbye, Rust kernel module!\n");
    }
}

