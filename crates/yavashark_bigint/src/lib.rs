mod storage;

use crate::storage::BigIntStorage;

#[derive(Clone)]
pub struct YSBigInt {
    storage: BigIntStorage,
}