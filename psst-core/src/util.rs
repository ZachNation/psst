use crate::error::Error;
use num_traits::{One, WrappingAdd};
use quick_protobuf::{BytesReader, MessageRead, MessageWrite, Writer};
use std::{
    io,
    io::SeekFrom,
    mem,
    net::{Shutdown, TcpStream},
    time::Duration,
};

const HTTP_CONNECT_TIMEOUT: Duration = Duration::from_millis(4 * 1000);

const HTTP_IO_TIMEOUT: Duration = Duration::from_millis(4 * 1000);

pub fn default_ureq_agent_builder(proxy_url: Option<&str>) -> Result<ureq::AgentBuilder, Error> {
    let builder = ureq::AgentBuilder::new()
        .timeout_connect(HTTP_CONNECT_TIMEOUT)
        .timeout_read(HTTP_IO_TIMEOUT)
        .timeout_write(HTTP_IO_TIMEOUT);
    if let Some(url) = proxy_url {
        let proxy = ureq::Proxy::new(url)?;
        Ok(builder.proxy(proxy))
    } else {
        Ok(builder)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Sequence<T>(T);

impl<T: One + WrappingAdd> Sequence<T> {
    pub fn new(value: T) -> Self {
        Sequence(value)
    }

    pub fn advance(&mut self) -> T {
        let next = self.0.wrapping_add(&T::one());
        mem::replace(&mut self.0, next)
    }
}

pub struct OffsetFile<T> {
    stream: T,
    offset: u64,
}

impl<T: io::Seek> OffsetFile<T> {
    pub fn new(mut stream: T, offset: u64) -> io::Result<OffsetFile<T>> {
        stream.seek(SeekFrom::Start(offset))?;
        Ok(OffsetFile { stream, offset })
    }
}

impl<T: io::Read> io::Read for OffsetFile<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }
}

impl<T: io::Write> io::Write for OffsetFile<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stream.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}

impl<T: io::Seek> io::Seek for OffsetFile<T> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let offset_pos = match pos {
            SeekFrom::Start(offset) => SeekFrom::Start(offset + self.offset),
            from_end_or_current => from_end_or_current,
        };
        let new_pos = self.stream.seek(offset_pos)?;
        let offset_new_pos = new_pos.saturating_sub(self.offset);
        Ok(offset_new_pos)
    }
}

pub struct TcpShutdown {
    stream: TcpStream,
    shutdown: bool,
}

impl TcpShutdown {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            shutdown: false,
        }
    }

    pub fn shutdown(&mut self) {
        if !self.shutdown {
            self.shutdown = true;
            let _ = self.stream.shutdown(Shutdown::Both);
        }
    }

    pub fn has_been_shut_down(&self) -> bool {
        self.shutdown
    }
}

pub fn serialize_protobuf<T>(msg: &T) -> Result<Vec<u8>, Error>
where
    T: MessageWrite,
{
    let mut buf = Vec::with_capacity(msg.get_size());
    let mut writer = Writer::new(&mut buf);
    msg.write_message(&mut writer)?;
    Ok(buf)
}

pub fn deserialize_protobuf<T>(buf: &[u8]) -> Result<T, Error>
where
    T: MessageRead<'static>,
{
    let mut reader = BytesReader::from_bytes(buf);
    let msg = {
        let static_buf: &'static [u8] = unsafe {
            // Sigh.  While `quick-protobuf` supports `--owned` variations of messages, they
            // are not compatible with `--dont_use_cow` flag, which, by itself, is already
            // producing messages that fully own their fields.  Therefore, we can pretend
            // the byte slice is static, because `msg` does not retain it.
            std::mem::transmute(buf)
        };
        T::from_reader(&mut reader, static_buf)?
    };
    Ok(msg)
}
