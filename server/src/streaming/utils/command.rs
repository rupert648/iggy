use iggy::error::Error;
use tokio::process::Command;

pub async fn ps_aux() -> Result<Vec<u8>, Error> {
  let output = Command::new("ps")
      .arg("aux")
      .output()
      .await?;

  Ok(output.stdout)
}

pub async fn top() -> Result<Vec<u8>, Error> {
  let output = Command::new("top")
      .arg("-l")
      .arg("1")
      .output()
      .await?;

  Ok(output.stdout)
}

pub async fn lsof() -> Result<Vec<u8>, Error> {
  let output = Command::new("lsof")
      .arg("-i")
      .output()
      .await?;

  Ok(output.stdout)
}