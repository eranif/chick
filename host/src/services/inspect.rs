use crate::models::package::DpkgRecord;
use crate::models::requests::Layer;
use crate::utils::errors::ImagePullError;
use crate::utils::line_reader::LineReader;
use flate2::read::GzDecoder;
use oci_client::{Client, Reference, secrets::RegistryAuth};
use std::io::Read;
use tar::Archive;

pub async fn pull_and_inspect_image(
    oci_client: &Client,
    image_reference: &str,
) -> Result<Vec<Layer>, Box<dyn std::error::Error>> {
    let mut layers = Vec::<Layer>::new();
    let reference = Reference::try_from(image_reference).map_err(|e| ImagePullError {
        message: format!("Invalid image reference: {}", e),
    })?;

    let (manifest, digest) = oci_client
        .pull_image_manifest(&reference, &RegistryAuth::Anonymous)
        .await
        .map_err(|e| ImagePullError {
            message: format!("Failed to pull image manifest: {}", e),
        })?;

    for layer in manifest.layers {
        let mut layer_buffer: Vec<u8> = Vec::new();
        match oci_client
            .pull_blob(&reference, &layer, &mut layer_buffer)
            .await
        {
            Ok(_) => {
                println!(
                    "Layer of media type: {}, pulled successfully: {}",
                    layer.media_type, layer.digest
                );

                let packages = inspect_layer(&layer_buffer, "var/lib/dpkg/status")?;
                let layer = Layer {
                    layer: layer.digest.to_string(),
                    packages,
                };
                layers.push(layer);
            }
            Err(e) => {
                return Err(Box::new(ImagePullError {
                    message: e.to_string(),
                }))
            }
        }
    }
    println!(
        "Image pulled successfully: {}, digest: {}",
        image_reference, digest
    );
    Ok(layers)
}

fn inspect_layer(
    layer_data: &[u8],
    target_path: &str,
) -> Result<Vec<DpkgRecord>, Box<dyn std::error::Error>> {
    let gz = GzDecoder::new(layer_data);
    let mut archive = Archive::new(gz);
    let mut packages: Vec<DpkgRecord> = vec![];

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        if path.to_string_lossy() == target_path {
            let mut buffer = Vec::new();
            entry.read_to_end(&mut buffer)?;

            let mut reader = LineReader::new(buffer.as_slice());
            while let Some(package) = read_package(&mut reader)? {
                packages.push(package);
            }
        }
    }

    Ok(packages)
}

fn read_package(
    reader: &mut LineReader,
) -> Result<Option<DpkgRecord>, Box<dyn std::error::Error>> {
    let mut package = DpkgRecord::default();

    let mut line = reader.next();
    if line.is_none() {
        return Ok(None);
    }

    loop {
        if let Some(line_str) = line {
            if line_str.is_empty() {
                break;
            }
            let (key, value) = line_str.split_once(':').unwrap_or(("", ""));
            match key {
                "Package" => package.package = value.trim().to_string(),
                "Status" => package.status = value.trim().to_string(),
                "Version" => package.version = value.trim().to_string(),
                _ => {}
            }
        }
        
        line = reader.next();
        if line.is_none() {
            break;
        }
    }
    
    println!("Package: {}", package.package);
    Ok(Some(package))
}