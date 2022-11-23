// 告诉 rustc 只有在禁用 test 标志时才编译 “no-std”
#![cfg_attr(not(test), no_std)]
// 告诉 rustc 只有在启用 test 标志时才编译 “test feature”
#![cfg_attr(test, feature(test))]

mod rotaryencoder;
pub use rotaryencoder::*;

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
