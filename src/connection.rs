use std::io::{self, Cursor};

use async_trait::async_trait;
use bytes::{Buf, BytesMut};
use mini_redis::{
    frame::{Error, Frame},
    Result,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

#[async_trait]
pub trait ConnectionTrait {
    fn new(stream: TcpStream) -> Self;
    async fn read_frame(&mut self) -> Result<Option<Frame>>;
    async fn write_frame(&mut self, frame: &Frame) -> Result<()>;
    fn parse_frame(&mut self) -> Result<Option<Frame>>;
    async fn write_decimal(&mut self, val: u64) -> io::Result<()>;
}

pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

#[async_trait]
impl ConnectionTrait for Connection {
    fn new(stream: TcpStream) -> Self {
        Self {
            stream: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(4096), // 4KB のキャパシティを持つバッファを確保する
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

            // バッファデータが足りなかった場合
            // ソケットからさらにデータを読み取ることを試みる
            //
            // 成功した場合、バイト数が返ってくる
            // `0` は "ストリームの終わり"、つまり相手側がコネクションを閉じたことを意味する
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                // 接続をきれいにシャットダウンするため、read バッファが空になるようにしなければならない
                // もしデータが残っているなら、それは相手がフレーム送信している途中でソケットを閉じたということを意味する
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err("connection reset by peer".into());
                }
            }
        }
    }

    /// フレーム全体をソケットに書き込む
    /// write システムコールの呼び出しを少なくするために、書き込みをバッファ化する。
    /// write バッファを用意して、フレームはソケットに書き込まれるより前にエンコードされた状態でバッファへと書き込まれる。
    /// しかし、read_frame() とは違い、ソケットへの書き込み前にすべてのフレームがバイト列としてバッファ化されるとは限らない。
    ///
    /// ref: バッファされた write https://zenn.dev/magurotuna/books/tokio-tutorial-ja/viewer/framing#%E3%83%90%E3%83%83%E3%83%95%E3%82%A1%E3%81%95%E3%82%8C%E3%81%9F-write
    /// バッファされた write を実装するため、BufWrite 構造体 を利用する。
    /// この構造体は AsyncWrite トレイトを実装する型 T によって初期化され、BufWriter 自身も AsyncWrite を実装しています。BufWriter に対して write が呼び出されると、内部の writer へと直接書き込むのではなく、バッファへと書き込みを行います。バッファがいっぱいになったら、コンテンツは内部の writer へと「流され」[1]、内部バッファのデータは消去されます。特定のケースにおいて、バッファをバイパスすることを可能にする最適化も存在しています。
    async fn write_frame(&mut self, frame: &Frame) -> Result<()> {
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

        /*
        最後に self.stream.flush().await を呼び出している。
        BufWriter は中間バッファに書き込みを蓄えるため、write を呼び出してもデータがソケットへと書き込まれることは保証されていない。
        return する前にフレームがソケットへと書き込まれてほしいので、flush() を呼んでいる。
        flush() を呼ぶことで、バッファの中で保留状態となっているデータがすべてソケットへと書き込まれる。

        write_frame() 内で flush() を 呼び出さないという代替案がある。
        代わりに、flush() 関数を Connection のメソッドとして提供する。
        こうすることで、呼び出し側が write バッファへと複数の小さいフレームを書き込んで、それからまとめてソケットへと書き込む、といったことができるようになる。
        こうすると write システムコールが 1 回で済む。
        しかし、このような実装は Connection API を複雑化させてしまう。Mini-Redis の目標の 1 つに「シンプルさ」というのがあるため、
        ここでは fn write_frame() の中で flush().await を呼び出すという実装にすることにした。
         */
        let _ = self.stream.flush().await;

        Ok(())
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

                // バッファからフレーム分を読み捨てる
                self.buffer.advance(len);

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

    /// Write a decimal frame to the stream
    async fn write_decimal(&mut self, val: u64) -> io::Result<()> {
        use std::io::Write;

        // Convert the value to a string
        let mut buf = [0u8; 12];
        let mut buf = Cursor::new(&mut buf[..]);
        write!(&mut buf, "{}", val)?;

        let pos = buf.position() as usize;
        self.stream.write_all(&buf.get_ref()[..pos]).await?;
        self.stream.write_all(b"\r\n").await?;

        Ok(())
    }
}
