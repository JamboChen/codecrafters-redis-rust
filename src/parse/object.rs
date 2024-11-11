use std::fmt::Display;

#[derive(Copy, Clone)]
pub enum Object {}

impl Display for Object {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
