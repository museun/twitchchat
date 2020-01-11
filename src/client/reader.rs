use super::*;

pub(super) async fn read_loop<R>(
    read: R,
    dispatcher: Arc<Mutex<Dispatcher>>,
) -> Result<Status, Error>
where
    R: AsyncRead + Send + Sync + Unpin + 'static,
{
    let mut reader = tokio::io::BufReader::new(read).lines();
    while let Some(line) = reader.next().await {
        let line = line? + "\r\n";
        for msg in crate::decode(&line) {
            let msg = msg?;
            log::trace!("< {}", msg.raw.escape_debug());
            dispatcher.lock().await.dispatch(&msg);
        }
    }
    Ok(Status::Eof)
}
