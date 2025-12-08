use embed_it::{Blake3_256Hash, Embed};
use hex::ToHex;

#[derive(Embed)]
#[embed(
    path = "../dist",
    dir(
        derive(Blake3),
        field(factory = ETagHeaderValue, name = etag, trait_name = DirEtagField, global)
    ),
    file(
        derive(Blake3), derive(Zstd), derive(Brotli),
        field(factory = ETagHeaderValue, name = etag, trait_name = FileEtagField, global)
    )
)]
pub struct Dist;

pub struct ETagHeaderValue {
    pub value: String,
}

impl ETagHeaderValue {
    pub fn create<T: Blake3_256Hash + ?Sized>(v: &T) -> Self {
        let hex_sha: String = v.blake3_256().encode_hex();
        Self {
            value: format!("\"{}\"", hex_sha),
        }
    }
}

impl DirFieldFactory for ETagHeaderValue {
    type Field = Self;

    fn create<T: Dir + ?Sized>(data: &T) -> Self::Field {
        Self::create(data)
    }
}

impl FileFieldFactory for ETagHeaderValue {
    type Field = Self;

    fn create<T: File + ?Sized>(data: &T) -> Self::Field {
        Self::create(data)
    }
}
