#[macro_use]
extern crate thiserror;

pub mod error;
pub mod ser;

pub use error::{Error, Result};
pub use ser::to_hashmap;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
