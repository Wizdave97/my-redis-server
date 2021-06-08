use bytes::Bytes;
use mini_redis::client;
use tokio::{self, sync};

enum Command<'a> {
    Get {
        key: &'a str,
        resp: Responder<Option<Bytes>>
    },
    Set{
        key: &'a str, 
        value: String,
        resp: Responder<()>
    }
}

type Responder<T> = sync::oneshot::Sender<mini_redis::Result<T>>;

#[tokio::main]
async fn main() {
    use Command::*;
    let mut client = client::connect("127.0.0.1:6379").await.unwrap();
    let (tx, mut rx): (sync::mpsc::Sender<Command>, sync::mpsc::Receiver<Command>) = sync::mpsc::channel(32);
    let tx1 = tx.clone();
    let tx2 = tx.clone();
    let manager = tokio::spawn(async move {
        while let Some(cmd) = rx.recv().await {
            match cmd {
                Get{key, resp} => {
                    let res = client.get(key).await;
                    resp.send(res);
                }
                Set{key, value, resp} => {
                    let res = client.set(key, value.into()).await;

                    resp.send(res);
                }
            }
        }
    });

    let t1 = tokio::spawn(async move {
        let (tx, mut rx) = sync::oneshot::channel();
        tx1.send(Command::Get{key: "hello", resp: tx}).await;
        let resp = rx.await;

        println!("Get return value: {:?}", resp);
        
    });

    let t2 = tokio::spawn(async move {
        let (tx, mut rx) = sync::oneshot::channel();
        tx2.send(Command::Set{key: "foo", value: "bar".to_string(), resp: tx}).await;

        let resp = rx.await;

        println!("Set return value: {:?}", resp);
    });



    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
}