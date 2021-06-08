use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};
use tokio::{self, io};

#[tokio::main]
async fn main() {
    let socket = TcpStream::connect("127.0.0.1:6142").await.unwrap();

    let (mut rd, mut wr) = io::split(socket);
    let _ = tokio::spawn(async move {
        wr.write_all(b"hello\r\n").await?;
        wr.write_all(b"world\r\n").await?;

        Ok::<(), io::Error>(())
    });

    let mut buffer = [0u8; 128];

    loop {
        let n = rd.read(&mut buffer).await.unwrap();

        if n == 0 {
            break;
        }

        println!("Got: {:#?}", String::from_utf8(buffer[..n].to_vec()));
    }
    
}