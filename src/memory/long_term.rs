#[cfg(feature = "long_term_memory")]
use crate::features::long_term_memory::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub enum LongTermMemory {
    #[default]
    None,
    #[cfg(feature = "long_term_memory")]
    Some(LtmHandler),
}

#[cfg(feature = "long_term_memory")]
impl From<LtmHandler> for LongTermMemory {
    fn from(handler: LtmHandler) -> Self {
        LongTermMemory::Some(handler)
    }
}

#[cfg(feature = "long_term_memory")]
impl TryInto<LtmHandler> for LongTermMemory {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<LtmHandler, Self::Error> {
        if let LongTermMemory::Some(handler) = self {
            Ok(handler)
        } else {
            Err(anyhow::anyhow!("LongTermMemory is None"))
        }
    }
}
