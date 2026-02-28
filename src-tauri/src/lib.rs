/// 调用曼波 TTS API，解析 JSON 响应，直接返回音频 URL。
#[tauri::command]
async fn synthesize_speech(text: String, api_key: String, format: String) -> Result<String, String> {
    let client = reqwest::Client::new();
    // 只允许 mp3 / wav，其余回退到 mp3
    let fmt = if format == "wav" { "wav" } else { "mp3" };

    let response = client
        .get("https://api.milorapart.top/apis/mbAIsc")
        .query(&[("text", text.as_str()), ("format", fmt)])
        .bearer_auth(&api_key)
        .send()
        .await
        .map_err(|e| format!("网络请求失败：{}", e))?;

    let status = response.status();
    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败：{}", e))?;

    // code 非 200 时作为错误返回
    let code = json.get("code").and_then(|v| v.as_i64()).unwrap_or(0);
    if code != 200 {
        let msg = json
            .get("msg")
            .and_then(|v| v.as_str())
            .unwrap_or("未知错误");
        return Err(format!("API 错误 {}：{}", status, msg));
    }

    // 提取音频 URL
    json.get("url")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "响应中缺少 url 字段".to_string())
}

/// 从 URL 下载音频文件，弹出系统保存对话框让用户选择保存位置。
#[tauri::command]
async fn save_audio(app: tauri::AppHandle, url: String) -> Result<(), String> {
    use tauri_plugin_dialog::DialogExt;

    // 推断文件名
    let filename = url
        .split('/')
        .last()
        .unwrap_or("audio.mp3")
        .to_string();

    // ① 先弹对话框——立即响应，不等下载
    let path = app
        .dialog()
        .file()
        .set_file_name(&filename)
        .blocking_save_file();

    // 用户取消则直接返回
    let path = match path {
        Some(tauri_plugin_dialog::FilePath::Path(p)) => p,
        _ => return Ok(()),
    };

    // ② 用户选好路径后才开始下载
    let bytes = reqwest::get(&url)
        .await
        .map_err(|e| format!("下载失败：{}", e))?
        .bytes()
        .await
        .map_err(|e| format!("读取失败：{}", e))?;

    std::fs::write(&path, &bytes)
        .map_err(|e| format!("写入文件失败：{}", e))?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![synthesize_speech, save_audio])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
