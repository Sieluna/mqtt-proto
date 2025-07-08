use alloc::vec::Vec;

use bytes::Bytes;

use crate::{
    read_string, read_u16, write_string, write_u16, AsyncRead, Encodable, Error, IoErrorKind, Pid,
    QoS, QosPid, SyncWrite, TopicName,
};

use super::Header;

/// Publish packet body type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Publish {
    pub dup: bool,
    pub retain: bool,
    pub qos_pid: QosPid,
    pub topic_name: TopicName,
    pub payload: Bytes,
}

#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for Publish {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Publish {
            dup: u.arbitrary()?,
            qos_pid: u.arbitrary()?,
            retain: u.arbitrary()?,
            topic_name: u.arbitrary()?,
            payload: Bytes::from(Vec::<u8>::arbitrary(u)?),
        })
    }
}

impl Publish {
    pub fn new(qos_pid: QosPid, topic_name: TopicName, payload: Bytes) -> Self {
        Publish {
            dup: false,
            retain: false,
            qos_pid,
            topic_name,
            payload,
        }
    }

    pub async fn decode_async<T: AsyncRead + Unpin>(
        reader: &mut T,
        header: Header,
    ) -> Result<Self, Error> {
        let mut remaining_len = header.remaining_len as usize;
        let topic_name = read_string(reader).await?;
        remaining_len = remaining_len
            .checked_sub(2 + topic_name.len())
            .ok_or(Error::InvalidRemainingLength)?;
        let qos_pid = match header.qos {
            QoS::Level0 => QosPid::Level0,
            QoS::Level1 => {
                remaining_len = remaining_len
                    .checked_sub(2)
                    .ok_or(Error::InvalidRemainingLength)?;
                QosPid::Level1(Pid::try_from(read_u16(reader).await?)?)
            }
            QoS::Level2 => {
                remaining_len = remaining_len
                    .checked_sub(2)
                    .ok_or(Error::InvalidRemainingLength)?;
                QosPid::Level2(Pid::try_from(read_u16(reader).await?)?)
            }
        };
        let payload = if remaining_len > 0 {
            let mut data = alloc::vec![0u8; remaining_len];
            reader.read_exact(&mut data).await.map_err(|e| match e {
                embedded_io_async::ReadExactError::UnexpectedEof => {
                    Error::IoError(IoErrorKind::UnexpectedEof)
                }
                embedded_io_async::ReadExactError::Other(e) => e.into(),
            })?;
            data
        } else {
            Vec::new()
        };
        Ok(Publish {
            dup: header.dup,
            qos_pid,
            retain: header.retain,
            topic_name: TopicName::try_from(topic_name)?,
            payload: Bytes::from(payload),
        })
    }
}

impl Encodable for Publish {
    fn encode<W: SyncWrite>(&self, writer: &mut W) -> Result<(), Error> {
        write_string(writer, &self.topic_name)?;
        match self.qos_pid {
            QosPid::Level0 => {}
            QosPid::Level1(pid) | QosPid::Level2(pid) => {
                write_u16(writer, pid.value())?;
            }
        }
        writer.write_all(self.payload.as_ref())?;
        Ok(())
    }

    fn encode_len(&self) -> usize {
        let mut length = 2 + self.topic_name.len();
        match self.qos_pid {
            QosPid::Level0 => {}
            QosPid::Level1(_) | QosPid::Level2(_) => {
                length += 2;
            }
        }
        length += self.payload.len();
        length
    }
}
