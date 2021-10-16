use blake2::{Blake2s, Digest};

fn main() {
    let mut hasher = Blake2s::new();
    hasher.update(b"Hello");

    let res = hasher.finalize();

    dbg!(hex::encode(res));
}
