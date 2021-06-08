use tokio::net::TcpStream;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, BufWriter};
use mini_redis::Frame;
use std::io::Cursor;
pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: Vec<u8>,
    cursor: usize
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(stream),
            buffer: vec![0; 4096],
            cursor: 0
        }
    }

    pub async fn read_frame(&mut self) -> io::Result<Option<Frame>>  {
        loop {
            if let Ok(Some(frame)) = self.parse_frame() {
                return Ok(Some(frame))
            }
    
            if self.buffer.len() == self.cursor {
                self.buffer.resize(self.cursor * 2, 0);
            }
            
            let n = self.stream.read(&mut self.buffer[self.cursor..]).await.unwrap();
    
            if n == 0 {
                if self.cursor == 0 {
                    return Ok(None)
                }
                else {
                    return Err(io::Error::new(io::ErrorKind::ConnectionReset, "Connection reset by peer"))
                }
            }
            else {
                self.cursor+=n;
            }
        }

    }

    pub async fn write_frame(&mut self, frame: &Frame) -> io::Result<Option<()>> {
        match frame {
            Frame::Simple(val) => {
                self.stream.write_u8(b'+').await?;
                self.stream.write_all(val.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Error(val) => {
                self.stream.write_u8(b'-').await?;
                self.stream.write_all(val.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Integer(val) => {
                self.stream.write_u8(b':').await?;
                self.write_decimal(*val).await?;
            }
            Frame::Null => {
                self.stream.write_all(b"$-1\r\n").await?;
            }
            Frame::Bulk(val) => {
                let len = val.len();
    
                self.stream.write_u8(b'$').await?;
                self.write_decimal(len as u64).await?;
                self.stream.write_all(val).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Array(_val) => unimplemented!(),
        }
    
        self.stream.flush().await.unwrap();
    
        Ok(Some(()))
    }

    pub async fn write_decimal(&mut self, val: u64) -> io::Result<Option<()>> {
        use std::io::Write;

        // Convert the value to a string
        let mut buf = [0u8; 12];
        let mut buf = Cursor::new(&mut buf[..]);
        write!(&mut buf, "{}", val)?;

        let pos = buf.position() as usize;
        self.stream.write_all(&buf.get_ref()[..pos]).await?;
        self.stream.write_all(b"\r\n").await?;

        Ok(Some(()))
    }

    // pub async fn write_value(&mut self, frame: &Frame) -> io::Result<Option<()>> {

    // }

    fn parse_frame(&mut self) -> io::Result<Option<Frame>> {
     
        let mut buf = Cursor::new(&self.buffer[..]);

        match Frame::check(&mut buf) {
            Ok(_) => {
                let len = buf.position() as usize;

                buf.set_position(0);
                let frame = Frame::parse(&mut buf).unwrap();
                let buffer_len = self.buffer.len();

                if buffer_len == (len + 1) {
                    self.buffer.clear();
                    self.buffer.resize(buffer_len, 0);
                }
                else {
                    let rem = self.buffer.split_off(len + 1);
                    self.buffer.clear();
                    self.buffer.resize(buffer_len, 0);
                    for (index, val) in rem.iter().enumerate() {
                        self.buffer[index] = *val;
                    }
                }
                self.cursor = 0;
                Ok(Some(frame))
            },
            Err(err) => {
                Err(io::Error::new(io::ErrorKind::InvalidData, err.to_string()))
            }
        }
        
    }
}