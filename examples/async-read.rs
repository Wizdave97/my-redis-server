use tokio::{fs::File, io::AsyncReadExt};

#[tokio::main]
async fn main() {
    let mut buffer = [0u8; 32];
    let mut file = File::open("./examples/hello-redis.rs").await.unwrap();

    let n = file.read(&mut buffer).await.unwrap();

    println!("Read {} bytes: {:#?}", n , buffer)
}