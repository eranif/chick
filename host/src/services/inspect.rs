use crate::models::package::DpkgRecord;
use crate::models::requests::Layer;
use crate::utils::errors::{ImagePullError, HyperlightGuestError};
use flate2::read::GzDecoder;
use oci_client::{Client, Reference, secrets::RegistryAuth};
use hyperlight_common::flatbuffer_wrappers::function_types::{ParameterValue, ReturnType, ReturnValue};
use hyperlight_host::{UninitializedSandbox, MultiUseSandbox, sandbox_state::transition::Noop, sandbox_state::sandbox::EvolvableSandbox};
use tar::Archive;
use std::io::Read;

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
        let mut sandbox_cfg = hyperlight_host::sandbox::SandboxConfiguration::default();
        sandbox_cfg.set_input_data_size(33554432);
        sandbox_cfg.set_host_exception_size(33554432);
        sandbox_cfg.set_host_function_definition_size(33554432);
        sandbox_cfg.set_guest_error_buffer_size(33554432);
        sandbox_cfg.set_guest_panic_context_buffer_size(33554432);
        sandbox_cfg.set_heap_size(33554432);
        sandbox_cfg.set_output_data_size(33554432);
                                      
        if path.to_string_lossy() == target_path {
            let mut buffer = Vec::new();
            entry.read_to_end(&mut buffer)?;

            let uninitialized_sandbox = UninitializedSandbox::new(
                hyperlight_host::GuestBinary::FilePath("/usr/local/bin/chick-guest".to_string()),
                Some(sandbox_cfg),
                None,
                None,
            ).expect("Failed to create uninitialized sandbox");
        
            let mut multi_use_sandbox: MultiUseSandbox = uninitialized_sandbox.evolve(Noop::default()).expect("Failed to evolve sandbox");
            
            let result = multi_use_sandbox.call_guest_function_by_name(
                "Inspect",
                ReturnType::VecBytes,
                Some(vec![ParameterValue::VecBytes(buffer)]),
            );
        
            match result {
                Ok(ReturnValue::String(result)) => {
                    packages = serde_json::from_str(&result).unwrap();
                },
                Ok(_) => {
                    return Err(Box::new(HyperlightGuestError{
                        message: format!("Invalid return type from inspect")
                    }
                    ))
                },
                Err(e) => {
                    return Err(Box::new(e))
                }
            }
        }
    }

    Ok(packages)
}
