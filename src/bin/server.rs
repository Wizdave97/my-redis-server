use bytes::Bytes;
use tokio::net::{TcpListener, TcpStream};
use std::{sync::{Mutex, Arc}};
use dashmap::DashMap;

type Db = Arc<Mutex<DashMap<String, Bytes>>>;
#[tokio::main]
pub async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let db: Db = Arc::new(Mutex::new(DashMap::new()));
    db.lock().as_mut().unwrap().insert("hello".to_string(), "world".into());
    loop {

        let (socket, _) = listener.accept().await.unwrap();
        let clone_db = Arc::clone(&db);
        tokio::spawn(async move {
            process(socket, clone_db).await
        });
        
        
    }
}


async fn process(socket: TcpStream, db: Db)  {
    use my_redis::Connection;
    use mini_redis::Frame;
    use mini_redis::Command::{self, Get, Set};

    
    let mut connection = Connection::new(socket);
    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match  db.lock().as_mut() {
            Ok(db)  => {
                match Command::from_frame(frame) {
                    Ok(Set(cmd)) => {
                        db.insert(cmd.key().into(), cmd.value().clone());
                        Frame::Simple("OK".to_string())
                    },
        
                    
                    Ok(Get(cmd)) => {
                        if let Some(value) = db.get(cmd.key()) {
                            
                            Frame::Bulk(value.clone().into())
                        }
                        else {
                            Frame::Null
                        }
                    },
                    Err(err) => panic!("Errored: {:?}", err),
                    Ok(cmd) => panic!("Unimplemented: {:?}", cmd)
                    
                }
            }
            Err(err) => {
                panic!("Failed to get database handle: {:?}", err)
            }
        };
        connection.write_frame(&response).await.unwrap();
    }
}


