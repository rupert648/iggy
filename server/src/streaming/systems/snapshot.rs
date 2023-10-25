use crate::streaming::systems::system::System;
use async_zip::tokio::write::ZipFileWriter;
use async_zip::{Compression, ZipEntryBuilder};
use iggy::error::Error;
use tokio::fs::File;
use crate::streaming::utils;

impl System {
  pub async fn create_snapshot_file(&self) -> Result<String, Error> {
    // TODO: file path okay?
    let file_name = format!(
        "{}/snapshot-{}.zip",
        self.config.get_system_path(),
        chrono::Utc::now().timestamp()
    );

    let mut file = File::create(file_name.clone()).await?;
    let mut writer = ZipFileWriter::with_tokio(&mut file);

    // TODO: warnings

    let ps_aux = utils::command::ps_aux().await?;
    let ps_aux_builder = ZipEntryBuilder::new("ps_aux.txt".into(), Compression::Deflate);
    writer.write_entry_whole(ps_aux_builder, &ps_aux).await;

    let top = utils::command::top().await?;
    let top_builder = ZipEntryBuilder::new("top_builder.txt".into(), Compression::Deflate);
    writer.write_entry_whole(top_builder, &top).await;

    let lsof = utils::command::lsof().await?;
    println!("lsof: {:?}", lsof);
    let lsof_builder = ZipEntryBuilder::new("lsof.txt".into(), Compression::Deflate);
    writer.write_entry_whole(lsof_builder, &lsof).await;
    
    writer.close().await;

    println!("Zip file created successfully!");
    
    Ok(file_name)
}
}