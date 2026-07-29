// SPDX-License-Identifier: AGPL-3.0-or-later

//! riboCipher framing validation for transport-tier IPC.

use biomeos_types::constants::ribocipher;

#[test]
fn signal_clear_is_0xec() {
    assert_eq!(ribocipher::SIGNAL_CLEAR, 0xEC);
}

#[test]
fn version_1_is_0x01() {
    assert_eq!(ribocipher::VERSION_1, 0x01);
}

#[test]
fn prefix_bytes_are_two_byte_sequence() {
    let prefix = [ribocipher::SIGNAL_CLEAR, ribocipher::VERSION_1];
    assert_eq!(prefix.len(), 2);
    assert_eq!(prefix, [0xEC, 0x01]);
}

#[tokio::test]
async fn write_ribocipher_signal_produces_correct_prefix() {
    use biomeos_core::ipc::write_ribocipher_signal;

    let mut buf = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buf);
    let mut async_cursor = tokio::io::BufWriter::new(&mut cursor);
    write_ribocipher_signal(&mut async_cursor).await.unwrap();
    tokio::io::AsyncWriteExt::flush(&mut async_cursor)
        .await
        .unwrap();
    drop(async_cursor);
    assert_eq!(&buf[..2], &[0xEC, 0x01]);
}

#[tokio::test]
async fn ribocipher_prefix_precedes_payload() {
    use tokio::io::AsyncWriteExt;

    let mut buf = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buf);
    {
        let mut writer = tokio::io::BufWriter::new(&mut cursor);
        writer
            .write_all(&[ribocipher::SIGNAL_CLEAR, ribocipher::VERSION_1])
            .await
            .unwrap();
        writer.write_all(b"{\"jsonrpc\":\"2.0\"}").await.unwrap();
        writer.flush().await.unwrap();
    }
    assert_eq!(&buf[..2], &[0xEC, 0x01]);
    assert_eq!(&buf[2..], b"{\"jsonrpc\":\"2.0\"}");
}
