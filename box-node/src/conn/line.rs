use std::collections::VecDeque;

use anyhow::{anyhow, Context, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};

// TODO: might be problematic for overengineered json packages later on!
// const MAX_TCP_PACKET_SIZE: usize = 65536;

pub struct LineConnWriter(WriteHalf<TcpStream>);

pub struct LineConnReader {
    queue: VecDeque<String>,
    reader: ReadHalf<TcpStream>,
}

#[derive(Debug)]
pub enum ReadError {
    ParseError(anyhow::Error),
    EnvError(anyhow::Error),
}

pub async fn line_con(stream: TcpStream) -> (LineConnWriter, LineConnReader) {
    let (reader, writer) = tokio::io::split(stream);
    (
        LineConnWriter(writer),
        LineConnReader {
            reader,
            queue: VecDeque::with_capacity(16),
        },
    )
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
        match parsed_res {
            Ok(t) => Ok(Some(t)),
            Err(e) => Err(ReadError::ParseError(e.into())),
        }
    }

    async fn read_string(&mut self) -> Result<Option<String>> {
        if let Some(line) = self.queue.pop_front() {
            return Ok(Some(line));
        }

        let mut accumulated = Vec::new();
        let mut buf = [0u8; 4096];

        loop {
            let n = self.reader.read(&mut buf).await?;

            if n == 0 {
                return if accumulated.is_empty() {
                    Ok(None)
                } else {
                    Err(anyhow!("Connection closed before null terminator"))
                };
            }

            let mut start = 0;
            let mut first_result = None;

            for i in 0..n {
                if buf[i] == 0 {
                    if i + 1 < n && buf[i + 1] == 0 {
                        break;
                    }

                    let mut line_bytes = accumulated.clone();
                    line_bytes.extend_from_slice(&buf[start..i]);

                    let s = String::from_utf8(line_bytes).context("failed to decode as utf-8")?;

                    if first_result.is_none() {
                        first_result = Some(s);
                    } else {
                        self.queue.push_back(s);
                    }

                    accumulated.clear();
                    start = i + 1;
                }
            }

            if let Some(result) = first_result {
                if start < n {
                    accumulated.extend_from_slice(&buf[start..n]);
                }
                return Ok(Some(result));
            }

            accumulated.extend_from_slice(&buf[..n]);
        }
    }
}
