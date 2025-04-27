use anyhow::{Context, Result};
use std::path::Path;
use tantivy::{Index, IndexReader, ReloadPolicy};
// use tantivy::directory::{Directory, RamDirectory};
// use tracing::{debug, info, warn};
// use serde_json::Value;
// use std::fs;

pub struct SearchIndex {
    pub index: Index,
    pub reader: IndexReader,
}

impl SearchIndex {
    pub fn open_from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let index = Index::open_in_dir(&path)
            .with_context(|| format!("Failed to open index in {:?}", path.as_ref()))?;

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .context("Failed to create IndexReader")?;

        Ok(Self { index, reader })
    }

    // pub fn open_from_path_to_ram<P: AsRef<Path>>(path: P) -> Result<Self> {
    //     let index = Self::load_index_into_ram(path)?;

    //     let reader = index
    //         .reader_builder()
    //         .reload_policy(ReloadPolicy::OnCommitWithDelay)
    //         .try_into()
    //         .context("Failed to create IndexReader")?;

    //     Ok(Self { index, reader })
    // }

    // fn load_index_into_ram<P: AsRef<Path>>(path: P) -> Result<Index> {
    //     let path = path.as_ref();
    //     info!(?path, "Loading Tantivy index into RAM");

    //     let meta_path = path.join("meta.json");
    //     info!(?meta_path, "Reading meta.json");
    //     let meta_bytes = fs::read(&meta_path)
    //         .with_context(|| format!("Failed to read meta.json at {:?}", meta_path))?;

    //     let meta_json: Value =
    //         serde_json::from_slice(&meta_bytes).context("Failed to parse meta.json as JSON")?;

    //     let segment_ids = meta_json["segments"]
    //         .as_array()
    //         .context("meta.json does not contain 'segments' array")?
    //         .iter()
    //         .filter_map(|seg| seg.get("segment_id"))
    //         .filter_map(|id| id.as_str())
    //         .map(|id| id.replace('-', ""))
    //         .collect::<Vec<_>>();

    //     info!(count = segment_ids.len(), "Found segments");
    //     info!(segments = ?segment_ids, "Found segment IDs");

    //     let mut files_to_copy = vec!["meta.json".to_string()];

    //     for entry in fs::read_dir(path)? {
    //         let entry = entry?;
    //         let file_name = entry.file_name().to_string_lossy().to_string();
    //         debug!(?file_name, "Found file in directory");

    //         if segment_ids
    //             .iter()
    //             .any(|seg_id| file_name.starts_with(seg_id))
    //         {
    //             debug!(?file_name, "File matched segment");
    //             files_to_copy.push(file_name);
    //         }
    //     }

    //     info!(count = files_to_copy.len(), "Copying files into RAM");

    //     let ram_dir = RamDirectory::create();
    //     for file_name in &files_to_copy {
    //         let full_path = path.join(file_name);
    //         match fs::read(&full_path) {
    //             Ok(data) => {
    //                 ram_dir
    //                     .atomic_write(Path::new(file_name), &data)
    //                     .with_context(|| format!("Failed to write {} into RAM", file_name))?;
    //                 debug!(?file_name, "Copied to RAM");
    //             }
    //             Err(err) => {
    //                 warn!(?file_name, ?err, "Failed to read file — skipping");
    //                 return Err(err.into());
    //             }
    //         }
    //     }

    //     info!("Opening index from RAM");
    //     Ok(Index::open(ram_dir).context("Failed to open index from RAM")?)
    // }
}
