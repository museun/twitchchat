use super::*;

#[tokio::test]
async fn encode_raw() {
    let mut out = vec![];
    encode(&raw("PRIVMSG #test :this is a test"), &mut out)
        .await
        .unwrap();
    assert_eq!(out, b"PRIVMSG #test :this is a test\r\n");
}

#[tokio::test]
async fn encode_pong() {
    let mut out = vec![];
    encode(&pong("123456789"), &mut out).await.unwrap();
    assert_eq!(out, b"PONG :123456789\r\n");
}

#[tokio::test]
async fn encode_ping() {
    let mut out = vec![];
    encode(&ping("123456789"), &mut out).await.unwrap();
    assert_eq!(out, b"PING 123456789\r\n");
}

#[tokio::test]
async fn encode_join() {
    let mut out = vec![];
    encode(&join("#museun"), &mut out).await.unwrap();
    assert_eq!(out, b"JOIN #museun\r\n");
}

#[tokio::test]
async fn encode_part() {
    let mut out = vec![];
    encode(&part("#museun"), &mut out).await.unwrap();
    assert_eq!(out, b"PART #museun\r\n");
}

#[tokio::test]
async fn encode_privmsg() {
    let mut out = vec![];
    encode(&privmsg("#museun", "this is a test of a line"), &mut out)
        .await
        .unwrap();
    assert_eq!(
        out,
        "PRIVMSG #museun :this is a test of a line\r\n".as_bytes()
    );

    let mut out = vec![];
    encode(&privmsg("#museun", &"foo ".repeat(500)), &mut out)
        .await
        .unwrap();
    assert_eq!(
        out,
        format!("PRIVMSG #museun :{}\r\n", &"foo ".repeat(500)).as_bytes()
    );
}
