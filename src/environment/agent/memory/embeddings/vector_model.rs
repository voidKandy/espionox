use byteorder::{BigEndian, ReadBytesExt};
use bytes::{BufMut, BytesMut};
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::convert::TryInto;
use std::error::Error;

use bytes::BytesMut;
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::postgres::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueRef};
use sqlx::{Decode, Encode, Postgres, Type};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmbeddingVector(Vec<f32>);

impl From<Vec<f32>> for EmbeddingVector {
    fn from(v: Vec<f32>) -> Self {
        EmbeddingVector(v)
    }
}

impl Into<Vec<f32>> for EmbeddingVector {
    fn into(self) -> Vec<f32> {
        self.0
    }
}

#[allow(unused)]
impl EmbeddingVector {
    pub fn to_vec(&self) -> Vec<f32> {
        self.0.clone()
    }

    pub fn score_l2(&self, other: &Self) -> f32 {
        let sum_of_squares: f32 = self
            .0
            .iter()
            .zip(other.0.iter())
            .map(|(&x, &y)| (x - y).powi(2))
            .sum();

        sum_of_squares.sqrt()
    }

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

impl PartialEq for EmbeddingVector {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Type<Postgres> for EmbeddingVector {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("vector")
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
