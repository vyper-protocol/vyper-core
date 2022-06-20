use anchor_lang::prelude::*;

pub trait Input {
    fn is_valid(&self) -> Result<()>;
}
