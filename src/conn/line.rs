use anyhow::{anyhow, Context, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};

// TODO: might be problematic for overengineered json packages later on!
const MAX_TCP_PACKET_SIZE: usize = 65536;

pub struct LineConnWriter(WriteHalf<TcpStream>);

pub struct LineConnReader(ReadHalf<TcpStream>);

pub enum ReadError {
    ParseError(anyhow::Error),
    EnvError(anyhow::Error),
}

pub async fn line_con(stream: TcpStream) -> (LineConnWriter, LineConnReader) {
    let (reader, writer) = tokio::io::split(stream);
    (LineConnWriter(writer), LineConnReader(reader))
}

//TODO: eh this convenience function is stupid!!
// we realistically should only pass in the line
// caller should take care of serialization
impl LineConnWriter {
    pub async fn write<T>(&mut self, data: T) -> Result<()>
    where
        T: Into<String>,
    {
        let mut line: String = data.try_into().context("failed to convert into line")?;
        line.push('\0');
        self.0
            .write_all(line.as_bytes())
            .await
            .context("failed to write line")?;
        Ok(())
    }
}

// TODO: the signal, that the connection is done ... is rather implicit
impl LineConnReader {
    pub async fn read<T>(&mut self) -> Result<Option<T>, ReadError>
    where
        T: TryFrom<String>,
        T::Error: Into<anyhow::Error>,
    {
        let line = match self
            .read_string()
            .await
            .context("failed to read line")
            .map_err(|e| ReadError::EnvError(e))?
        {
            Some(line) => line,
            None => return Ok(None),
        };

        let parsed_res: Result<T, T::Error> = line.try_into();
        match parsed_res{
            Ok(t) => Ok(Some(t)),
            Err(e) => Err(ReadError::ParseError(e.into())),
        }
    }

    async fn read_string(&mut self) -> Result<Option<String>> {
        let mut accumulated = Vec::new();
        let mut buf = [0u8; 4096];

        loop {
            let n = self.0.read(&mut buf).await?;

            if n == 0 {
                // Connection closed
                return if accumulated.is_empty() {
                    Ok(None)
                } else {
                    Err(anyhow!("Connection closed before null terminator"))
                };
            }

            if let Some(pos) = buf[..n].iter().position(|&b| b == 0) {
                accumulated.extend_from_slice(&buf[..pos]);

                return match String::from_utf8(accumulated) {
                    Ok(s) => Ok(Some(s)),
                    Err(e) => Err(e).context("failed tgo decode as utf-8"),
                };
            }

            accumulated.extend_from_slice(&buf[..n]);
        }
    }
}
