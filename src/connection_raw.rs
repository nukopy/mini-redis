use std::io::{self, Cursor};

use async_trait::async_trait;
use mini_redis::{
    frame::{Error, Frame},
    Result,
};
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

use crate::connection::ConnectionTrait;

pub struct Connection {
    stream: TcpStream,
    buffer: Vec<u8>,
    cursor: usize,
}

#[async_trait]
impl ConnectionTrait for Connection {
    fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            buffer: vec![0; 4096], // 4KB のキャパシティを持つバッファを確保する
            cursor: 0,
        }
    }

    /// コネクションからフレームを読み取る
    /// データはソケットから読み取られ、read バッファへ蓄えられます。フレームがパースされたら、対応するデータがバッファから削除されます。
    /// EOF に到達したら `None` を返す
    async fn read_frame(&mut self) -> Result<Option<Frame>> {
        loop {
            // バッファされたデータからフレームをパースすることを試みる
            // 十分な量のデータがバッファに蓄えられていたら、フレームをパースして return する
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            // バッファが十分なキャパシティを必ず持つように調整する
            if self.buffer.len() == self.cursor {
                // Grow the buffer
                self.buffer.resize(self.buffer.len() * 2, 0); // 0 で埋める
            }

            // 何バイト読んだかを記録しながらバッファに読み取っていく
            let n = self.stream.read(&mut self.buffer[self.cursor..]).await?;

            if n == 0 {
                if self.cursor == 0 {
                    return Ok(None);
                } else {
                    return Err("connection reset by peer".into());
                }
            } else {
                // カーソルを更新する
                self.cursor += n;
            }
        }
    }

    /// コネクションにフレームを書き込む
    async fn write_frame(&mut self, frame: &Frame) -> Result<()> {
        unimplemented!()
    }

    fn parse_frame(&mut self) -> Result<Option<Frame>> {
        // Buf 型を作る
        let mut buf = Cursor::new(&self.buffer[..]);

        // フレーム全体が取得可能かどうかをチェックする
        match Frame::check(&mut buf) {
            // Frame::check
            Ok(_) => {
                // フレームのバイト長を取得する
                let len = buf.position() as usize;

                // `parse` を呼び出すため、内部カーソルをリセットする
                buf.set_position(0);

                // フレームをパースする
                let frame = Frame::parse(&mut buf)?;

                // バッファからフレーム分を読み捨てる vec
                self.buffer.drain(..len);

                // 呼び出し側にフレームを返す
                Ok(Some(frame))
            }
            Err(Error::Incomplete) => {
                // フレーム全体が取得できなかったので、バッファを維持する
                Ok(None)
            }
            Err(e) => Err(e.into()),
        }
    }

    // Write a decimal frame to the stream
    async fn write_decimal(&mut self, val: u64) -> io::Result<()> {
        unimplemented!()
    }
}
