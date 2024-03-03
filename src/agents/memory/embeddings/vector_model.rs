use byteorder::{BigEndian, ReadBytesExt};
use bytes::{BufMut, BytesMut};
use std::convert::TryInto;
use std::error::Error;

use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::postgres::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueRef};
use sqlx::{Decode, Encode, Postgres};

use super::EmbeddingVector;
#[allow(unused)]
impl EmbeddingVector {
    pub(crate) fn from_sql(
        mut buf: &[u8],
    ) -> Result<EmbeddingVector, Box<dyn Error + Sync + Send>> {
        let dim = buf.read_u16::<BigEndian>()?;
        let unused = buf.read_u16::<BigEndian>()?;
        if unused != 0 {
            return Err("expected unused to be 0".into());
        }

        let mut vec = vec![0.0; dim as usize];
        buf.read_f32_into::<BigEndian>(&mut vec)?;

        Ok(EmbeddingVector(vec))
    }

    pub(crate) fn to_sql(&self, w: &mut BytesMut) -> Result<(), Box<dyn Error + Sync + Send>> {
        let dim = self.0.len();
        w.put_u16(dim.try_into()?);
        w.put_u16(0);

        for v in self.0.iter() {
            w.put_f32(*v);
        }

        Ok(())
    }
}

impl Encode<'_, Postgres> for EmbeddingVector {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        let mut w = BytesMut::new();
        self.to_sql(&mut w).unwrap();
        buf.extend(&w[..]);
        IsNull::No
    }
}

impl Decode<'_, Postgres> for EmbeddingVector {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        let buf = <&[u8] as Decode<Postgres>>::decode(value)?;
        EmbeddingVector::from_sql(buf)
    }
}

impl PgHasArrayType for EmbeddingVector {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_vector")
    }
}
