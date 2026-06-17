
use std::collections::HashMap;
use std::path::PathBuf;


use std::sync::Arc;

#[cfg(target_arch = "wasm32")]
use {
    wasm_bindgen::JsCast,
};
pub async fn load_bytes(path: &str) -> Vec<u8> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::fs::read(path).expect("Failed to read file")
    }
    #[cfg(target_arch = "wasm32")]
    {
        let url = format!("/{}", path);
        let response = web_sys::window()
            .unwrap()
            .fetch_with_str(&url);
        
        let response = wasm_bindgen_futures::JsFuture::from(response)
            .await
            .unwrap();
        let response: web_sys::Response = response.dyn_into().unwrap();
        let buffer = wasm_bindgen_futures::JsFuture::from(
            response.array_buffer().unwrap()
        ).await.unwrap();
        
        js_sys::Uint8Array::new(&buffer).to_vec()
    }
}


pub async fn create_texture(
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    path: &str,
) -> wgpu::Texture {
    let bytes = load_bytes(path).await;
    let img = image::load_from_memory(&bytes)
        .expect("Failed to load texture")
        .to_rgba8();

    let (width, height) = img.dimensions();
    let rgba = img.into_raw();

    // 2. Create texture
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("sprite texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * width),
            rows_per_image: Some(height),
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    texture
}





fn resolve_relative_path(base_path: &str, relative_path: &str) -> String {
    let mut base = PathBuf::from(base_path);
    base.pop();

    let mut combined = base.join(relative_path);
    combined = PathBuf::from(
        combined
            .components()
            .fold(PathBuf::new(), |mut acc, c| {
                match c {
                    std::path::Component::ParentDir => { acc.pop(); }
                    std::path::Component::CurDir => {}
                    _ => acc.push(c.as_os_str()),
                }
                acc
            }),
    );

    combined.to_string_lossy().to_string()
}

//--------------------------------------
//----Reads one u32 from gpu memory----
//--------------------------------------
pub async fn gpu_readback_byte(device: &wgpu::Device, queue: &wgpu::Queue, gpu_buffer: &wgpu::Buffer, readback_buffer: &wgpu::Buffer, read_offset: u64) -> u32 {
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_buffer_to_buffer(
        &gpu_buffer,
        read_offset,
        &readback_buffer,
        0,
        4,
    );

    queue.submit(Some(encoder.finish()));

    let slice = readback_buffer.slice(..);

    slice.map_async(wgpu::MapMode::Read, |result| {
        result.unwrap();
    });

    device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: None,
    }).unwrap();

    let value: u32 = {
        let data = slice.get_mapped_range();
        u32::from_le_bytes(data[0..4].try_into().unwrap())
    };

    readback_buffer.unmap();
    return value;
}

#[cfg(target_arch = "wasm32")]
use {
    js_sys,
};
async fn load_shader_source(
    url: &str,
    seen: &mut std::collections::HashSet<String>,
) -> anyhow::Result<String> {
    if seen.contains(url) {
        return Ok(String::new());
    }
    seen.insert(url.to_string());


    let url = {
        #[cfg(all(target_arch = "wasm32", debug_assertions))]
        { format!("{}?t={}", url, js_sys::Date::now() as u64) } //DISABLE CACHING
        #[cfg(any(not(target_arch = "wasm32"), not(debug_assertions)))]
        { url }
    };

    let bytes = crate::utils::load_bytes(&url).await;
    let source = String::from_utf8(bytes)?;
    
    let include_regex = regex::Regex::new(r#"^\s*#include\s+"(.+?)"\s*$"#).unwrap();
    let mut result = String::new();
    for line in source.lines() {
        if let Some(cap) = include_regex.captures(line) {
            let include_path = &cap[1];
            let resolved = resolve_relative_path(&url, include_path);
            let included = Box::pin(load_shader_source(&resolved, seen)).await?;
            result.push_str(&included);
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }
    Ok(result)
}

pub async fn create_shader(
    device: &wgpu::Device,
    code: &str,
) -> anyhow::Result<wgpu::ShaderModule> {
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("shader"),
        source: wgpu::ShaderSource::Wgsl(code.into()),
    });

    Ok(module)
}

pub async fn load_shader_modules(
    device: &wgpu::Device,
    shader_files: &HashMap<&str, String>,
) -> anyhow::Result<HashMap<String, wgpu::ShaderModule>> {

    let mut out = std::collections::HashMap::new();

    for (key, path) in shader_files.iter() {
        let mut seen = std::collections::HashSet::new();

        let source = load_shader_source(path, &mut seen).await?;
        let module = create_shader(&device, &source).await?;

        out.insert(key.to_string(), module);
    }

    Ok(out)
}




