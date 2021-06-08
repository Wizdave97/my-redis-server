

use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpListener};
use tokio::{self, io::{self}, task::{JoinHandle}};

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6142").await.unwrap();

    // loop {
    //     let (mut socket, _) = listener.accept().await?;

    //     tokio::spawn(async move {
    //         let (mut rd, mut wr) = socket.split();
    //         io::copy(&mut rd, &mut wr).await.or_else::<io::Error, _>(|err| {
    //             println!("Error occurred while copying: {:?}", err);
    //             Ok(0)
    //         })
    //     });

    // }

    let task:JoinHandle<Result<(), io::Error>> = tokio::spawn(async move {
        loop {
            let (mut socket, _) = listener.accept().await?;
            let mut buffer = vec![0; 1024];
            match socket.read(&mut buffer).await {
                Ok(0) => Ok(()),
                Ok(n) => {
                    match socket.write_all(&buffer[..n]).await {
                        Ok(_) => Ok(()),
                        Err(err) => Err(err)
                    }
                    
                },
                Err(err) => Err(err)
            }?;
            
        }
    });

    
    task.await.unwrap()
}