#![cfg(feature = "process")]

use anyhow::Error as AnyhowError;
use mkutils::Utils;

#[tokio::test]
async fn test_copy() -> Result<(), AnyhowError> {
    let string = "hello world";

    string.write_to_clipboard().await?;

    let clipboard_string = String::read_from_clipboard().await?;

    assert_eq!(string, clipboard_string);

    ().ok()
}
